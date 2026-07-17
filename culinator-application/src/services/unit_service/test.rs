use crate::{UnitConvertRequest, UnitFormatRequest, UnitService};
use culinator_models::{Locale, UnitSystem};

#[test]
fn convert_returns_metric_mass_in_grams() {
    let service = UnitService::new();
    let response = service
        .convert(&UnitConvertRequest {
            value: 1.0,
            from_unit: "kg".to_owned(),
            to_unit: "g".to_owned(),
        })
        .expect("convert");
    assert_eq!(response.value, 1000.0);
    assert_eq!(response.unit, "g");
    assert_eq!(response.dimension, "mass");
}

#[test]
fn format_uses_requested_unit_system() {
    let service = UnitService::new();
    let response = service
        .format(&UnitFormatRequest {
            value: 236.588_236_5,
            unit: "ml".to_owned(),
            unit_system: UnitSystem::UsCustomary,
            locale: Locale::EnUs,
        })
        .expect("format");
    assert_eq!(response.formatted, "1 cup");
}

#[test]
fn density_registry_is_mutable() {
    let mut service = UnitService::new();
    service.densities_mut().set_override("almond flour", 0.48);
    assert_eq!(
        service.densities().density_g_per_ml("almond flour"),
        Some(0.48)
    );
}
