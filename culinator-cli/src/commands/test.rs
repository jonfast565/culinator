#[test]
fn command_exports_are_available() {
    let _ = super::check_recipe as fn(&std::path::Path) -> anyhow::Result<()>;
}
