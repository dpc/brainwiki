use super::base::Data;
use std::env;
use stpl::html::*;
use stpl::Render;

pub fn flash(data: &Data) -> (impl Render, impl Render) {
    (
        /*
        div.class("container")(data.flash.as_ref().map(|flash| {
            div.id("flash").class(format!(
                "alert alert-{} mt-2",
                if flash.is_error { "danger" } else { "info" }
            ))(flash.msg.clone())
        })),
        */
        (),
        script.type_("text/javascript")(raw(FLASH_SCRIPT)),
    )
}

pub fn img_svg_icon_src(name: &str) -> String {
    format!("/static/theme/Entypo+/Entypo+/{}.svg", name)
}

pub fn img_svg_icon(name: &'static str, white: bool) -> impl Render {
    img.class(if white {
        "svg icon icon-white"
    } else {
        "svg icon"
    }).src(img_svg_icon_src(name))
        .alt(name)
}

pub fn img_svg_miniicon(name: &'static str, white: bool) -> impl Render {
    img.class(if white {
        "svg icon mini-icon icon-white"
    } else {
        "svg mini-icom icon"
    }).src(img_svg_icon_src(name))
        .alt(name)
}

pub fn img_svg_icon_social_src(name: &str) -> String {
    format!(
        "/static/theme/Entypo+/Entypo+ Social Extension/{}.svg",
        name
    )
}

pub fn img_svg_icon_social(name: &'static str, white: bool) -> impl Render {
    img.class(if white {
        "svg icon icon-white"
    } else {
        "svg icon"
    }).src(img_svg_icon_social_src(name))
        .alt(name)
}

pub struct BreadCrumbItem(Box<Render>);

impl<T: Render + 'static> From<T> for BreadCrumbItem {
    fn from(t: T) -> BreadCrumbItem {
        BreadCrumbItem(Box::new(t))
    }
}

pub fn breadcrumb(mut names: Vec<BreadCrumbItem>) -> impl Render {
    let names_len = names.len();

    nav.aria_label("breadcrumb").role("navigation")(ol.class("breadcrumb")(
        (names
            .drain(..)
            .enumerate()
            .map(|(n, render)| Box::new(li.class("breadcrumb-item")(render.0)) as Box<Render>)
            .collect::<Vec<Box<Render>>>()),
    ))
}

pub fn url_base() -> String {
    env::var("BASE_URL").unwrap_or_else(|_| "/".into())
}

pub fn narrowing_tags_col(cur_url: &str, narrowing_tags: &::data::NarrowingTagsSet) -> impl Render {
    col_menu(if !narrowing_tags.is_empty() {
        let mut list: Vec<(_, _)> = narrowing_tags.iter().collect();
        list.sort_by(|n, m| n.0.cmp(m.0));
        Some((
            h2("Tags"),
            p(list
                .iter()
                .map(|(tag, nums)| {
                    (
                        a.href(url_append(cur_url, tag))((
                            format!("#{}", tag),
                            nbsp,
                            format!("({})", nums),
                        )),
                        " ",
                    )
                })
                .collect::<Vec<_>>()),
        ))
    } else {
        None
    })
}

pub fn url_append(base: &str, tag: &str) -> String {
    if base.as_bytes().last().cloned() == Some('/' as u8) {
        format!("{}{}/", base, tag)
    } else {
        format!("{}/{}", base, tag)
    }
}

pub fn breadcrumb_from_tags(tags: &[String]) -> Box<Render> {
    if tags.is_empty() {
        Box::new(breadcrumb(vec!["Home".into()]))
    } else {
        Box::new(breadcrumb(
            tags.iter()
                .map(|tag| BreadCrumbItem::from(a.href(url_append("/", tag.as_str()))(tag.clone())))
                .collect(),
        ))
    }
}

pub fn col<C: Render + 'static>(content: C) -> impl Render {
    div.class("col-9 px-4")(content)
}
pub fn col_menu<C: Render + 'static>(content: C) -> impl Render {
    div.class("col-3 px-4")(content)
}

pub fn row<C: Render + 'static>(content: C) -> impl Render {
    div.class("row")(content)
}
const FLASH_SCRIPT: &str = include_str!("flash.js");
