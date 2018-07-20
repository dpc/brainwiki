//! BrainWiki is a wiki where everything is addressed using tags. This allows
//! organization without any premeditated structure.
//!
//! Created for personal use, to help me gather and view my messy md
//! notes.
//!
//! Eg.
//!
//! ```markdown
//! [My idea about brainwiki](/idea/brainwiki)
//! ```
//!
//! will link to any/all pages that contain #idea and #brainwiki tags,
//! potentially broadening the search to the first best match.
//!
//! Goals:
//!
//! * minimalism and simplicity
//! * easy deployment
//! * low resource consumption
//!
//! Current status: usable in it's basic form.
//!
//! It supports:
//!
//! * markdown
//! * watching for FS changes
//!
//! In plans:
//!
//! * ACE editor integration
//!
//! UI based on Bootstrap. Written in Rust using actix-web.
//!
//! ### Using
//!
//! Clone, `cargo build`. Run the binary with `--data <dir-with-md-files>`

#![feature(rust_2018_preview, use_extern_macros)]
#![feature(nll)]

extern crate actix_web;
extern crate chrono;
extern crate lazy_static;
extern crate listenfd;
extern crate pulldown_cmark;
extern crate regex;
extern crate structopt;
#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bytes;
extern crate futures;
extern crate notify;
extern crate serde_json;
extern crate stpl;
#[macro_use]
extern crate derive_more;
extern crate boolinator;
#[macro_use]
extern crate quicli;

mod data;
mod markdown;
mod opts;
mod page;
mod settings;
mod tpl;
mod web;

use structopt::StructOpt;

type Result<T> = std::result::Result<T, failure::Error>;

//main!(|args: opts::Opts, log_level: verbosity| {
main!(|opts: opts::Opts| {
    let settings = settings::SiteSettings::load_or_create_in(
        &opts.data_dir,
    )?;

    let state = data::SyncState::new();

    let _watcher = data::FsWatcher::new(
        opts.data_dir.clone(),
        state.clone(),
    );

    state.write().insert_from_dir(&opts.data_dir).unwrap();

    web::start(state, opts);
});
