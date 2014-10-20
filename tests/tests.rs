#![feature(phase)]
#![feature(globs)]

#[phase(plugin)]
extern crate deuterium_orm;
extern crate time;
extern crate deuterium;

use deuterium::*;

use std::default::Default;
use time::Timespec;

#[test]
fn test() {

    deuterium_model! jedi {
        struct Jedi {
            id: String,
            name: String,
            force_level: u8,
            side: bool,
            created_at: Timespec,
            updated_at: Timespec
        }
    }

    let table = JediTable;
}