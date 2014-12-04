#![feature(plugin_registrar, quote)]
#![feature(tuple_indexing)]
#![feature(macro_rules)]
#![feature(concat_idents)]
#![feature(globs)]
#![feature(phase)]

#[cfg(feature = "postgres")]
extern crate postgres;
#[cfg(feature = "postgres")]
extern crate r2d2;
#[cfg(feature = "postgres")]
extern crate r2d2_postgres;
extern crate time;

extern crate rustc;
extern crate syntax;

#[phase(plugin)]
extern crate regex_macros;
extern crate regex;

extern crate deuterium;

#[cfg(feature = "postgres")]
pub use adapter::postgres::*;

pub mod adapter;
pub mod migration;
pub mod plugin;


