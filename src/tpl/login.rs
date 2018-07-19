use super::base;
use super::misc::*;
use stpl::html::{button, div, form, input, span};
use stpl::Render;

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub base: base::Data,
    pub cur_url: String,
}

pub fn page(data: &Data) -> impl Render {
    let content = (
        breadcrumb_from_tags(&["Login".into()]),
        row((
            col_menu(()),
            col(form
                .class("form-inline mx-1")
                .role("login")
                .id("search-form")
                .action("/~login")
                .method("post")(div.class("input-group")((
                input
                    .id("login-password")
                    .class("form-control")
                    .placeholder("Password...")
                    .attr("type", "password")
                    .name("password"),
                span.class("input-group-btn")(button
                    .id("login-submit-button")
                    .type_("submit")
                    .class("btn btn-info")((
                    "Login",
                ))),
            )))),
        )),
    );

    let buttons = ();

    let js = ();

    base::base_with_js(
        &data.base,
        Box::new(content),
        Box::new(buttons),
        Box::new(js),
    )
}
