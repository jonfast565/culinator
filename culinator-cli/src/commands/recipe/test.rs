use super::*;
#[test]
fn check_recipe_accepts_valid_file() {
    let path = std::env::temp_dir().join(format!("culinator-{}.cg", uuid::Uuid::new_v4()));
    std::fs::write(&path, "culinator 0.3; recipe x { title \"X\"; }").expect("write");
    check_recipe(&path).expect("check");
    let _ = std::fs::remove_file(path);
}

#[test]
fn check_recipe_fails_for_semantic_diagnostics() {
    let path = std::env::temp_dir().join(format!("culinator-{}.cg", uuid::Uuid::new_v4()));
    std::fs::write(
        &path,
        r#"culinator 0.3;
recipe x {
    title "X";
    process method {
        operation mix does mix {
            input missing;
        }
    }
}"#,
    )
    .expect("write");
    let result = check_recipe(&path);
    let _ = std::fs::remove_file(path);
    assert!(result.is_err());
}
