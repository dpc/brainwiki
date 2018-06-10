use actix_web::middleware::session::{RequestSession};
use actix_web::{
    error, fs, middleware, pred, server, App, Error, HttpRequest, HttpResponse, Result, Responder
};
use actix_web::{http, Path };
use actix_web::dev::ResourceHandler;

fn index(_ : String) -> impl Responder {
    format!("Hello world")
}

fn login(_ : String) -> impl Responder {
    format!("Login - not implemented yet")
}



fn def(req: HttpRequest) -> impl Responder {
    format!("default: {}", req.path())
}

pub fn start() {

    server::new(
        || App::new()
            .route("/", http::Method::GET, index)
            .route("/~login", http::Method::GET, login)
            .handler("/~static", fs::StaticFiles::new("./static"))
            .default_resource(
                |r| r.get().f(def)
                             )
            )
        .bind("127.0.0.1:8080").unwrap()
        .run();
}
