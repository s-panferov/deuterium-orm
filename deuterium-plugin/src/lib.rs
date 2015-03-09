#![feature(plugin)]
#![feature(plugin_registrar)]
#![feature(rustc_private)]
#![feature(path)]
#![feature(core)]
#![plugin(regex_macros)]

extern crate syntax;
extern crate rustc;
extern crate regex;

use rustc::plugin;
use syntax::parse::token;

use syntax::{codemap};
use syntax::ext::base;
use syntax::parse::parser;

#[macro_use] mod helpers;
#[macro_use] mod model;
#[macro_use] mod migration;

#[plugin_registrar]
#[doc(hidden)]
pub fn plugin_registrar(reg: &mut plugin::Registry) {
    reg.register_syntax_extension(token::intern("deuterium_model"),
        syntax::ext::base::IdentTT(Box::new(model::model), None, false));

    reg.register_syntax_extension(token::intern("load_migrations"),
        syntax::ext::base::NormalTT(Box::new(migration::migration), None, false));
}

pub trait Parser<Cfg> {
    fn parse(&mut parser::Parser, Cfg) -> Self;
}

pub trait Generator<Cfg> {
    fn generate<'a>(self, codemap::Span, &mut base::ExtCtxt, Cfg) -> Box<base::MacResult + 'a>;
}
