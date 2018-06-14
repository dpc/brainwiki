#![feature(try_from)]
#![allow(unused)]
#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate stpl;

#[macro_use]
extern crate failure;

mod tpl;

fn main() {
    stpl::handle_dynamic()
        .template(&tpl::view_tpl()) ;
}
