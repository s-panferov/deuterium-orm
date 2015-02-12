#![feature(plugin)]
#![feature(env)]
#![feature(core)]
#![feature(test)]

#![plugin(deuterium_orm)]

#[macro_use] extern crate deuterium_orm;
extern crate time;
extern crate deuterium;

extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate test;

use std::env;
use deuterium::*;
use deuterium_orm::*;
use time::Timespec;

use postgres::Connection;

macro_rules! assert_sql {
    ($query:expr, $s:expr) => (
        assert_eq!($query.to_final_sql().as_slice(), $s)
    )
}

#[derive(Clone, Debug, PartialEq, FromPrimitive)]
pub enum Side {
    DarkSide,
    LightSide,
}

deuterium_enum!(Side);

deuterium_model! jedi {
    #[primary_key(id)]
    #[before_create(created_at)]
    #[before_save(updated_at)]
    pub struct Jedi {
        id: i32,
        name: String,
        force_level: i32,
        side: Side,
        created_at: Timespec,
        updated_at: Timespec
    }
}

impl Jedi {
    pub fn ordered() -> SelectQuery<(), LimitMany, Jedi> {
        Jedi::table().select_all().order_by(&Jedi::created_at_f())
    }
}

fn created_at(token: &mut Jedi) {
    token.set_created_at(::time::get_time());
}

fn updated_at(token: &mut Jedi) {
    token.set_updated_at(::time::get_time());
}

fn setup_tables(cn: &Connection) {
   cn.batch_execute(r#"
        DROP TABLE IF EXISTS jedi CASCADE;
        CREATE TABLE jedi (
            id          serial PRIMARY KEY,
            name        varchar(40) NOT NULL,
            force_level integer NOT NULL,
            side        SMALLINT NOT NULL,
            created_at  timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at  timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL
        );

        INSERT INTO jedi (name, force_level, side) VALUES
            ('Luke Skywalker', 100, 1),
            ('Mace Windu', 90, 1),
            ('Obi-Wan Kenoby', 99, 1),
            ('Kit Fisto', 70, 1),
            ('Count Dooku', 99, 0),
            ('Darth Maul', 70, 0),
            ('Anakin Skywalker', 100, 0);

    "#).unwrap();
}

fn setup_pg() -> adapter::postgres::PostgresPool {

    let connection_uri = match env::var_string("POSTGRES_CONNECTION") {
        Ok(val) => val,
        Err(_) => "postgres://localhost/jedi".to_string()
    };

    let manager = r2d2_postgres::PostgresConnectionManager::new(connection_uri.as_slice(), ::postgres::SslMode::None);
    let config = r2d2::Config {
        pool_size: 5,
        test_on_check_out: true,
        ..std::default::Default::default()
    };

    let handler = Box::new(r2d2::NoopErrorHandler);
    r2d2::Pool::new(config, manager, handler).unwrap()
}

#[test]
fn select() {
    let pool = setup_pg();
    let cn = pool.get().unwrap();

    setup_tables(&*cn);

    assert_eq!((query_models!(
        &Jedi::ordered().where_(Jedi::side_f().is(Side::LightSide)),
        &*cn, &[]
    )).len(), 4);

    let anakin = (query_model!(
        &Jedi::ordered().where_(Jedi::name_f().is("Anakin Skywalker".to_string())).first(),
        &*cn, &[]
    )).unwrap();

    assert_eq!(anakin.get_force_level(), &100);
    assert_eq!(anakin.get_side(), &Side::DarkSide);
}

#[test]
fn insert() {
    let pool = setup_pg();
    let cn = pool.get().unwrap();
    setup_tables(&*cn);

    let mut jedi = Jedi::empty();
    jedi.set_name("Pants Olmos".to_string());
    jedi.set_force_level(10);
    jedi.set_side(Side::DarkSide);

    assert_eq!(exec_pg!(&jedi.create_query(), &*cn, &[]), 1);

    let olmos = (query_model!(
        &Jedi::table().select_all().where_(Jedi::name_f().is("Pants Olmos".to_string())).first(),
        &*cn, &[]
    )).unwrap();

    assert_eq!(olmos.get_name(), &"Pants Olmos".to_string());
    assert_eq!(olmos.get_force_level(), &10);
    assert_eq!(olmos.get_side(), &Side::DarkSide);
}

#[test]
fn update() {
    let pool = setup_pg();
    let cn = pool.get().unwrap();
    setup_tables(&*cn);

    let mut anakin = (query_model!(
        &Jedi::table().select_all().where_(Jedi::name_f().is("Anakin Skywalker".to_string())).first(),
        &*cn, &[]
    )).unwrap();

    assert_eq!(anakin.get_side(), &Side::DarkSide);

    anakin.set_side(Side::LightSide);
    assert_eq!(exec_pg!(&anakin.update_query(), &*cn, &[]), 1);
}

#[test]
fn delete() {
    let pool = setup_pg();
    let cn = pool.get().unwrap();
    setup_tables(&*cn);

    let mut anakin = (query_model!(
        &Jedi::table().select_all().where_(Jedi::name_f().is("Anakin Skywalker".to_string())).first(),
        &*cn, &[]
    )).unwrap();

    assert_eq!(exec_pg!(&anakin.delete_query(), &*cn, &[]), 1);
}
