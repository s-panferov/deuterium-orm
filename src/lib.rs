#![license = "MIT"]
#![feature(plugin_registrar, quote)]
#![feature(tuple_indexing)]
#![feature(macro_rules)]

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
    ($model:ident, $table:ident, $table_name:expr, [ $(($field_name:ident, $field_type:ty)),+ ]) => (

        impl $table {
            pub fn new() -> $table {
                $table {
                    table_name_: $table_name,
                    table_alias_: None
                }
            }

            pub fn alias(alias: String) -> $table {
                $table {
                    table_name_: $table_name,
                    table_alias_: Some(alias)
                }
            }

            $(
                pub fn $field_name(&self) -> NamedField<$field_type> {
                    match &self.table_alias_ {
                        &Some(ref alias) => NamedField::<$field_type>::new_qual(stringify!($field_name), alias.as_slice()),
                        &None => NamedField::<$field_type>::new(stringify!($field_name))
                    }
                }
            )+    
        }

        impl ToOrmTable<$table> for $model {
            fn table() -> $table {
                $table::new()
            }

            fn alias(alias: String) -> $table {
                $table::alias(alias)
            }
        }
        
    )
}

pub trait ToOrmTable<T> {
    fn table() -> T;
    fn alias(alias: String) -> T;
}