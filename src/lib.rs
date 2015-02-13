#![feature(concat_idents)]
#![feature(plugin)]
#![feature(core)]
#![feature(path)]
#![feature(std_misc)]
#![feature(env)]
#![feature(io)]

#![plugin(regex_macros)]

#[cfg(feature = "postgres")]
extern crate postgres;
#[cfg(feature = "postgres")]
extern crate r2d2;
#[cfg(feature = "postgres")]
extern crate r2d2_postgres;
extern crate time;

extern crate regex;
extern crate deuterium;

#[cfg(feature = "postgres")]
pub use adapter::postgres::*;

#[macro_use] pub mod adapter;
#[macro_use] pub mod migration;


