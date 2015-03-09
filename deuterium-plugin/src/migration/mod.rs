use syntax::{ast, codemap, parse};
use syntax::ext::base;
use std::path;

use super::Generator;

#[macro_use] mod macro_ext;
mod parser;
mod generator;

#[derive(Clone)]
pub struct MigrationState {
    pub path: path::PathBuf
}

pub fn migration<'cx>(cx: &'cx mut base::ExtCtxt, sp: codemap::Span, tokens: &[ast::TokenTree]) -> Box<base::MacResult + 'cx> {

    // Parse a full ModelState from the input, emitting errors if used incorrectly.
    let state: MigrationState = super::Parser::parse(&mut parse::tts_to_parser(cx.parse_sess(), tokens.to_vec(), cx.cfg()), (sp, &mut*cx));
    state.generate(sp, cx, ())
}