use super::misc::*;
use crate::settings::SiteSettings;
use stpl::{html::*, Render};

use boolinator::Boolinator;

#[derive(Clone, Debug)]
pub struct Data<'a> {
    pub title: String,
    pub is_logged_in: bool,
    pub site_settings: &'a SiteSettings,
}

pub fn search_form(_data: &Data) -> impl Render {
    form.class("form-inline mx-1")
        .role("search")
        .id("search-form")
        .action("/~search")
        .method("post")(div.class("input-group")((
        input
            .id("search-query")
            .class("form-control")
            .placeholder("Tags...")
            .attr("type", "text")
            .name("q"),
        span.class("input-group-btn")(button
            .id("search-button")
            .type_("submit")
            .class("btn btn-outline-secondary")(
            (
            "Search",
        )
        )),
    )))
}

pub fn navbar(
    data: &Data,
    buttons: Box<dyn Render>,
) -> impl Render {
    (nav.id("main-navbar").class(
        "navbar navbar-expand-sm navbar-light bg-light fixed-top",
    )((div.class("container")((
        a.class("navbar-brand").href("/")(data.site_settings.short_name.to_owned()),
        button
            .class("navbar-toggler")
            .attr("type", "button")
            .data_toggle("collapse")
            .data_target("#navbar-collapse")
            .aria_controls("navbar-collapse")
            .aria_expanded("false")
            .aria_label("Toggle navigation")(span.class("navbar-toggler-icon")),
        div.id("navbar-collapse")
            .class("collapse navbar-collapse show")((
            ul.class("navbar-nav mr-auto")((li.class("nav-item dropdown")(if false {
                Some((
                    a.id("dropdown-top")
                        .class("nav-link dropdown-toggle mr-auto")
                        .href("/")
                        .data_toggle("dropdown")
                        .aria_haspopup("true")
                        .aria_expanded("false")("Top"),
                    div.class("dropdown-menu").aria_labelledby("dropdown01")((a
                        .class("dropdown-item")
                        .href("/")(
                        "Home"
                    ),)),
                ))
            } else {
                None
            }),)),
            data.is_logged_in.as_some(buttons),
            search_form(data),
            data.is_logged_in
                .as_some(form.action("/~logout").method("post")(button
                    .name("logout-button")
                    .id("logout-button")
                    .class("btn btn-outline-warning mx-1")
                    .value("logout")(
                    "Logout"
                ))),
            (!data.is_logged_in).as_some(a
                .id("login-button")
                .class("btn btn-outline-success mx-1")
                .href("/~login")("Login")),
        )),
    )),)),)
}

pub fn my_footer(site_settings: &SiteSettings) -> impl Render {
    footer.id("footer").class("container py-1 my-1")((row(div
        .class("col text-center mx-1 px-4")(
        span(site_settings.footer.clone()),
    )),))
}

#[allow(unused)]
pub fn base(
    data: &Data,
    content: Box<dyn Render>,
    buttons: Box<dyn Render>,
) -> impl Render {
    base_with_js(data, content, buttons, Box::new(()))
}

pub fn base_with_js(
    data: &Data,
    content: Box<dyn Render>,
    buttons: Box<dyn Render>,
    js: Box<Render + 'static>,
) -> impl Render {
    let (flash_body, flash_js) = flash(data);

    (
        doctype("html"),
        html((
            head((
                meta.charset("utf-8"),
                meta.name("viewport").content("width=device-width, initial-scale=1, shrink-to-fit=no"),
                meta.name("description").content(""),
                meta.name("author").content(data.site_settings.author.clone()),
                title(data.title.clone()),

                (
                    link.rel("icon").href("/~theme/favicon.ico"),
                    link.rel("stylesheet").href("/~theme/bootstrap.min.css"),
                    link.rel("stylesheet").href("/~theme/custom.css"),
                )
            )),
            body(wrapper.class("d-flex flex-column")((
                flash_body,
                navbar(data, buttons),
                main
                    .id("main")
                    .role("main")
                    .class("container mb-5")(
                    content,
                ),
                my_footer(&data.site_settings),
                (
                script.src("https://code.jquery.com/jquery-3.2.1.min.js").crossorigin("anonymous"),
                script.src("https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.12.3/umd/popper.min.js")
                    .integrity("sha384-vFJXuSJphROIrBnz7yo7oB41mKfc8JzQZiCq4NCceLEaO4IHwicKwpJf9c9IpFgh")
                    .crossorigin("anonymous"),
                script.src("https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0-beta.2/js/bootstrap.min.js")
                    .integrity("sha384-alpBpkh1PFOepccYVYDB4do5UnbKysX5WZXm3XxPqe5iKTfUKjNkCk9SaVuEZflJ")
                    .crossorigin("anonymous"),
                flash_js,
                js,
                )
            )))
        ))
    )
}
