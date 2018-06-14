use super::misc;
use super::misc::*;
use stpl::Render;
use stpl::html::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Flash {
    pub is_error: bool,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub title: String,
}


pub fn my_footer() -> impl Render {
    footer.id("footer").class("container py-1 my-1")((row(
        div.class("col text-center mx-1 px-4")(span((
            "© 2017-2018 Copyright: ",
            a.href(misc::url_base())("Hacker Audit"),
        ))),
    ),))
}

pub fn base(data: &Data, content: Box<Render + 'static>) -> impl Render {
    base_with_js(data, content, Box::new(()))
}

pub fn base_with_js(
    data: &Data,
    content: Box<Render + 'static>,
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
                meta.name("author").content("Hacker Audit team"),
                title(data.title.clone()),

                (
                    link.rel("icon").href("/~static/favicon.ico"),
                    link.rel("stylesheet").href("/~static/theme/flatly/bootstrap.min.css"),
                    link.rel("stylesheet").href("/~static/theme/custom.css"),
                )
            )),
            body(wrapper.class("d-flex flex-column")((
                flash_body,
                main
                    .id("main")
                    .role("main")
                    .class("container mb-5")(
                    content,
                ),
                my_footer(),
                (
                script.src("https://code.jquery.com/jquery-3.2.1.min.js").crossorigin("anonymous"),
                script.src("https://cdnjs.cloudflare.com/ajax/libs/popper.js/1.12.3/umd/popper.min.js")
                    .integrity("sha384-vFJXuSJphROIrBnz7yo7oB41mKfc8JzQZiCq4NCceLEaO4IHwicKwpJf9c9IpFgh")
                    .crossorigin("anonymous"),
                script.src("https://maxcdn.bootstrapcdn.com/bootstrap/4.0.0-beta.2/js/bootstrap.min.js")
                    .integrity("sha384-alpBpkh1PFOepccYVYDB4do5UnbKysX5WZXm3XxPqe5iKTfUKjNkCk9SaVuEZflJ")
                    .crossorigin("anonymous"),
                script.type_("text/javascript")(
                    raw(WHITE_ICONS_SCRIPT)
                ),
                flash_js,
                js,
                )
            )))
        ))
    )
}

const WHITE_ICONS_SCRIPT: &str = include_str!("white-icon.js");
