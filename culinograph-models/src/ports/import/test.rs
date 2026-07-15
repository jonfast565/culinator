use super::*;
#[test]
fn import_ports_are_object_safe() {
    fn accepts(_: &dyn SettingsStore) {}
    let _ = accepts;
}
