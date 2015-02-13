use syntax::{ast, codemap};
use syntax::ext::base;
use syntax::parse::parser;

impl<'a, 'b> super::super::Parser<(codemap::Span, &'a mut base::ExtCtxt<'b>)> for super::MigrationState {
    fn parse(parser: &mut parser::Parser,
             (_sp, _cx): (codemap::Span, &'a mut base::ExtCtxt)) -> super::MigrationState {
        let item = parser.parse_expr();

        let path = match &item.node {
            &ast::ExprLit(ref item) => {
                match &item.node {
                    &ast::LitStr(ref s, _) => s,
                    _ => panic!("Please provide simple literal path to migrations")
                }
            },
            _ => {
                panic!("Please provide simple literal path to migrations")
            }
        };

        super::MigrationState{
            path: ::std::env::current_dir().map(|dir| dir.join(&Path::new(path))).unwrap()
        }
    }
}
