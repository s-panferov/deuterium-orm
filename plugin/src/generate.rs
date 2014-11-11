use syntax::{ast, codemap};
use syntax::ptr::P;
use syntax::ext::base;
use syntax::owned_slice::OwnedSlice;
use syntax::codemap::Spanned;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::ext::quote::rt::ToSource;

use syntax::ext::build::AstBuilder;

use model::{ModelState};

pub trait Generate<Cfg> {
    fn generate<'a>(self, codemap::Span, &mut base::ExtCtxt, Cfg) -> Box<base::MacResult + 'a>;
}

impl Generate<()> for ModelState {
    fn generate<'a>(self, sp: codemap::Span, cx: &mut base::ExtCtxt, _: ()) -> Box<base::MacResult + 'a> {
        let name = self.mod_name.clone();
        let struct_name = self.model.ident.clone().name.as_str().to_string();
        let ts_name = struct_name + "Table".to_string();
        
        let model_struct_def = match &self.model.node {
            &ast::ItemStruct(ref model_struct_def, _) => model_struct_def.clone(),
            _ => panic!("Unexpected item")
        };

        let mut ts_fields = vec![];
        for field in model_struct_def.fields.iter() {
            let field_ty = field.node.ty.to_source();

            // Name table field as model field
            let (field_name, visibility) = match field.node.kind {
                ast::NamedField(field_ident, visibility) => {
                    let name = field_ident.name.as_str().to_string();
                    let visibility = match visibility {
                        ast::Public => "pub",
                        ast::Inherited => ""
                    };
                    (name, visibility)
                },
                _ => panic!("Can't use unnamed fields in models")
            };

            ts_fields.push((
                field_name.to_string(), 
                field_ty, 
                format!("{}_f", field_name),  
                format!("get_{}", field_name),  
                format!("set_{}", field_name),  
                format!("__{}_changed", field_name),  
                format!("{}_changed", field_name),  
                visibility
            ));
        }

        let ty_def_macro_body = format!("{}, {}, {}, {}, {}, \"{}\", {}",
            struct_name,
            struct_name + "Meta",
            ts_name,
            ts_name + "ManySelectQueryExt",
            ts_name + "OneSelectQueryExt",
            name.name.as_str(),
            ts_fields.to_string()
        );

        let mut impls = vec![];

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
                            parameters: ast::AngleBracketedParameters(
                                ast::AngleBracketedParameterData {
                                    lifetimes: vec![],
                                    types: OwnedSlice::empty()  
                                }
                            )
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

        impls.push(impl_mac);

        match self.primary_key {
            None => panic!("Please provide primary key for {}", struct_name),
            Some(ref primary_key) if primary_key.is_empty() => panic!("Please provide primary key for {}", struct_name),
            Some(ref primary_key) => {
                let lookup_predicate = generate_lookup_predicate(&struct_name, primary_key);

                let impl_primary_key_mac = P(ast::Item {
                    ident: cx.ident_of(""),
                    attrs: vec![],
                    id: ast::DUMMY_NODE_ID,
                    node: ast::ItemMac(Spanned{
                        node: ast::MacInvocTT(
                            ast::Path {
                                span: sp,
                                global: false,
                                segments: vec![ast::PathSegment{
                                    identifier: cx.ident_of("primary_key"),
                                    parameters: ast::AngleBracketedParameters(
                                        ast::AngleBracketedParameterData {
                                            lifetimes: vec![],
                                            types: OwnedSlice::empty()  
                                        }
                                    )
                                }]
                            },
                            cx.parse_tts(format!("self, {}, {}", 
                                struct_name,
                                lookup_predicate
                            )),
                            0
                        ),
                        span: sp
                    }),
                    vis: self.model.vis,
                    span: sp
                });

                impls.push(impl_primary_key_mac);
            }
        }
        
        base::MacItems::new(impls.into_iter())
    }
}

fn generate_lookup_predicate(struct_name: &String, primary_key: &Vec<String>) -> String {
    let keys: Vec<String> = primary_key.iter().map(|pk| {
        format!("({}::{}_f().is(self.get_{}().clone()))", 
            struct_name, 
            pk,
            pk
        )
    }).collect();
    format!("{{{}}}", keys.connect(".and"))
}