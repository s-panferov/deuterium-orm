#![feature(phase)]
#![feature(globs)]

#[phase(plugin)]
extern crate deuterium_orm;
extern crate deuterium_orm;
extern crate time;
extern crate deuterium;

use deuterium::*;

use std::default::Default;
use time::Timespec;

deuterium_model! jedi {
    pub struct Jedi {
        id: String,
        name: String,
        force_level: u8,
        side: bool,
        created_at: Timespec,
        updated_at: Timespec
    }
}

impl Jedi {
    pub fn new() -> uint {
        1u
    }
}

#[test]
fn test() {
    let table = JediTable::name();
}