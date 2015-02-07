#![feature(plugin_registrar, quote)]
#![feature(concat_idents)]
#![feature(plugin)]
#![feature(rustc_private)]
#![feature(core)]
#![feature(path)]
#![feature(std_misc)]
#![feature(env)]
#![feature(io)]

#[cfg(feature = "postgres")]
extern crate postgres;
#[cfg(feature = "postgres")]
extern crate r2d2;
#[cfg(feature = "postgres")]
extern crate r2d2_postgres;
extern crate time;

extern crate rustc;
extern crate syntax;

#[plugin]
extern crate regex_macros;
extern crate regex;

extern crate deuterium;

#[cfg(feature = "postgres")]
pub use adapter::postgres::*;

pub mod adapter;
pub mod migration;
pub mod plugin;


