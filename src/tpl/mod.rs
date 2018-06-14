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
pub mod view;
pub mod misc;

use failure::{self, Fail};
use serde;
use std;
use std::path::{Path, PathBuf};
use stpl;
use stpl::{Template, TemplateExt};
use stpl::html;

pub fn view_tpl() -> impl Template<Argument = ::tpl::view::Data> {
    html::Template::new("view", ::tpl::view::page)
}

pub fn render<T: stpl::Template>(template: &T, data: &<T as Template>::Argument) -> Vec<u8>
where
    <T as Template>::Argument: serde::Serialize + 'static,
{
    let path = std::env::args_os().next().unwrap();
    let path: &Path = path.as_ref();
    let mut path: PathBuf = path.to_path_buf();
    path.set_file_name("template");

    template.render_dynamic(&path, data).unwrap_or_else(|e| {
        eprintln!("Rendering template {} failed: {}", template.key(), e);
        let b = failure::Backtrace::new();
        let stdout = e.stdout().unwrap_or_else(|| &[]);
        let stderr = e.stderr ().unwrap_or_else(|| &[]);
        eprintln!(
            "stdout: {}\nstderr: {}\nbacktrace:{}\n",
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&stderr),
            e.backtrace().unwrap_or_else(|| &b),
            );
        "Internal error".into()
    })
}
