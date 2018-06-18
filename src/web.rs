use actix_web::http;
use actix_web::{
    error, fs, server, App, HttpRequest, HttpResponse,
    Responder, Result,
};

use data::{self, MatchType};
use opts::Opts;

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
fn def(
    req: HttpRequest<State>,
) -> Result<HttpResponse, error::Error> {
    let cur_url = req.path();
    let mut tags: Vec<String> =
        cur_url.split("/").skip(1).map(Into::into).collect();
    let data = req.state().data.read();

    let prefer_exact = if tags.last() == Some(&"".into()) {
        tags.pop();
        false
    } else {
        true
    };

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
        MatchType::Many(page_ids) => {
            let body = tpl::render(
                &tpl::index_tpl(),
                &tpl::index::Data {
                    base: tpl::base::Data {
                        title: if match_.matching_tags.is_empty()
                        {
                            ::config::WIKI_NAME_TEXT.into()
                        } else {
                            match_.matching_tags.join("/")
                        },
                    },
                    pages: page_ids
                        .iter()
                        .map(|page_id| {
                            data.pages_by_id
                                .get(&page_id)
                                .unwrap()
                                .clone()
                        })
                        .collect(),
                    cur_url: cur_url.into(),
                    narrowing_tags: match_.narrowing_tags,
                    matching_tags: match_.matching_tags,
                },
            );

            Ok(HttpResponse::Ok().body(body))
        }
        MatchType::None => {
            Ok(HttpResponse::Ok().body(format!("Not Found :(")))
        }
    }
}

#[derive(Clone)]
struct State {
    data: ::data::SyncState,
    opts: Opts,
}

pub fn start(data: data::SyncState, opts: Opts) {
    let state = State {
        data: data,
        opts: opts.clone(),
    };
    server::new(move || {
        App::with_state(state.clone())
            //.route("/", http::Method::GET, index)
            .route("/~login", http::Method::GET, login)
            .handler("/~theme", fs::StaticFiles::new(opts.theme_dir.clone()))
            .default_resource(|r| r.get().f(def))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .run();
}
