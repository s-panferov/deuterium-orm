#![feature(phase)]
#![feature(globs)]
#![feature(macro_rules)]

#[phase(plugin)]
extern crate deuterium_orm;
extern crate deuterium_orm;
extern crate time;
extern crate deuterium;

use deuterium::*;
use deuterium_orm::*;
use std::sync::Arc;
use std::default::Default;

use time::Timespec;

macro_rules! assert_sql(
    ($query:expr, $s:expr) => (
        assert_eq!($query.to_final_sql().as_slice(), $s)
    )
)

deuterium_model! jedi {
    #[allow(dead_code)]
    pub struct Jedi {
        id: String,
        name: String,
        force_level: u8,
        side: bool,
        created_at: Timespec,
        updated_at: Timespec
    }
}

impl JediTable {
    pub fn ordered() -> SelectQuery<(), LimitMany> {
        JediTable::from().select_all().order_by(&JediTable::created_at())
    }
}

#[test]
fn test() {
    let query = JediTable::ordered().where_(JediTable::name().is("Luke"));
    assert_sql!(query, "SELECT * FROM jedi WHERE name = 'Luke' ORDER BY created_at ASC;")

    let jedi = create_model!(Jedi, name: "Luke Skywalker".to_string());
}