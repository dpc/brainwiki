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
    );

    base::base(&data.base, Box::new(content))
}
