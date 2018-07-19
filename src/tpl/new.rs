use super::base;
use super::misc::*;
use chrono;
use stpl::html::{button, div, raw, script};
use stpl::Render;

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub base: base::Data,
    pub cur_url: String,
}

pub fn page(data: &Data) -> impl Render {
    let content = (
        breadcrumb_from_tags(&["New".into()]),
        row((
            col_menu(()),
            col((div.id("edit_tab")((div.id("editor").class("my-2")((
                format!(
                    "Creation-Date: {}",
                    chrono::Local::now().format("%F %H-%M %Z")
                ),
                "\n\n# ",
            )),)),)),
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

    let buttons = button
        .id("save")
        .type_("submit")
        .class("btn btn-outline-primary mx-1")("Save");

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
const VIEW_JS: &str = include_str!("new.js");
