use stpl::html::*;
use stpl::Render;

use super::misc::*;
use super::{base, misc};
use data::Page;

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub base: base::Data,
    pub page: Page,
    pub cur_url: String,
    pub narrowing_tags: ::data::NarrowingTagsSet,
}

pub fn page(data: &Data) -> impl Render {
    let content = (
        misc::breadcrumb(
            data.page
                .tags
                .iter()
                .map(|tag| {
                    misc::BreadCrumbItem::from(a.href(misc::url_append("/", tag.as_str()))(
                        tag.clone(),
                    ))
                })
                .collect(),
        ),
        row((
            misc::narrowing_tags_col(&data.cur_url, &data.narrowing_tags),
            col(data.page.rendered.clone()),
        )),
    );

    base::base(&data.base, Box::new(content))
}
