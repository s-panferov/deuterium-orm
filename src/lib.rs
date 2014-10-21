#![license = "MIT"]
#![feature(plugin_registrar, quote)]
#![feature(tuple_indexing)]
#![feature(macro_rules)]
#![feature(concat_idents)]

extern crate syntax;
extern crate rustc;

use rustc::plugin;
use syntax::parse::token;

use std::default::Default;

use dt_plugin::model;

mod dt_plugin;
mod define_model_macro;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut plugin::Registry) {
    reg.register_syntax_extension(token::intern("deuterium_model"), 
        syntax::ext::base::IdentTT(box model, None));
}

pub enum FieldValue<T> {
    Unknown,
    Known(T)
}

impl<T> Default for FieldValue<T> {
    fn default () -> FieldValue<T> {
        Unknown
    }
}

#[macro_export]
macro_rules! create_model(
    ($model:ident, $($field_name:ident: $field_value:expr),+) => (
        $model {
            $(
                $field_name: deuterium_orm::Known($field_value),
            )+

            ..std::default::Default::default()
        }
    )
)

