use stpl::{
    html::{a, button, div, raw, script},
    Render,
};

use super::{
    base,
    misc::{self, *},
};
use crate::{data, page::Page};

#[derive(Clone)]
pub struct Data<'a> {
    pub base: base::Data<'a>,
    pub page: Page,
    pub cur_url: String,
    pub narrowing_tags: data::NarrowingTagsSet,
}

pub fn page(data: &Data) -> impl Render {
    let content = (
        breadcrumb_from_tags(&data.page.tags.as_slice()),
        row((
            col_menu((
                misc::narrowing_tags_col(
                    &data.cur_url,
                    &data.narrowing_tags,
                ),
                misc::broadening_tags_col(
                    &data.cur_url,
                    data.page.tags.clone(),
                ),
            )),
            col((
                div.id("view_tab")((data.page.html.clone(),)),
                div.id("edit_tab")
                    .attr("style", "display: none;")(
                    (
                    div.id("editor").class("my-2")(
                        data.page.md.clone(),
                    ),
                )
                ),
            )),
        )),
    );

    fn ace_script(f: &str) -> impl Render {
        script
            .src(format!(
                "https://cdnjs.cloudflare.com/ajax/libs/ace/1.3.3/{}",
                f
            ))
            .type_("text/javascript")
            .charset("utf-8")
    }

    let buttons = (
        a.id("new")
            .class("btn btn-outline-primary mx-1")
            .href("/~new")("New"),
        button
            .id("edit")
            .type_("submit")
            .class("btn btn-outline-primary mx-1")(
            "Edit"
        ),
        button
            .id("save")
            .attr("style", "display: none;")
            .type_("submit")
            .class("btn btn-outline-primary mx-1")(
            "Save"
        ),
    );

    let js = (
        ace_script("ace.js"),
        ace_script("keybinding-vim.js"),
        ace_script("mode-markdown.js"),
        script.type_("text/javascript")(raw(VIEW_JS)),
    );

    base::base_with_js(
        &data.base,
        Box::new(content),
        Box::new(buttons),
        Box::new(js),
    )
}
const VIEW_JS: &str = include_str!("view.js");
