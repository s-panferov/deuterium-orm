#![license = "MIT"]
#![feature(plugin_registrar, quote)]
#![feature(tuple_indexing)]
#![feature(macro_rules)]
#![feature(concat_idents)]
#![feature(globs)]

#[cfg(feature = "postgres")]
extern crate postgres;
#[cfg(feature = "postgres")]
extern crate r2d2;
#[cfg(feature = "postgres")]
extern crate r2d2_postgres;
extern crate time;

extern crate deuterium;

#[cfg(feature = "postgres")]
pub use adapter::postgres::*;

pub mod adapter;
pub mod migration;


