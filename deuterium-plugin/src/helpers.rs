use syntax::{ast, codemap};
use syntax::ptr::P;
use syntax::ext::base;
use syntax::owned_slice::OwnedSlice;
use syntax::codemap::Spanned;
use syntax::ext::quote::rt::ExtParseUtils;

pub fn generate_macro_invocation(cx: &mut base::ExtCtxt, macro_name: &str, macro_body: String, sp: codemap::Span) -> P<ast::Item> {
    P(ast::Item {
        ident: cx.ident_of(""),
        attrs: vec![],
        id: ast::DUMMY_NODE_ID,
        node: ast::ItemMac(Spanned{
            node: ast::MacInvocTT(
                ast::Path {
                    span: sp,
                    global: false,
                    segments: vec![ast::PathSegment{
                        identifier: cx.ident_of(macro_name),
                        parameters: ast::AngleBracketedParameters(
                            ast::AngleBracketedParameterData {
                                lifetimes: vec![],
                                types: OwnedSlice::empty(),
                                bindings: OwnedSlice::empty()
                            }
                        )
                    }]
                },
                cx.parse_tts(macro_body),
                0
            ),
            span: sp
        }),
        vis: ast::Public,
        span: sp
    })
}
