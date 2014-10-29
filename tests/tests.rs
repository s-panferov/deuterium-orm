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
use time::Timespec;

use postgres::NoSsl;
use postgres::PostgresConnection;
use r2d2_postgres::PostgresPoolManager;

macro_rules! assert_sql(
    ($query:expr, $s:expr) => (
        assert_eq!($query.to_final_sql().as_slice(), $s)
    )
)

deuterium_model! jedi {
    pub struct Jedi {
        id: i32,
        name: String,
        force_level: i32,
        light_side: bool,
        created_at: Timespec,
        updated_at: Timespec
    }
}

impl Jedi {
    pub fn ordered() -> SelectQuery<(), LimitMany, Jedi> {
        Jedi::from().select_all().order_by(&Jedi::created_at_f())
    }
}

fn setup_tables(cn: &PostgresConnection) {
   cn.batch_execute(r#"
        DROP TABLE IF EXISTS jedi CASCADE;
        CREATE TABLE jedi (
            id          serial PRIMARY KEY,
            name        varchar(40) NOT NULL,
            force_level integer,
            light_side  boolean,
            created_at  timestamptz DEFAULT CURRENT_TIMESTAMP,
            updated_at  timestamptz DEFAULT CURRENT_TIMESTAMP 
        );

        INSERT INTO jedi (name, force_level, light_side) VALUES
            ('Luke Skywalker', 100, true),
            ('Mace Windu', 90, true),
            ('Obi-Wan Kenoby', 99, true),
            ('Kit Fisto', 70, true),
            ('Count Dooku', 99, false),
            ('Darth Maul', 70, false),
            ('Anakin Skywalker', 100, false);

    "#).unwrap();
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
    let cn = pool.get().unwrap();

    setup_tables(&*cn);

    Jedi::ordered().where_(Jedi::name_f().is("Luke Skywalker".to_string())).query_list(&*cn);
    Jedi::ordered().where_(Jedi::name_f().is("Anakin Skywalker".to_string())).first().query(&*cn).unwrap();
}