use syntax::{ast, codemap};
use syntax::ptr::P;
use syntax::ext::base;
use syntax::owned_slice::OwnedSlice;
use syntax::codemap::Spanned;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::ext::quote::rt::ToSource;

use syntax::ext::build::AstBuilder;

use std::ascii::AsciiExt;

use plugin::model::{ModelState, MigrationState};

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

        let ty_def_macro_body = format!("{}, {}, {}, {}, {}, \"{}\", {}, {}, {}",
            struct_name,
            struct_name + "Meta",
            ts_name,
            ts_name + "ManySelectQueryExt",
            ts_name + "OneSelectQueryExt",
            name.name.as_str(),
            ts_fields.to_string(),
            self.before_create.to_string(),
            self.before_save.to_string()
        );

        let mut impls = vec![];

        let impl_mac = generate_macro_invocation(cx, "define_model", ty_def_macro_body, sp);
        impls.push(impl_mac);

        match self.primary_key {
            None => panic!("Please provide primary key for {}", struct_name),
            Some(ref primary_key) if primary_key.is_empty() => panic!("Please provide primary key for {}", struct_name),
            Some(ref primary_key) => {
                let lookup_predicate = generate_lookup_predicate(&struct_name, primary_key);
                let impl_primary_key_mac = generate_macro_invocation(cx, "primary_key", format!("self, {}, {}", 
                    struct_name,
                    lookup_predicate
                ), sp);

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

impl Generate<()> for MigrationState {
    fn generate<'a>(self, sp: codemap::Span, cx: &mut base::ExtCtxt, _: ()) -> Box<base::MacResult + 'a> {

        let pathes = ::std::io::fs::readdir(&self.path).unwrap();
        let mut migrations = vec![];

        let path_checker = regex!(r"^_(\d{12})");
        let upcaser = regex!(r"_([a-z])");

        for path in pathes.iter() {
            let filestem = path.filestem_str().unwrap();
            let captures = path_checker.captures(filestem);

            if captures.is_none() { continue };

            let captures = captures.unwrap();
            let tm = captures.at(1);
            let version: u64 = from_str(tm).unwrap();
            let name = filestem.replace(captures.at(0), "");

            let name = upcaser.replace_all(name.as_slice(), |caps: &::regex::Captures| {
                caps.at(1).to_ascii_upper()
            });

            migrations.push((filestem.to_string(), version, name.to_string()).to_string());
        }

        let macro_body = migrations.connect(", ");

        let mut impls = vec![];
        impls.push(generate_macro_invocation(cx, "migrations", macro_body, sp));
        base::MacItems::new(impls.into_iter())

    }
}

fn generate_macro_invocation(cx: &mut base::ExtCtxt, macro_name: &str, macro_body: String, sp: codemap::Span) -> P<ast::Item> {
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
                                types: OwnedSlice::empty()  
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