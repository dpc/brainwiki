// GET /a/b/c - search for a post with a/b/c tag
//   a is most important, c least important
//   if unique match, it's page id - respond with id
//   if not unique, respond with list of tags and posts to qualify
//   if no matches, remove c, try again
//
// POST / - create a new page
//    if the page with the same tags exists, return error
// PUT /a/b/c - update page
//    if the page with the same tags exists, return error
//
// DELETE /id - delete page
//
// POST /~login login
// ANY /~... other special stuff

use actix_web::{
    error, fs, http,
    middleware::{
        session::{CookieSessionBackend, RequestSession, SessionStorage},
        Finished, Middleware, Started,
    },
    server, App, AsyncResponder, Form, FromRequest, HttpMessage, HttpRequest, HttpResponse, Query,
    Result,
};

use futures::Future;

use crate::{
    config,
    data::{self, MatchType, PageId},
    opts::Opts,
    page::Page,
    tpl,
};

#[derive(Fail, Debug)]
enum UserError {
    #[fail(display = "Unauthorized")]
    Unauthorized,
}

type UserResult<T> = std::result::Result<T, UserError>;

impl error::ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            UserError::Unauthorized => HttpResponse::new(http::StatusCode::UNAUTHORIZED),
        }
    }
}

fn login_get(req: HttpRequest<State>) -> Result<HttpResponse> {
    let cur_url = req.path();
    let body = tpl::render(
        &tpl::login_tpl(),
        &tpl::login::Data {
            base: tpl::base::Data {
                title: config::WIKI_NAME_TEXT.into(),
                search_query: None,
            },
            cur_url: cur_url.into(),
        },
    );
    Ok(HttpResponse::Ok().body(body))
}

#[derive(Deserialize, Debug, Clone)]
struct PasswordForm {
    password: String,
}

fn login_post(req: HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = error::Error>> {
    Form::<PasswordForm>::extract(&req)
        .and_then(move |form| {
            if form.password == "bazinga" {
                req.session().set("logge_in", true)?;
                Ok(redirect_to_303("/"))
            } else {
                Ok(redirect_to_303("/~login"))
            }
        })
        .responder()
}

fn logout(_req: HttpRequest<State>) -> Result<HttpResponse> {
    Ok(redirect_to_303("/"))
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::TemporaryRedirect()
        .header("Location", location)
        .finish()
}

fn redirect_to_303(location: &str) -> HttpResponse {
    HttpResponse::Found().header("Location", location).finish()
}

#[derive(Debug, Serialize, Deserialize)]
struct PostInput {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PostResponse {
    redirect: String,
}

fn assert_is_authorized(req: &HttpRequest<State>) -> UserResult<()> {
    if !req
        .session()
        .get::<bool>("logged_in")
        .unwrap_or(None)
        .unwrap_or(false)
    {
        return Err(UserError::Unauthorized);
    }

    Ok(())
}

fn post(req: HttpRequest<State>) -> Result<Box<Future<Item = HttpResponse, Error = error::Error>>> {
    assert_is_authorized(&req)?;
    let data = req.state().data.clone();
    let data_dir = req.state().opts.data_dir.clone();

    Ok(req
        .json()
        .from_err()
        .and_then(move |input: PostInput| {
            let data_read = data.read();

            let new_page = Page::from_markdown(input.text.clone());

            let lookup = data_read.lookup_exact(new_page.tags.clone());

            if lookup != data::LookupOutcome::None {
                return Ok(HttpResponse::Conflict().body("{}"));
            }

            drop(data_read);

            data.write_new_file(&new_page, data_dir.as_path())?;
            Ok(HttpResponse::Ok().json(PostResponse {
                redirect: new_page.to_full_url(true),
            }))
        })
        .map_err(|e| {
            println!("2: {}", e);
            e
        })
        .responder())
}

#[derive(Debug, Serialize, Deserialize)]
struct PutInput {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PutResponse {
    redirect: String,
}

fn put(
    req: HttpRequest<State>,
) -> UserResult<Box<Future<Item = HttpResponse, Error = error::Error>>> {
    assert_is_authorized(&req)?;
    let cur_url = req.path().to_owned();
    let data = req.state().data.clone();

    Ok(req
        .json()
        .map_err(|e| {
            println!("{}", e);
            e
        })
        .from_err()
        .and_then(move |input: PutInput| {
            let (url_tags, _) = url_to_tags(cur_url.as_str());
            let data_read = data.read();
            let page_id = data_read.lookup(url_tags)?;

            let new_page = Page::from_markdown(input.text.clone());

            let lookup = data_read.lookup_exact(new_page.tags.clone());

            if lookup == data::LookupOutcome::Many {
                return Ok(HttpResponse::Conflict().body("{}"));
            } else if let data::LookupOutcome::One(id) = lookup {
                if id != page_id {
                    return Ok(HttpResponse::Conflict().body("{}"));
                }
            }

            let existing_path = data_read.path_by_id.get(&page_id).unwrap().clone();

            drop(data_read);

            data.replace_file(&existing_path, &new_page)?;

            Ok(HttpResponse::Ok().json(PutResponse {
                redirect: new_page.to_full_url(true),
            }))
        })
        .map_err(|e| {
            println!("2: {}", e);
            e
        })
        .responder())
}

fn get_index(
    match_: &data::Match,
    cur_url: &str,
    page_ids: &[PageId],
    data: &data::State,
) -> Result<HttpResponse> {
    let mut pages: Vec<_> = page_ids
        .iter()
        .map(|page_id| data.pages_by_id.get(&page_id).unwrap().clone())
        .collect();
    pages.sort_by(|n, m| n.title.cmp(&m.title));
    let body = tpl::render(
        &tpl::index_tpl(),
        &tpl::index::Data {
            base: tpl::base::Data {
                title: if match_.matching_tags.is_empty() {
                    config::WIKI_NAME_TEXT.into()
                } else {
                    match_.matching_tags.join("/")
                },
                search_query: None,
            },
            pages: pages,
            cur_url: cur_url.into(),
            narrowing_tags: match_.narrowing_tags.clone(),
            matching_tags: match_.matching_tags.clone(),
        },
    );

    Ok(HttpResponse::Ok().body(body))
}

fn url_to_tags(url: &str) -> (Vec<String>, bool) {
    let mut tags: Vec<_> = url.split("/").skip(1).map(Into::into).collect();

    let prefer_exact = if tags.last() == Some(&"".into()) {
        tags.pop();
        false
    } else {
        true
    };

    (tags, prefer_exact)
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
}

fn search_get(query: Query<SearchQuery>) -> Result<HttpResponse> {
    let tags: Vec<String> = query
        .q
        .trim()
        .split(|c| c == ' ' || c == ',')
        .filter(|s| s != &"")
        .map(Into::into)
        .collect();
    Ok(redirect_to(
        (String::from("/") + tags.join("/").as_str()).as_str(),
    ))
}

fn search_post(query: Form<SearchQuery>) -> Result<HttpResponse> {
    let tags: Vec<String> = query
        .q
        .trim()
        .split(|c| c == ' ' || c == ',')
        .filter(|s| s != &"")
        .map(Into::into)
        .collect();
    Ok(redirect_to_303(
        (String::from("/") + tags.join("/").as_str()).as_str(),
    ))
}

fn new_page(req: HttpRequest<State>) -> Result<HttpResponse> {
    let cur_url = req.path();

    let body = tpl::render(
        &tpl::new_tpl(),
        &tpl::new::Data {
            base: tpl::base::Data {
                title: "New post".into(),
                search_query: None,
            },
            cur_url: cur_url.into(),
        },
    );
    Ok(HttpResponse::Ok().body(body))
}

fn get(req: HttpRequest<State>) -> Result<HttpResponse> {
    let cur_url = req.path();
    let (tags, prefer_exact) = url_to_tags(cur_url);
    let data = req.state().data.read();

    let match_ = data.find_best_match(tags.clone(), prefer_exact);

    if match_.has_unmatched_tags() {
        return Ok(redirect_to(match_.to_precise_url(prefer_exact).as_str()));
    }

    match match_.type_ {
        MatchType::One(page_id) => {
            let page = data.pages_by_id.get(&page_id).unwrap();
            if match_.is_one() && match_.matching_tags.len() < page.tags.len() {
                return Ok(redirect_to(page.to_full_url(prefer_exact).as_str()));
            }
            let body = tpl::render(
                &tpl::view_tpl(),
                &tpl::view::Data {
                    base: tpl::base::Data {
                        title: page.title.clone(),
                        search_query: None,
                    },
                    page: page.clone(),
                    cur_url: cur_url.into(),
                    narrowing_tags: match_.narrowing_tags,
                },
            );
            Ok(HttpResponse::Ok().body(body))
        }
        MatchType::Many(ref page_ids) => get_index(&match_, cur_url, page_ids.as_slice(), &*data),
        MatchType::None => Ok(HttpResponse::Ok().body(format!("Not Found :("))),
    }
}

#[derive(Clone)]
struct State {
    data: data::SyncState,
    opts: Opts,
}

struct Logger;

impl<S> Middleware<S> for Logger {
    fn start(&self, req: &mut HttpRequest<S>) -> Result<Started> {
        println!("{} {}", req.method(), req.path());
        Ok(Started::Done)
    }

    fn finish(&self, _req: &mut HttpRequest<S>, _resp: &HttpResponse) -> Finished {
        Finished::Done
    }
}

pub fn start(data: data::SyncState, opts: Opts) {
    let state = State {
        data: data,
        opts: opts.clone(),
    };

    let mut listenfd = listenfd::ListenFd::from_env();

    let server = server::new(move || {
        App::with_state(state.clone())
            .middleware(Logger)
            .middleware(SessionStorage::new(
                CookieSessionBackend::signed(&[0; 32]).secure(false),
            ))
            .route("/~login", http::Method::GET, login_get)
            .route("/~login", http::Method::POST, login_post)
            .route("/~logout", http::Method::POST, logout)
            .route("/~search", http::Method::GET, search_get)
            .route("/~search", http::Method::POST, search_post)
            .route("/~new", http::Method::GET, new_page)
            .handler("/~theme", fs::StaticFiles::new(opts.theme_dir.clone()))
            .default_resource(|r| {
                r.get().f(get);
                r.post().f(post);
                r.put().f(put);
            })
    });
    if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    }.run();
}
