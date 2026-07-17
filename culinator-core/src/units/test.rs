use super::*;

fn q(value: f64, unit: &str) -> Quantity {
    Quantity {
        value,
        unit: unit.to_owned(),
        dimension: Dimension::from_unit(unit),
    }
}

#[test]
fn mass_round_trip_kg_to_lb_and_back() {
    let original = q(2.5, "kg");
    let pounds = original.convert_to("lb").expect("kg to lb");
    let back = pounds.convert_to("kg").expect("lb to kg");
    assert!((back.value - original.value).abs() < 1e-9);
}

#[test]
fn volume_round_trip_cup_to_ml_and_back() {
    let original = q(1.0, "cup");
    let ml = original.convert_to("ml").expect("cup to ml");
    assert!((ml.value - 236.588_236_5).abs() < 1e-6);
    let back = ml.convert_to("cup").expect("ml to cup");
    assert!((back.value - original.value).abs() < 1e-9);
}

#[test]
fn temperature_round_trip_f_to_c_and_back() {
    let original = q(350.0, "f");
    let celsius = original.convert_to("c").expect("f to c");
    assert!((celsius.value - 176.666_666_666_666_66).abs() < 1e-6);
    let back = celsius.convert_to("f").expect("c to f");
    assert!((back.value - original.value).abs() < 1e-6);
}

#[test]
fn time_round_trip_minutes_to_seconds() {
    let original = q(15.0, "min");
    let seconds = original.convert_to("s").expect("min to s");
    assert_eq!(seconds.value, 900.0);
}

#[test]
fn length_round_trip_inches_to_mm() {
    let original = q(2.0, "in");
    let mm = original.convert_to("mm").expect("in to mm");
    assert!((mm.value - 50.8).abs() < 1e-9);
}

#[test]
fn volume_to_mass_uses_density() {
    let water = q(250.0, "ml");
    let mass = water.to_mass(1.0).expect("water density");
    assert!((mass.value - 250.0).abs() < 1e-9);
    assert_eq!(mass.unit, "g");
}

#[test]
fn mass_to_volume_uses_density() {
    let flour = q(118.0, "g");
    let volume = flour.to_volume(0.59).expect("flour density");
    assert!((volume.value - 200.0).abs() < 0.01);
    assert_eq!(volume.unit, "ml");
}

#[test]
fn ingredient_density_registry_resolves_builtins_and_overrides() {
    let mut registry = IngredientDensity::new();
    assert_eq!(registry.density_g_per_ml("water"), Some(1.0));
    assert_eq!(registry.density_g_per_ml("flour"), Some(0.59));
    registry.set_override("flour", 0.55);
    assert_eq!(registry.density_g_per_ml("flour"), Some(0.55));
}

#[test]
fn format_quantity_metric_mass_prefers_kilograms() {
    let quantity = q(1500.0, "g");
    assert_eq!(
        format_quantity(&quantity, UnitSystem::Metric, Locale::EnUs),
        "1.5 kg"
    );
}

#[test]
fn format_quantity_us_customary_temperature_uses_fahrenheit() {
    let quantity = q(180.0, "c");
    assert_eq!(
        format_quantity(&quantity, UnitSystem::UsCustomary, Locale::EnUs),
        "356°F"
    );
}

#[test]
fn as_grams_delegates_to_shared_mass_factors() {
    assert_eq!(q(1.0, "kg").as_grams(), Some(1000.0));
    assert_eq!(q(1.0, "pound").as_grams(), Some(453.592_37));
    assert_eq!(q(2.0, "cup").as_grams(), None);
}

#[test]
fn convert_rejects_cross_dimension_units() {
    let mass = q(1.0, "g");
    assert!(matches!(
        mass.convert_to("ml"),
        Err(UnitError::DimensionMismatch { .. })
    ));
}
