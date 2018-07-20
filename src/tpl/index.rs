use stpl::{html::*, Render};

use super::{
    base,
    misc::{self, *},
};

use crate::{data, page::Page};

#[derive(Clone, Debug)]
pub struct Data<'a> {
    pub base: base::Data<'a>,
    pub cur_url: String,
    pub pages: Vec<Page>,
    pub narrowing_tags: data::NarrowingTagsSet,
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
                    .map(|page| {
                        li(a.href(page.url())(
                            page.title.clone(),
                        ))
                    })
                    .collect::<Vec<_>>()),
            )),
        )),
    );

    let buttons = a
        .id("new")
        .class("btn btn-outline-primary mx-1")
        .href("/~new")("New");

    base::base_with_js(
        &data.base,
        Box::new(content),
        Box::new(buttons),
        Box::new(()),
    )
}
