use num::ToPrimitive;
use time::now_utc;
use std::io::Write;
use std::fs;
use std::path;
use postgres;
use std::collections;

pub fn gen_timecode() -> String {
    now_utc().strftime("%y%m%d%H%M%S").unwrap().to_string()
}

pub fn gen_full_name(name: &str) -> String {
    format!("_{}_{}", gen_timecode(), name)
}

pub fn create_migration_file(name: &str, base_path: path::PathBuf) -> String {
    let full_name = gen_full_name(name);
    let final_path = base_path.join(&format!("{}.rs", full_name)[..]);

    let mut file = match fs::OpenOptions::new().create(true).write(true).open(&final_path) {
        Ok(f) => f,
        Err(e) => panic!("file error: {}", e),
    };

    file.write_all(b"").unwrap();
    full_name
}

pub struct Migration<Conn> {
    version: u64,
    name: String,
    raw: Box<RawMigration<Conn> + 'static>
}

impl<Conn> Migration<Conn> {
    pub fn new(version: u64, name: &str, raw: Box<RawMigration<Conn> + 'static>) -> Migration<Conn> {
        Migration {
            version: version,
            name: name.to_string(),
            raw: raw
        }
    }

    pub fn version(&self) -> &u64 { &self.version }
    pub fn name(&self) -> &str { &self.name }
    pub fn raw(&self) -> &Box<RawMigration<Conn> + 'static> { &self.raw }
}

pub trait RawMigration<Conn> {
    fn up(&self, cn: &Conn);
    fn down(&self, cn: &Conn);
}

pub type Migrations = Vec<Box<Migration<postgres::Connection>>>;
pub type MigrationRefs<'a> = Vec<&'a Box<Migration<postgres::Connection>>>;

pub fn ensure_schema_migrations(cn: &postgres::Connection) {
    cn.execute("CREATE TABLE IF NOT EXISTS schema_migrations (
         version BIGINT NOT NULL
    );", &[]).unwrap();
}

pub fn insert_version(version: &i64, cn: &postgres::Connection) {
    cn.execute("INSERT INTO schema_migrations VALUES ($1);", &[version]).unwrap();
}

pub fn delete_version(version: &i64, cn: &postgres::Connection) {
    cn.execute("DELETE FROM schema_migrations WHERE version = $1;", &[version]).unwrap();
}

pub fn get_versions_as_hash(cn: &postgres::Connection) -> collections::HashMap<i64, bool> {
    let stmt = cn.prepare("SELECT version FROM schema_migrations ORDER BY version desc;").unwrap();
    let rows = stmt.query(&[]).unwrap();
    let mut db_versions: collections::HashMap<i64, bool> = collections::HashMap::new();

    for row in rows {
        db_versions.insert(row.get(0), true);
    }

    db_versions
}

pub fn get_versions_as_vec(cn: &postgres::Connection) -> Vec<i64> {
    let stmt = cn.prepare("SELECT version FROM schema_migrations ORDER BY version desc;").unwrap();
    let rows = stmt.query(&[]).unwrap();
    let mut db_versions: Vec<i64> = vec![];

    for row in rows {
        db_versions.push(row.get(0));
    }

    db_versions
}

pub fn run(migrations: &Migrations, cn: &postgres::Connection) {
    ensure_schema_migrations(cn);
    let db_versions = get_versions_as_hash(cn);

    let migrations_to_run: MigrationRefs = migrations.iter().filter(|m| {
        let version = m.version().to_i64().unwrap();
        !db_versions.contains_key(&version)
    }).collect();

    for migration in migrations_to_run.iter() {
        migration.raw().up(cn);
        insert_version(&migration.version().to_i64().unwrap(), cn);

        println!("Migration completed: {} {}", migration.version(), migration.name());
    }
}

pub fn rollback(steps: usize, migrations: &Migrations, cn: &postgres::Connection) {
    ensure_schema_migrations(cn);
    let db_versions = get_versions_as_vec(cn);
    let db_versions_to_run = &db_versions[0..steps];

    let migrations_to_run: MigrationRefs = migrations.iter().filter(|m| {
        let version = m.version().to_i64().unwrap();
        db_versions_to_run.contains(&version)
    }).collect();

    for migration in migrations_to_run.iter() {
        migration.raw().down(cn);
        delete_version(&migration.version().to_i64().unwrap(), cn);

        println!("Migration reverted: {} {}", migration.version(), migration.name());
    }
}
