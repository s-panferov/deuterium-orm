use syntax::{ast, codemap};
use syntax::ext::base;
use syntax::parse::parser::Parser;
use syntax::attr::first_attr_value_str_by_name;

use model::{ModelState};

pub trait Parse<Cfg> {
    fn parse(&mut Parser, Cfg) -> Self;
}

impl<'a, 'b> Parse<(codemap::Span, &'a mut base::ExtCtxt<'b>, Option<ast::Ident>)> for ModelState {
    fn parse(parser: &mut Parser,
             (_sp, _cx, name): (codemap::Span, &'a mut base::ExtCtxt, Option<ast::Ident>)) -> ModelState {

        let name = match name {
            Some(name) => name,
            None => {
                parser.fatal("Name of the table must be present in model block")
            }
        };

        // Guards on struct
        let maybe_model_struct = parser.parse_item_with_outer_attributes();
        let model_struct = match maybe_model_struct {
            Some(model_struct) => {
                match model_struct.node {
                    ast::ItemStruct(_, _) => model_struct,
                    _ => {
                        let span = parser.span;
                        parser.span_fatal(span, "Only struct can be presented in the body")
                    }
                }
            },
            None => {
                let span = parser.span;
                parser.span_fatal(span, "Please provide model struct")
            }
        };

        let primary_key = first_attr_value_str_by_name(model_struct.attrs.as_slice(), "pk").map(|s| s.get().to_string());

        ModelState {
            mod_name: name,
            model: model_struct,
            primary_key: primary_key
        }
    }
}

