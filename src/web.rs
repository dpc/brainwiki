use actix_web::dev::ResourceHandler;
use actix_web::middleware::session::RequestSession;
use actix_web::{error, fs, middleware, pred, server, App, Error, HttpRequest, HttpResponse,
                Responder, Result};
use actix_web::{http, Path};
use std::sync;

use opts::Opts;
use {data::{self, Match, MatchType}};

use tpl;

fn index(_: String) -> impl Responder {
    format!("Hello world")
}

fn login(_: String) -> impl Responder {
    format!("Login - not implemented yet")
}

fn def(req: HttpRequest<State>) -> Result<HttpResponse, error::Error> {
    let tags = req.path().split("/").map(Into::into).collect();
    let data = req.state().data.read().unwrap();
    let match_ = data.find_best_match(tags) ;
    match match_.type_ {

        MatchType::One( page_id) => {
            let html = data.pages_by_id.get(&page_id).unwrap().rendered.clone();
            let body = tpl::render(&tpl::view_tpl(), &tpl::view::Data {
                base: tpl::base::Data {
                    title: "TITLE_TBD".into(),
                },
                page_rendered: html,
            });
            Ok(HttpResponse::Ok().body(body))
        }
        MatchType::Many(page_ids) => {
            Ok(HttpResponse::Ok().body(format!("Results: {}", page_ids.len())))
        }
        MatchType::None => {
            Ok(HttpResponse::Ok().body(format!("Not Found :(")))
        }
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
