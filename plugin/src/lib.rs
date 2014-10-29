#![license = "MIT"]
#![feature(plugin_registrar, quote)]
#![feature(tuple_indexing)]
#![feature(macro_rules)]
#![feature(concat_idents)]

extern crate rustc;
extern crate syntax;

use rustc::plugin;
use syntax::parse::token;

use model::model;

mod generate;
mod define_model;
mod model;
mod parse;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut plugin::Registry) {
    reg.register_syntax_extension(token::intern("deuterium_model"), 
        syntax::ext::base::IdentTT(box model, None));
}

#[macro_export]
macro_rules! create_model(
    ($model:ident, $($field_name:ident: $field_value:expr),+) => (
        $model {
            $(
                $field_name: Some($field_value),
            )+

            ..std::default::Default::default()
        }
    )
)
