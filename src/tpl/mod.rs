pub mod base;
pub mod index;
pub mod misc;
pub mod view;
pub mod new;

use serde;
use std;
use std::path::{Path, PathBuf};
use stpl;
use stpl::html;
use stpl::Template;

macro_rules! def_tpl {
    ($name:ident, $key:ident) => {
        pub fn $name(
        ) -> impl Template<Argument = crate::tpl::$key::Data> {
            html::Template::new(
                stringify!($key),
                crate::tpl::$key::page,
            )
        }
    };
}

def_tpl!(view_tpl, view);
def_tpl!(index_tpl, index);
def_tpl!(new_tpl, new);

pub fn render<T: stpl::Template>(
    template: &T,
    data: &<T as Template>::Argument,
) -> Vec<u8>
where
    <T as Template>::Argument: serde::Serialize + 'static,
{
    let path = std::env::args_os().next().unwrap();
    let path: &Path = path.as_ref();
    let mut path: PathBuf = path.to_path_buf();
    path.set_file_name("template");

    let mut out = vec![];

    template.render(data, &mut out).unwrap();

    out
}
