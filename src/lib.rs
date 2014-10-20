#![license = "MIT"]
#![feature(plugin_registrar, quote)]
#![feature(tuple_indexing)]

extern crate syntax;
extern crate rustc;

use rustc::plugin;
use syntax::parse::token;

use model::model;

mod model;
mod parse;
mod generate;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut plugin::Registry) {
    reg.register_syntax_extension(token::intern("deuterium_model"), 
        syntax::ext::base::IdentTT(box model, None));
}

// pub trait OrmTable {
//     fn new() -> OrmTable;
//     fn new_with_alias(String) -> OrmTable;
// }

// pub trait ToOrmTable<T: OrmTable> {
//     fn table() -> T {
//         T::new()
//     }

//     fn alias(alias: String) -> T {
//         T::new_with_alias(alias)
//     }
// }