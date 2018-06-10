#![allow(unused)]

#[macro_use]
extern crate lazy_static;
extern crate pulldown_cmark;
extern crate regex;
extern crate actix_web;

use std::collections::{HashMap, HashSet};

use std::path::PathBuf;
mod markdown;
mod web;

type PageId = u32;

struct Page {
    rendered: String,
    path: PathBuf,
}

struct State {
    pages_by_id: HashMap<PageId, Page>,
    tag_sets: HashMap<String, HashSet<PageId>>,

}


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
    markdown::parse_markdown(&"");

    web::start();

}
