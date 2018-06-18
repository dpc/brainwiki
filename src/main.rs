#![feature(nll)]

#[macro_use]
extern crate lazy_static;
extern crate actix_web;
extern crate pulldown_cmark;
extern crate regex;
#[macro_use]
extern crate structopt;
//#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate notify;
extern crate serde_json;
extern crate stpl;

mod config;
mod data;
mod markdown;
mod opts;
mod page;
mod tpl;
mod web;

type Result<T> = std::result::Result<T, failure::Error>;

// GET /a/b/c - search for a post with a/b/c tag
//   a is most important, c least important
//   if unique match, it's page id - respond with id
//   if not unique, respond with list of tags and posts to qualify
//   if no matches, remove c, try again
//
// POST / - create a new page
//    if the page with the same tags exists, return error
// PUT /a/b/c - update page
//    if the page with the same tags exists, return error
//
// DELETE /id - delete page
//
// POST /~login login
// ANY /~... other special stuff

fn main() {
    let opts = opts::from_args();
    markdown::parse_markdown("");

    let state = data::SyncState::new();

    let _watcher = data::FsWatcher::new(
        opts.data_dir.clone(),
        state.clone(),
    );

    state.write().insert_from_dir(&opts.data_dir).unwrap();

    web::start(state, opts);
}
