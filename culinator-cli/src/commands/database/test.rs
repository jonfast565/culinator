use super::*;
#[test]
fn initializes_database_file() {
    let path =
        std::env::temp_dir().join(format!("culinator-cli-{}.sqlite3", uuid::Uuid::new_v4()));
    init_database(&path).expect("initialize");
    assert!(path.exists());
    let _ = std::fs::remove_file(path);
}
