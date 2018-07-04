use stpl::html::*;
use stpl::Render;

use super::misc::*;
use super::{base, misc};

use page::Page;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Data {
    pub base: base::Data,
    pub cur_url: String,
    pub pages: Vec<Page>,
    pub narrowing_tags: ::data::NarrowingTagsSet,
    pub matching_tags: Vec<String>,
}

pub fn page(data: &Data) -> impl Render {
    let content = (
        breadcrumb_from_tags(&data.matching_tags.as_slice()),
        row((
            col_menu(misc::narrowing_tags_col(
                &data.cur_url,
                &data.narrowing_tags,
            )),
            col((
                h2("Matching Pages"),
                ul(data
                    .pages
                    .iter()
                    .map(|page| li(a.href(page.url())(page.title.clone())))
                    .collect::<Vec<_>>()),
            )),
        )),
    );

    let buttons = (a.id("new").class("btn btn-info mx-1").href("/~new")("New"),);
    base::base(&data.base, Box::new(content), Box::new(buttons))
}
