use time::now_utc;
use std::io::{File, Open, ReadWrite};

pub fn gen_timecode() -> String {
    now_utc().strftime("%Y%m%d%H%M%S").unwrap()
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