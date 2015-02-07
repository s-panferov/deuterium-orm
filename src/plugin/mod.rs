use rustc::plugin;
use syntax::parse::token;
use syntax;

use self::model::model;
use self::model::migration;

mod generate;
mod define_model;
mod model;
mod parse;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut plugin::Registry) {
    reg.register_syntax_extension(token::intern("deuterium_model"), 
        syntax::ext::base::IdentTT(Box::new(model), None));

    reg.register_syntax_extension(token::intern("load_migrations"), 
        syntax::ext::base::NormalTT(Box::new(migration), None));
}

#[macro_export]
macro_rules! create_model {
    ($model:ident, $($field_name:ident: $field_value:expr),+) => (
        $model {
            $(
                $field_name: Some($field_value),
            )+

            ..std::default::Default::default()
        }
    )
}