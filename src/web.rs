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

use actix_web::http;
use actix_web::middleware::{Finished, Middleware, Started};
use actix_web::{error, fs, server, App, HttpRequest, HttpResponse, Responder, Result};
use actix_web::{AsyncResponder, HttpMessage};

use data::{self, MatchType, PageId};
use opts::Opts;

use futures::Future;
use std::path::Path;

use tpl;

fn index(_: String) -> impl Responder {
    format!("Hello world")
}

fn login(_: String) -> impl Responder {
    format!("Login - not implemented yet")
}

fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::TemporaryRedirect()
        .header("Location", location)
        .finish()
}

fn post(req: HttpRequest<State>) -> Result<HttpResponse, error::Error> {
    let cur_url = req.path();
    let (_tags, _prefer_exact) = url_to_tags(cur_url);
    println!("WOOFHOO");
    Ok(HttpResponse::Ok().body("{}"))
}

#[derive(Debug, Serialize, Deserialize)]
struct PutInput {
    text: String,
}

fn put(req: HttpRequest<State>) -> Box<Future<Item = HttpResponse, Error = error::Error>> {
    let cur_url = req.path().to_owned();
    let data = req.state().data.clone();
    req.json()
        .map_err(|e|{
                 println!("{}", e);
                 e})
        .from_err()  // convert all errors into `Error`
        .and_then(move |input: PutInput| {

            let (url_tags, _) = url_to_tags(cur_url.as_str());
            let data_read = data.read();
            let page_id = data_read.lookup(url_tags)?;

            let new_page = ::page::Page::from_markdown(input.text.clone(), Path::new("/"))?;

            let match_ = data_read.find_best_match(new_page.tags.clone(), true);

            match match_.type_ {
                MatchType::Many(_) => {
                    return Ok(HttpResponse::Conflict().body("{}"))
                },
                MatchType::One(id) => {
                    if id != page_id {
                        return Ok(HttpResponse::Conflict().body("{}"))
                    }
                }
                MatchType::None => {}
            }

            Ok(HttpResponse::Ok().json(()))
        })
        .map_err(|e|{
                 println!("2: {}", e);
                 e})
    .responder()
}

fn get_index(
    match_: &::data::Match,
    cur_url: &str,
    page_ids: &[PageId],
    data: &::data::State,
) -> Result<HttpResponse, error::Error> {
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
                    ::config::WIKI_NAME_TEXT.into()
                } else {
                    match_.matching_tags.join("/")
                },
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

//    Ok(HttpResponse::c().body("{}"))
fn get(req: HttpRequest<State>) -> Result<HttpResponse, error::Error> {
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
    data: ::data::SyncState,
    opts: Opts,
}

struct Logger;

impl<S> Middleware<S> for Logger {
    fn start(&self, req: &mut HttpRequest<S>) -> Result<Started> {
        println!("{} {}", req.method(), req.path());
        Ok(Started::Done)
    }

    fn finish(&self, req: &mut HttpRequest<S>, resp: &HttpResponse) -> Finished {
        Finished::Done
    }
}

pub fn start(data: data::SyncState, opts: Opts) {
    let state = State {
        data: data,
        opts: opts.clone(),
    };

    server::new(move || {
        App::with_state(state.clone())
            .middleware(Logger)
            //.route("/", http::Method::GET, index)
            .route("/~login", http::Method::GET, login)
            .handler("/~theme", fs::StaticFiles::new(opts.theme_dir.clone()))
            .default_resource(|r| {

                               r.get().f(get);
                               r.post().f(post);
                               r.put().f(put);
            })
    }).bind("127.0.0.1:8080")
        .unwrap()
        .run();
}
