#![feature(phase)]
#![feature(globs)]
#![feature(macro_rules)]

#[phase(plugin)]
extern crate deuterium_plugin;
extern crate deuterium_orm;
extern crate time;
extern crate deuterium;

extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;

use deuterium::*;
use deuterium_orm::*;
use std::sync::Arc;

use postgres::NoSsl;
use r2d2_postgres::PostgresPoolManager;

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

fn setup_pg() -> adapter::postgres::PostgresPool {
    let manager = PostgresPoolManager::new("postgres://panferov@localhost/jedi", NoSsl);
    let config = r2d2::Config {
        pool_size: 5,
        test_on_check_out: true,
        ..std::default::Default::default()
    };

    let handler = r2d2::NoopErrorHandler;
    r2d2::Pool::new(config, manager, handler).unwrap()
}

#[test]
fn test() {

    let pool = setup_pg();
    pool.get().unwrap();

    let query = JediTable::ordered().where_(JediTable::name().is("Luke")).first();
    assert_sql!(query, "SELECT * FROM jedi WHERE name = 'Luke' ORDER BY created_at ASC LIMIT 1;")

    let jedi = create_model!(Jedi, name: "Luke Skywalker".to_string());
}