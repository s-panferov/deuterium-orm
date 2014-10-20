use syntax::{ast, codemap};
use syntax::ext::base;
use syntax::parse::parser::Parser;

use model::{ModelState};

/// Trait that means something can be parsed with a configuration.
pub trait Parse<Cfg> {
    /// Parse Self from a Parser and a configuration object.
    fn parse(&mut Parser, Cfg) -> Self;
}

impl<'a, 'b> Parse<(codemap::Span, &'a mut base::ExtCtxt<'b>, Option<ast::Ident>)> for ModelState {
    fn parse(parser: &mut Parser,
             (_sp, _cx, name): (codemap::Span, &'a mut base::ExtCtxt, Option<ast::Ident>)) -> ModelState {

        let name = match name {
            // Top-level describe block.
            Some(name) => name,
            // Nested describe block.
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

        ModelState {
            mod_name: name,
            model: model_struct
        }
    }
}

