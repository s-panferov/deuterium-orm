use syntax::{ast, codemap};
use syntax::ext::base;
use syntax::util::small_vector;
use syntax::ext::build::AstBuilder;
use syntax::print::pprust;

use super::super::helpers;

impl super::super::Generator<()> for super::ModelState {
    fn generate<'a>(self, sp: codemap::Span, cx: &mut base::ExtCtxt, _: ()) -> Box<base::MacResult + 'a> {
        let name = self.mod_name.clone();
        let struct_name = self.model.ident.clone().name.as_str().to_string();
        let ts_name = struct_name.clone() + "Table";

        let model_struct_def = match &self.model.node {
            &ast::ItemStruct(ref model_struct_def, _) => model_struct_def.clone(),
            _ => panic!("Unexpected item")
        };

        let mut ts_fields = vec![];
        for field in model_struct_def.fields.iter() {
            let field_ty = pprust::ty_to_string(&*field.node.ty);

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
                visibility.to_string()
            ));
        }

        let ty_def_macro_body = format!("{}, {}, {}, {}, {}, \"{}\", {}, {}, {}",
            struct_name.clone(),
            struct_name.clone() + "Meta",
            ts_name.clone(),
            ts_name.clone() + "ManySelectQueryExt",
            ts_name.clone() + "OneSelectQueryExt",
            name.name.as_str(),
            format!("[{}]", ts_fields.iter().map(|s|
                format!("({})", &[&s.0, &s.1, &s.2, &s.3, &s.4, &s.5, &s.6, &s.7].connect(", "))
            ).collect::<Vec<String>>().connect(", ")),
            format!("[{}]", self.before_create.connect(", ")),
            format!("[{}]", self.before_save.connect(", "))
        );

        let mut impls = vec![];

        let impl_mac = helpers::generate_macro_invocation(cx, "define_model", ty_def_macro_body, sp);
        impls.push(impl_mac);

        match self.primary_key {
            None => panic!("Please provide primary key for {}", struct_name),
            Some(ref primary_key) if primary_key.is_empty() => panic!("Please provide primary key for {}", struct_name),
            Some(ref primary_key) => {
                let lookup_predicate = generate_lookup_predicate(&struct_name, primary_key);
                let impl_primary_key_mac = helpers::generate_macro_invocation(cx, "primary_key", format!("self, {}, {}",
                    struct_name,
                    lookup_predicate
                ), sp);

                impls.push(impl_primary_key_mac);
            }
        }

        base::MacEager::items(small_vector::SmallVector::many(impls))
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
