/*pub mod home;
pub mod base;
pub mod unauthorized;
pub mod user;
pub mod search;
pub mod top;
pub mod security;
pub mod pkg;
pub mod pkgver;
pub mod review;
pub mod stats;
pub mod register;*/
pub mod data;
pub mod base;
pub mod misc;
pub mod view;
pub mod index;

use failure::{self, Fail};
use serde;
use std;
use std::path::{Path, PathBuf};
use stpl;
use stpl::{Template, TemplateExt};
use stpl::html;

macro_rules! def_tpl {
    // This macro takes an argument of designator `ident` and
    // creates a function named `$func_name`.
    // The `ident` designator is used for variable/function names.
    ($name:ident, $key:ident) => (

        pub fn $name() -> impl Template<Argument = ::tpl::$key::Data> {
            html::Template::new(stringify!($key), ::tpl::$key::page)
        }
    )
}

def_tpl!(view_tpl, view);
def_tpl!(index_tpl, index);


pub fn render<T: stpl::Template>(template: &T, data: &<T as Template>::Argument) -> Vec<u8>
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
