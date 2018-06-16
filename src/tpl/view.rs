use stpl::html::*;
use stpl::Render;

use super::misc::*;
use super::{base, misc};

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub base: base::Data,
    pub page_rendered: String,
    pub cur_url: String,
    pub narrowing_tags: ::data::NarrowingTagsSet,
}

pub fn page(data: &Data) -> impl Render {
    let content = (
        misc::breadcrumb(vec!["Home".into()]),
        misc::narrowing_tags_row(&data.cur_url, &data.narrowing_tags),
        row((col(data.page_rendered.clone()),)),
    );

    base::base(&data.base, Box::new(content))
}
