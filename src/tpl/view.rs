use stpl::html::{div, raw, script};
use stpl::Render;

use super::misc::*;
use super::{base, misc};
use page::Page;

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub base: base::Data,
    pub page: Page,
    pub cur_url: String,
    pub narrowing_tags: ::data::NarrowingTagsSet,
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
            col(data.page.html.clone()),
        )),
        row(div.id("editor")(data.page.md.clone())),
    );

    fn ace_script(f: &str) -> impl Render {
        script
            .src(format!("https://cdnjs.cloudflare.com/ajax/libs/ace/1.3.3/{}", f))
            .type_("text/javascript")
            .charset("utf-8")
    }

    let js = (
        ace_script("ace.js"),
        ace_script("keybinding-vim.js"),
        ace_script("mode-markdown.js"),
        script.type_("text/javascript")(raw(ACE_JS)),
    );
    base::base_with_js(
        &data.base,
        Box::new(content),
        Box::new(js),
    )
}

const ACE_JS: &str = include_str!("ace.js");
