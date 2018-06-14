use stpl::Render;
use stpl::html::*;

use super::{base, misc};
use super::misc::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub base: base::Data,
    pub page_rendered: String,
}

pub fn page(data: &Data) -> impl Render {
    let content = (
        misc::breadcrumb(vec!["Home".into()]),
        row((
            col(data.page_rendered.clone()),
       ))
    );

    base::base(&data.base, Box::new(content))
}
