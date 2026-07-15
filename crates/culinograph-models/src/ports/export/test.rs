use super::*;

#[test]
fn exporter_port_is_object_safe() {
    fn accepts(_: &dyn RecipeExporter) {}
    let _ = accepts;
}
