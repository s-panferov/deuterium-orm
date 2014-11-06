use time::now_utc;
use std::io::{File, Open, ReadWrite};
use postgres::Connection;
use std::collections::hash_map::HashMap;

pub fn gen_timecode() -> String {
    now_utc().strftime("%y%m%d%H%M%S").unwrap()
}

pub fn gen_full_name(name: &str) -> String {
    format!("_{}_{}", gen_timecode(), name)
}

pub fn create_migration_file(name: &str, base_path: Path) -> String {
    let full_name = gen_full_name(name);
    let final_path = base_path.join(format!("{}.rs", full_name));

    let mut file = match File::open_mode(&final_path, Open, ReadWrite) {
        Ok(f) => f,
        Err(e) => panic!("file error: {}", e),
    };

    file.write(b"").unwrap();
    full_name
}

pub trait RawMigration<Conn> {
    fn version(&self) -> u64;
    fn up(&self, cn: &Conn);
    fn down(&self, cn: &Conn);
}

pub type Migrations = Vec<Box<RawMigration<Connection> + 'static>>;
pub type MigrationRefs<'a> = Vec<&'a Box<RawMigration<Connection> + 'static>>;

pub fn ensure_schema_migrations(cn: &Connection) {
    cn.execute("CREATE TABLE IF NOT EXISTS schema_migrations (
         version BIGINT NOT NULL
    );", &[]).unwrap();
}

pub fn insert_version(version: &i64, cn: &Connection) {
    cn.execute("INSERT INTO schema_migrations VALUES ($1);", &[version]).unwrap();
}

pub fn delete_version(version: &i64, cn: &Connection) {
    cn.execute("DELETE FROM schema_migrations WHERE version = $1;", &[version]).unwrap();
}

pub fn get_versions_as_hash(cn: &Connection) -> HashMap<i64, bool> {
    let stmt = cn.prepare("SELECT version FROM schema_migrations ORDER BY version desc;").unwrap();
    let mut rows = stmt.query(&[]).unwrap();
    let mut db_versions: HashMap<i64, bool> = HashMap::new();

    for row in rows {
        db_versions.insert(row.get(0), true);
    }

    db_versions
}

pub fn get_versions_as_vec(cn: &Connection) -> Vec<i64> {
    let stmt = cn.prepare("SELECT version FROM schema_migrations ORDER BY version desc;").unwrap();
    let mut rows = stmt.query(&[]).unwrap();
    let mut db_versions: Vec<i64> = vec![];

    for row in rows {
        db_versions.push(row.get(0));
    }

    db_versions
}

pub fn run(migrations: &Migrations, cn: &Connection) {
    ensure_schema_migrations(cn);
    let db_versions = get_versions_as_hash(cn);

    let migrations_to_run: MigrationRefs = migrations.iter().filter(|m| {
        let version = m.version().to_i64().unwrap();
        !db_versions.contains_key(&version)
    }).collect();

    for migration in migrations_to_run.iter() {
        migration.up(cn);
        insert_version(&migration.version().to_i64().unwrap(), cn);

        println!("Migration completed: {}", migration.version());
    }
}

pub fn rollback(steps: uint, migrations: &Migrations, cn: &Connection) {
    ensure_schema_migrations(cn);
    let db_versions = get_versions_as_vec(cn);
    let db_versions_to_run = db_versions.slice(0, steps);

    let migrations_to_run: MigrationRefs = migrations.iter().filter(|m| {
        let version = m.version().to_i64().unwrap();
        db_versions_to_run.contains(&version)
    }).collect();

    for migration in migrations_to_run.iter() {
        migration.down(cn);
        delete_version(&migration.version().to_i64().unwrap(), cn);

        println!("Migration reverted: {}", migration.version());
    }
}