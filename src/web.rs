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
    error, fs,
    http::{self, StatusCode},
    middleware::{
        session::{
            CookieSessionBackend, RequestSession, SessionStorage,
        },
        Finished, Middleware, Started,
    },
    server, App, AsyncResponder, Form, FromRequest, HttpMessage,
    HttpRequest, HttpResponse, Query, Responder, Result,
};

use crate::settings::Site;
use std::sync::Arc;
use stpl::html::RenderExt;

const LOGGED_IN_COOKIE_NAME: &str = "logged_in";

use futures::Future;

use crate::{
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
            UserError::Unauthorized => {
                HttpResponse::new(http::StatusCode::UNAUTHORIZED)
            }
        }
    }
}

fn can_edit(req: &HttpRequest<State>) -> Result<bool> {
    let local = req.state().opts.local;
    let logged_in = req
        .session()
        .get::<bool>(LOGGED_IN_COOKIE_NAME)?
        .unwrap_or(false);
    Ok(local || logged_in)
}

fn can_login(req: &HttpRequest<State>) -> bool {
    let local = req.state().opts.local;
    let password_set =
        req.state().site_settings.hashed_password.is_some();

    !local && password_set
}

const BOOTSTRAP_MIN_CSS: &[u8] =
    include_bytes!("../theme/bootstrap.min.css");
const CUSTOM_CSS: &[u8] = include_bytes!("../theme/custom.css");
const FAVICON_ICO: &[u8] =
    include_bytes!("../theme/favicon.ico");

fn theme_get(req: HttpRequest<State>) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("");

    match name {
        "bootstrap.min.css" => {
            HttpResponse::build(StatusCode::OK)
                .content_type("text/css; charset=utf-8")
                .header(
                    http::header::CACHE_CONTROL,
                    "public, max-age=600",
                )
                .body(BOOTSTRAP_MIN_CSS)
        }
        "custom.css" => HttpResponse::build(StatusCode::OK)
            .content_type("text/css; charset=utf-8")
            .header(
                http::header::CACHE_CONTROL,
                "public, max-age=600",
            )
            .body(CUSTOM_CSS),
        "favicon.ico" => FAVICON_ICO.into(),
        _ => {
            return HttpResponse::new(http::StatusCode::NOT_FOUND)
        }
    }
}

fn login_get(req: HttpRequest<State>) -> Result<HttpResponse> {
    let cur_url = req.path();

    let hashed_password =
        req.state().site_settings.hashed_password.clone();
    let local = req.state().opts.local;

    if local || hashed_password.is_none() {
        return Ok(redirect_to_303("/"));
    }
    let base = tpl::base::Data::from(&req);
    let body = tpl::login::page(&tpl::login::Data {
        base: base,
        cur_url: cur_url.into(),
    });
    Ok(HttpResponse::Ok().body(body.render_to_vec()))
}

#[derive(Deserialize, Debug, Clone)]
struct PasswordForm {
    password: String,
}

fn login_post(
    req: HttpRequest<State>,
) -> Box<Future<Item = HttpResponse, Error = error::Error>> {
    let hashed_password =
        req.state().site_settings.hashed_password.clone();
    let local = req.state().opts.local;

    Form::<PasswordForm>::extract(&req)
        .and_then(move |form| {
            if local || hashed_password.is_none() {
                Ok(redirect_to_303("/"))
            } else if libpasta::verify_password(
                &hashed_password.unwrap(),
                form.password.clone(),
            ) {
                req.session().set(LOGGED_IN_COOKIE_NAME, true)?;
                Ok(redirect_to_303("/"))
            } else {
                Ok(redirect_to_303("/~login"))
            }
        })
        .responder()
}

fn logout(req: HttpRequest<State>) -> Result<HttpResponse> {
    req.session().remove(LOGGED_IN_COOKIE_NAME);
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

fn assert_is_authorized(
    req: &HttpRequest<State>,
) -> UserResult<()> {
    if !req
        .session()
        .get::<bool>(LOGGED_IN_COOKIE_NAME)
        .unwrap_or(None)
        .unwrap_or(false)
    {
        return Err(UserError::Unauthorized);
    }

    Ok(())
}

fn post(
    req: HttpRequest<State>,
) -> Result<Box<Future<Item = HttpResponse, Error = error::Error>>>
{
    assert_is_authorized(&req)?;
    let data = req.state().data.clone();
    let data_dir = req.state().opts.data_dir.clone();

    Ok(req
        .json()
        .from_err()
        .and_then(move |input: PostInput| {
            let data_read = data.read();

            let new_page =
                Page::from_markdown(input.text.clone());

            let lookup =
                data_read.lookup_exact(new_page.tags.clone());

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
) -> UserResult<
    Box<Future<Item = HttpResponse, Error = error::Error>>,
> {
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

            let new_page =
                Page::from_markdown(input.text.clone());

            let lookup =
                data_read.lookup_exact(new_page.tags.clone());

            if lookup == data::LookupOutcome::Many {
                return Ok(HttpResponse::Conflict().body("{}"));
            } else if let data::LookupOutcome::One(id) = lookup {
                if id != page_id {
                    return Ok(
                        HttpResponse::Conflict().body("{}")
                    );
                }
            }

            let existing_path = data_read
                .path_by_id
                .get(&page_id)
                .unwrap()
                .clone();

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
    req: &HttpRequest<State>,
    match_: &data::Match,
    cur_url: &str,
    page_ids: &[PageId],
    data: &data::State,
) -> Result<HttpResponse> {
    let mut pages: Vec<_> = page_ids
        .iter()
        .map(|page_id| {
            data.pages_by_id.get(&page_id).unwrap().clone()
        })
        .collect();
    pages.sort_by(|n, m| n.title.cmp(&m.title));
    let mut base = tpl::base::Data::from(req);

    base.title = if match_.matching_tags.is_empty() {
        req.state().site_settings.short_name.clone()
    } else {
        match_.matching_tags.join("/")
    };
    let body = tpl::index::page(&tpl::index::Data {
        base: base,
        pages: pages,
        cur_url: cur_url.into(),
        narrowing_tags: match_.narrowing_tags.clone(),
        matching_tags: match_.matching_tags.clone(),
    });

    Ok(HttpResponse::Ok().body(body.render_to_vec()))
}

fn url_to_tags(url: &str) -> (Vec<String>, bool) {
    let mut tags: Vec<_> =
        url.split("/").skip(1).map(Into::into).collect();

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

fn search_get(
    query: Query<SearchQuery>,
) -> Result<HttpResponse> {
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

fn search_post(
    query: Form<SearchQuery>,
) -> Result<HttpResponse> {
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
    assert_is_authorized(&req)?;
    let cur_url = req.path();

    let mut base = tpl::base::Data::from(&req);
    base.title = "New post".into();
    let body = tpl::new::page(&tpl::new::Data {
        base: base,
        cur_url: cur_url.into(),
    });
    Ok(HttpResponse::Ok().body(body.render_to_vec()))
}

fn get(req: HttpRequest<State>) -> Result<HttpResponse> {
    let cur_url = req.path();
    let (tags, prefer_exact) = url_to_tags(cur_url);
    let data = req.state().data.read();

    let match_ =
        data.find_best_match(tags.clone(), prefer_exact);

    if match_.has_unmatched_tags() {
        return Ok(redirect_to(
            match_.to_precise_url(prefer_exact).as_str(),
        ));
    }

    match match_.type_ {
        MatchType::One(page_id) => {
            let page = data.pages_by_id.get(&page_id).unwrap();
            if match_.is_one()
                && match_.matching_tags.len() < page.tags.len()
            {
                return Ok(redirect_to(
                    page.to_full_url(prefer_exact).as_str(),
                ));
            }
            let mut base = tpl::base::Data::from(&req);
            base.title = page.title.clone();
            let body = tpl::view::page(&tpl::view::Data {
                base: base,
                page: page.clone(),
                cur_url: cur_url.into(),
                narrowing_tags: match_.narrowing_tags,
            });
            Ok(HttpResponse::Ok().body(body.render_to_vec()))
        }
        MatchType::Many(ref page_ids) => get_index(
            &req,
            &match_,
            cur_url,
            page_ids.as_slice(),
            &*data,
        ),
        MatchType::None => {
            Ok(HttpResponse::Ok().body(format!("Not Found :(")))
        }
    }
}

#[derive(Clone)]
struct State {
    data: data::SyncState,
    opts: Opts,
    site_settings: Arc<Site>,
}

struct Logger;

impl<S> Middleware<S> for Logger {
    fn start(
        &self,
        req: &mut HttpRequest<S>,
    ) -> Result<Started> {
        println!("{} {}", req.method(), req.path());
        Ok(Started::Done)
    }

    fn finish(
        &self,
        _req: &mut HttpRequest<S>,
        _resp: &HttpResponse,
    ) -> Finished {
        Finished::Done
    }
}

impl<'a> From<&'a HttpRequest<State>>
    for crate::tpl::base::Data<'a>
{
    fn from(req: &'a HttpRequest<State>) -> Self {
        Self {
            title: req.state().site_settings.short_name.clone(),
            site_settings: &req.state().site_settings,
            can_edit: can_edit(req).unwrap_or(false),
            can_login: can_login(req),
        }
    }
}

pub fn start(
    data: data::SyncState,
    site_settings: Site,
    opts: Opts,
) {
    let state = State {
        data: data,
        opts: opts.clone(),
        site_settings: Arc::new(site_settings),
    };

    let mut listenfd = listenfd::ListenFd::from_env();

    let server = server::new(move || {
        let app = App::with_state(state.clone())
            .middleware(Logger)
            .middleware(SessionStorage::new(
                // TODO
                CookieSessionBackend::signed(&[0; 32])
                    .secure(false),
            ))
            .route("/~login", http::Method::GET, login_get)
            .route("/~login", http::Method::POST, login_post)
            .route("/~logout", http::Method::POST, logout)
            .route("/~search", http::Method::GET, search_get)
            .route("/~search", http::Method::POST, search_post)
            .route("/~new", http::Method::GET, new_page);
        let app = if let Some(dir) = opts.theme_dir.clone() {
            app.handler("/~theme", fs::StaticFiles::new(dir))
        } else {
            app.route(
                "/~theme/{name}",
                http::Method::GET,
                theme_get,
            )
        };
        app.default_resource(|r| {
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
