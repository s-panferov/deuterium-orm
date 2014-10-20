use syntax::{ast, ast_util, codemap};
use syntax::ptr::P;
use syntax::ext::base;
use syntax::owned_slice::OwnedSlice;
use syntax::codemap::Spanned;
use syntax::ext::quote::rt::ExtParseUtils;

use syntax::ext::build::AstBuilder;
use model::{ModelState};

pub trait Generate<Cfg> {
    fn generate<'a>(self, codemap::Span, &mut base::ExtCtxt, Cfg) -> Box<base::MacResult + 'a>;
}

impl Generate<()> for ModelState {
    fn generate<'a>(self, sp: codemap::Span, cx: &mut base::ExtCtxt, _: ()) -> Box<base::MacResult + 'a> {
        let name = self.mod_name.clone();
        let struct_name = self.model.ident.clone();

        let ts_name = struct_name.name.as_str().to_string() + "Table".to_string();
        
        let model_struct_def = match &self.model.node {
            &ast::ItemStruct(ref model_struct_def, _) => model_struct_def.clone(),
            _ => fail!("Unexpected item")
        };

        let mut ts_fields = vec![];
        for field in model_struct_def.fields.iter() {
            let field_ty = match &field.node.ty.node {
                &ast::TyPath(ref path, _, _) => {
                    let path_idents: Vec<ast::Ident> = path.segments.iter().map(|s| s.identifier).collect();
                    ast_util::path_name_i(path_idents.as_slice())
                },
                _ => fail!("??")
            };
            // Name table field as model field
            let field_name = match field.node.kind {
                ast::NamedField(field_ident, _) => field_ident.name.as_str().to_string(),
                _ => fail!("Can't use unnamed fields in models")
            };

            ts_fields.push((field_name, field_ty));
        }

        let ty_def_macro_body = format!("{}, {}, {}, \"{}\", {}",
            struct_name.name.as_str(),
            ts_name,
            ts_name + "Instance",
            name.name.as_str(),
            ts_fields.to_string()
        );

        let impl_mac = P(ast::Item {
            ident: cx.ident_of(""),
            attrs: vec![],
            id: ast::DUMMY_NODE_ID,
            node: ast::ItemMac(Spanned{
                node: ast::MacInvocTT(
                    ast::Path {
                        span: sp,
                        global: false,
                        segments: vec![ast::PathSegment{
                            identifier: cx.ident_of("define_model"),
                            lifetimes: vec![],
                            types: OwnedSlice::empty()
                        }]
                    },
                    cx.parse_tts(ty_def_macro_body),
                    0
                ),
                span: sp
            }),
            vis: self.model.vis,
            span: sp
        });

        base::MacItems::new(vec![impl_mac].into_iter())
    }
}