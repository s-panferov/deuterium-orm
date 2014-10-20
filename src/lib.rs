#![license = "MIT"]
#![feature(plugin_registrar, quote)]
#![feature(tuple_indexing)]
#![feature(macro_rules)]
#![feature(concat_idents)]

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

#[macro_export]
macro_rules! define_model {
    ($model:ident, $table:ident, $table_inst:ident, $table_name:expr, [ $(($field_name:ident, $field_type:ty)),+ ]) => (

        struct $table;

        struct $table_inst {
            name: String,
            alias: Option<String>
        }

        impl $table {

            pub fn table_name() -> &'static str {
                $table_name
            }

            pub fn alias(alias: &str) -> $table_inst {
                $table_inst {
                    name: $table::table_name().to_string(),
                    alias: Some(alias.to_string())
                }
            }

            $(
                pub fn $field_name() -> NamedField<$field_type> {
                    NamedField::<$field_type>::new(stringify!($field_name))
                }
            )+   
        }

        impl $table_inst {
            $(
                pub fn $field_name(&self) -> NamedField<$field_type> {
                    match self.alias.as_ref() {
                        Some(alias) => NamedField::<$field_type>::new_qual(self.name.as_slice(), alias.as_slice()),
                        None => NamedField::<$field_type>::new(self.name.as_slice())
                    }
                }
            )+  
        }
    )
}