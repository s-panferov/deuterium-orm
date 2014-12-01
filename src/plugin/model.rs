use syntax::{ast, codemap, parse};
use syntax::ptr::P;
use syntax::ext::base;

use plugin::parse::Parse;
use plugin::generate::Generate;

#[deriving(Clone)]
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
    let state: ModelState = Parse::parse(&mut parse::tts_to_parser(cx.parse_sess(), tokens, cx.cfg()), (sp, &mut*cx, Some(name)));

    state.generate(sp, cx, ())
}

#[deriving(Clone)]
pub struct MigrationState {
    pub path: Path
}

pub fn migration<'cx>(cx: &'cx mut base::ExtCtxt, sp: codemap::Span, tokens: &[ast::TokenTree]) -> Box<base::MacResult + 'cx> {
    
    // Parse a full ModelState from the input, emitting errors if used incorrectly.
    let state: MigrationState = Parse::parse(&mut parse::tts_to_parser(cx.parse_sess(), tokens.to_vec(), cx.cfg()), (sp, &mut*cx));
    state.generate(sp, cx, ())
}
