use stpl::html::*;
use stpl::Render;

use super::misc::*;
use super::{base, misc};
use std::collections::HashMap;

use data::Page;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Data {
    pub base: base::Data,
    pub cur_url: String,
    pub pages: Vec<Page>,
    pub narrowing_tags: ::data::NarrowingTagsSet,
}

pub fn page(data: &Data) -> impl Render {
    let content = (
        misc::breadcrumb(vec!["TODO".into()]),
        misc::narrowing_tags_row(&data.cur_url, &data.narrowing_tags),
        row(col((
            h2("Matches"),
            ul(data
                .pages
                .iter()
                .map(|page| li(a.href(page.url())(page.title.clone())))
                .collect::<Vec<_>>()),
        ))),
    );

    base::base(&data.base, Box::new(content))
}
