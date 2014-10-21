use syntax::{ast, codemap, parse};
use syntax::ptr::P;
use syntax::ext::base;

use dt_plugin::parse::Parse;
use dt_plugin::generate::Generate;

#[deriving(Clone)]
pub struct ModelState {
    pub mod_name: ast::Ident,
    pub model: P<ast::Item>,
}

pub fn model<'a>(cx: &'a mut base::ExtCtxt, sp: codemap::Span,
                name: ast::Ident, tokens: Vec<ast::TokenTree>) -> Box<base::MacResult + 'a> {
    
    // Parse a full ModelState from the input, emitting errors if used incorrectly.
    let state: ModelState = Parse::parse(&mut parse::tts_to_parser(cx.parse_sess(), tokens, cx.cfg()), (sp, &mut*cx, Some(name)));

    state.generate(sp, cx, ())
}
