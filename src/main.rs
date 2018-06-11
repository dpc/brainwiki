#![allow(unused)]

#[macro_use]
extern crate lazy_static;
extern crate actix_web;
extern crate pulldown_cmark;
extern crate regex;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate failure;

use std::path::PathBuf;
mod markdown;
mod web;
mod opts;
mod fs;

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
    let opt = opts::from_args();
    markdown::parse_markdown(&"");

    web::start();
}
