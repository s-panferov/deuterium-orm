
## Working example

```rust

extern crate deuterium;
extern crate time;

use time::Timespec;

// Please use glob, we need A LOOOT of stuff from Deuterium
use deuterium::*;

// All magic is done by compiler plugin. Please note that every field of 
// `FieldType` will be converted to Option<FieldType> for partial init
// support.
deuterium_model! jedi {
    pub struct Jedi {
        id: i32,
        name: String,
        force_level: i32,
        side: bool,
        created_at: Timespec,
        updated_at: Timespec
    }
}

// Example of custom model methods
impl Jedi {
    pub fn ordered() -> SelectQuery<(), LimitMany, Jedi> {
        // All fields available as `{{field_name}}_f` methods
        Jedi::from().select_all().order_by(&Jedi::created_at_f())
    }
}

fn setup_tables(cn: &PostgresConnection) {
   // DeuteriumORM don't validate schema for now so be carefull
   cn.batch_execute(r#"
        DROP TABLE IF EXISTS jedi CASCADE;
        CREATE TABLE jedi (
            id          serial PRIMARY KEY,
            name        varchar(40) NOT NULL,
            force_level integer,
            side        boolean,
            created_at  timestamptz DEFAULT CURRENT_TIMESTAMP,
            updated_at  timestamptz DEFAULT CURRENT_TIMESTAMP 
        );

        INSERT INTO jedi (name, force_level, side) VALUES
            ('Luke Skywalker', 100, true),
            ('Anakin Skywalker', 100, false);
    "#).unwrap();
}

// Setup PostgreSQL connection
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

fn main() {
    let pool = setup_pg();
    let cn = pool.get().unwrap();

    setup_tables(&*cn);

    // Get Vec<Jedi> from database
    Jedi::ordered().where_(Jedi::name_f().is("Luke Skywalker")).query_list(&*cn);

    // Get Option<Jedi> from database
    Jedi::ordered().where_(Jedi::name_f().is("Anakin Skywalker")).first().query(&*cn).unwrap();
}

# Tests

Please run tests with `RUST_TEST_TASKS=1 cargo test`