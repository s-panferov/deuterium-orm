#![license = "MIT"]
#![feature(plugin_registrar, quote)]
#![feature(tuple_indexing)]
#![feature(macro_rules)]
#![feature(concat_idents)]

#[cfg(feature = "postgres")]
extern crate postgres;
#[cfg(feature = "postgres")]
extern crate r2d2;
#[cfg(feature = "postgres")]
extern crate r2d2_postgres;

extern crate deuterium;

pub mod adapter;

