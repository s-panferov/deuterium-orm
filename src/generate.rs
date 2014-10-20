use syntax::{ast, abi, ast_util, codemap};
use syntax::ptr::P;
use syntax::ext::base;
use syntax::parse::token;
use syntax::owned_slice::OwnedSlice;
use syntax::codemap::Spanned;

use syntax::ext::build::AstBuilder;
use model::{ModelState};

/// Trait meaning something can be turned into an ast::Item with configuration.
pub trait Generate<Cfg> {
    /// Turn Self into an ast::Item with a configuration object.
    fn generate<'a>(self, codemap::Span, &mut base::ExtCtxt, Cfg) -> Box<base::MacResult + 'a>;
}

impl Generate<()> for ModelState {
    fn generate<'a>(self, sp: codemap::Span, cx: &mut base::ExtCtxt, _: ()) -> Box<base::MacResult + 'a> {
        // Get the name of this mod.
        let name = self.mod_name.clone();
        let struct_name = self.model.ident.clone();

        let ts_name = struct_name.name.as_str().to_string() + "Table".to_string();
        let ts_ident = cx.ident_of(ts_name.as_slice());

        let mut ts_fields = vec![];

        let model_struct_def = match &self.model.node {
            &ast::ItemStruct(ref model_struct_def, _) => model_struct_def.clone(),
            _ => fail!("Unexpected item")
        };

        for field in model_struct_def.fields.iter() {
            let field_ident = match field.node.kind {
                ast::NamedField(field_ident, _) => field_ident,
                _ => fail!("Can't use unnamed fields in models")
            };

            // fail!(ast::NamedField(field_ident, ast::Public).to_string());

            ts_fields.push(Spanned {
                node: ast::StructField_ {
                    kind: ast::NamedField(field_ident, ast::Public),
                    id: ast::DUMMY_NODE_ID,
                    ty: P(ast::Ty{
                        id: ast::DUMMY_NODE_ID,
                        node: ast::TyPath(
                            ast::Path {
                                span: sp,
                                global: false,
                                segments: vec![ast::PathSegment{
                                    identifier: cx.ident_of("NamedField"),
                                    lifetimes: vec![],
                                    types: OwnedSlice::from_vec(vec![field.node.ty.clone()])
                                }]
                            },
                            None,
                            ast::DUMMY_NODE_ID
                        ),
                        span: sp
                    }),
                    attrs: vec![]
                },
                span: sp.clone()
            })
        }

        // Create the final Item that represents the benchmark.
        let ts = P(ast::Item {
            ident: ts_ident,
            attrs: vec![],
            id: ast::DUMMY_NODE_ID,
            node: ast::ItemStruct(
                P(ast::StructDef {
                    fields: ts_fields,
                    ctor_id: None
                }),
                ast_util::empty_generics(),
            ),
            vis: self.model.vis,
            span: sp
        });

        base::MacItems::new(vec![self.model, ts].into_iter())
    }
}