use syntax::{ast, codemap, parse};
use syntax::ptr::P;
use syntax::ext::base;

use parse::Parse;
use generate::Generate;

#[deriving(Clone)]
pub struct ModelState {
    pub mod_name: ast::Ident,
    pub model: P<ast::Item>,
}

/// Defines the overarching `describe!` syntax extension.
///
/// All other macros in stainless are actually "fake" in the sense
/// that they are detected and expanded inside of the implementation
/// of `describe!`.
pub fn model<'a>(cx: &'a mut base::ExtCtxt, sp: codemap::Span,
                name: ast::Ident, tokens: Vec<ast::TokenTree>) -> Box<base::MacResult + 'a> {
    
    // Parse a full ModelState from the input, emitting errors if used incorrectly.
    let state: ModelState = Parse::parse(&mut parse::tts_to_parser(cx.parse_sess(), tokens, cx.cfg()), (sp, &mut*cx, Some(name)));

    state.generate(sp, cx, ())
}
