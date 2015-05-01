use syntax::{ast, codemap, parse};
use syntax::ptr::P;
use syntax::ext::base;

use super::Generator;

#[macro_use] mod macro_ext;
mod parser;
mod generator;

#[derive(Clone)]
pub struct ModelState {
    pub mod_name: ast::Ident,
    pub model: P<ast::Item>,
    pub primary_key: Option<Vec<String>>,
    pub before_create: Vec<String>,
    pub before_save: Vec<String>,
}

pub fn model<'cx>(cx: &'cx mut base::ExtCtxt, sp: codemap::Span,
                name: ast::Ident, tokens: Vec<ast::TokenTree>) -> Box<base::MacResult + 'cx> {

    // Parse a full ModelState from the input, emitting errors if used incorrectly.
    let state: ModelState = super::Parser::parse(&mut parse::tts_to_parser(cx.parse_sess(), tokens, cx.cfg()), (sp, &mut*cx, Some(name)));

    state.generate(sp, cx, ())
}
