use actix_web::dev::ResourceHandler;
use actix_web::middleware::session::RequestSession;
use actix_web::{
    error, fs, middleware, pred, server, App, Error, HttpRequest, HttpResponse, Responder, Result,
};
use actix_web::{http, Path};
use std::sync;

use data::{self, Match, MatchType};
use opts::Opts;

use tpl;

fn index(_: String) -> impl Responder {
    format!("Hello world")
}

fn login(_: String) -> impl Responder {
    format!("Login - not implemented yet")
}

fn def(req: HttpRequest<State>) -> Result<HttpResponse, error::Error> {
    let cur_url = req.path();
    let mut tags: Vec<String> = cur_url.split("/").skip(1).map(Into::into).collect();
    let data = req.state().data.read().unwrap();

    let prefer_exact = if tags.last() == Some(&"".into()) {
        tags.pop();
        false
    } else {
        true
    };

    let match_ = data.find_best_match(tags, prefer_exact);

    if !match_.unmatched_tags.is_empty() {
        let mut location = String::from("/") + match_.matching_tags.join("/").as_str();
        if !prefer_exact {
            location += "/";
        }
        return Ok(HttpResponse::TemporaryRedirect()
            .header("Location", location)
            .finish());
    }

    match match_.type_ {
        MatchType::One(page_id) => {
            let html = data.pages_by_id.get(&page_id).unwrap().rendered.clone();
            let body = tpl::render(
                &tpl::view_tpl(),
                &tpl::view::Data {
                    base: tpl::base::Data {
                        title: "TITLE_TBD".into(),
                    },
                    page_rendered: html,
                    cur_url: cur_url.into(),
                    narrowing_tags: match_.narrowing_tags,
                },
            );
            Ok(HttpResponse::Ok().body(body))
        }
        MatchType::Many(page_ids) => {
            let body = tpl::render(
                &tpl::index_tpl(),
                &tpl::index::Data {
                    base: tpl::base::Data {
                        title: "TITLE_TBD".into(),
                    },
                    pages: page_ids
                        .iter()
                        .map(|page_id| data.pages_by_id.get(&page_id).unwrap().clone())
                        .collect(),
                    cur_url: cur_url.into(),
                    narrowing_tags: match_.narrowing_tags,
                },
            );

            Ok(HttpResponse::Ok().body(body))
        }
        MatchType::None => Ok(HttpResponse::Ok().body(format!("Not Found :("))),
    }
}

#[derive(Clone)]
struct State {
    data: sync::Arc<sync::RwLock<data::State>>,
    opts: Opts,
}

pub fn start(data: data::State, opts: Opts) {
    let state = State {
        data: sync::Arc::new(sync::RwLock::new(data)),
        opts,
    };
    server::new(move || {
        App::with_state(state.clone())
            //.route("/", http::Method::GET, index)
            .route("/~login", http::Method::GET, login)
            .handler("/~static", fs::StaticFiles::new("./static"))
            .default_resource(|r| r.get().f(def))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .run();
}
