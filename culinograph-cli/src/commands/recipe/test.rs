use super::*;
#[test]
fn check_recipe_accepts_valid_file() {
    let path = std::env::temp_dir().join(format!("culinograph-{}.cg", uuid::Uuid::new_v4()));
    std::fs::write(&path, "culinograph 0.3; recipe x { title \"X\"; }").expect("write");
    check_recipe(&path).expect("check");
    let _ = std::fs::remove_file(path);
}
