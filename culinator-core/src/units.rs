use crate::{Dimension, Quantity, Symbol};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitSystem {
    Metric,
    UsCustomary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    EnUs,
    EnGb,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitError {
    UnknownUnit(Symbol),
    DimensionMismatch { from: Dimension, to: Dimension },
    UnsupportedDimension(Dimension),
    InvalidDensity,
}

impl std::fmt::Display for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownUnit(unit) => write!(f, "unknown unit `{unit}`"),
            Self::DimensionMismatch { from, to } => {
                write!(f, "cannot convert {from:?} to {to:?}")
            }
            Self::UnsupportedDimension(dimension) => {
                write!(f, "conversion is not supported for {dimension:?}")
            }
            Self::InvalidDensity => write!(f, "density must be finite and greater than zero"),
        }
    }
}

impl std::error::Error for UnitError {}

/// Snake-case dimension label for APIs and export.
pub fn dimension_label(dimension: Dimension) -> &'static str {
    match dimension {
        Dimension::Mass => "mass",
        Dimension::Volume => "volume",
        Dimension::Count => "count",
        Dimension::Time => "time",
        Dimension::Temperature => "temperature",
        Dimension::Length => "length",
        Dimension::Area => "area",
        Dimension::Energy => "energy",
        Dimension::Ratio => "ratio",
        Dimension::Concentration => "concentration",
        Dimension::Boolean => "boolean",
        Dimension::Text => "text",
    }
}

/// Normalize a unit token for table lookup (trim, strip trailing period, lowercase).
pub fn normalize_unit(unit: &str) -> String {
    unit.trim().trim_end_matches('.').to_ascii_lowercase()
}

fn mass_to_grams_factor(unit: &str) -> Option<f64> {
    Some(match normalize_unit(unit).as_str() {
        "g" | "gram" | "grams" | "gm" | "gms" => 1.0,
        "kg" | "kilogram" | "kilograms" | "kilo" | "kilos" => 1000.0,
        "mg" | "milligram" | "milligrams" => 0.001,
        "oz" | "ounce" | "ounces" => 28.349_523_125,
        "lb" | "lbs" | "pound" | "pounds" => 453.592_37,
        "dram" | "drams" => 1.771_845_2,
        _ => return None,
    })
}

fn volume_to_ml_factor(unit: &str) -> Option<f64> {
    Some(match normalize_unit(unit).as_str() {
        "ml" | "milliliter" | "milliliters" | "millilitre" | "millilitres" | "cc" => 1.0,
        "cl" | "centiliter" | "centiliters" | "centilitre" | "centilitres" => 10.0,
        "dl" | "deciliter" | "deciliters" | "decilitre" | "decilitres" => 100.0,
        "l" | "liter" | "liters" | "litre" | "litres" => 1000.0,
        "tsp" | "tsps" | "teaspoon" | "teaspoons" => 4.928_921_593_75,
        "tbsp" | "tbsps" | "tbs" | "tablespoon" | "tablespoons" => 14.786_764_781_25,
        "floz" | "fl_oz" | "fluid_ounce" | "fluid_ounces" => 29.573_529_562_5,
        "cup" | "cups" => 236.588_236_5,
        "pt" | "pint" | "pints" => 473.176_473,
        "qt" | "quart" | "quarts" => 946.352_946,
        "gal" | "gallon" | "gallons" => 3785.411_784,
        "dash" | "dashes" => 0.616_115_086_789_1,
        "pinch" | "pinches" => 0.308_057_543_394_5,
        "smidgen" | "smidgens" => 0.078_862_745_098_0,
        "drop" | "drops" => 0.049_289_215_937_5,
        _ => return None,
    })
}

fn length_to_mm_factor(unit: &str) -> Option<f64> {
    Some(match normalize_unit(unit).as_str() {
        "mm" | "millimeter" | "millimeters" | "millimetre" | "millimetres" => 1.0,
        "cm" | "centimeter" | "centimeters" | "centimetre" | "centimetres" => 10.0,
        "m" | "meter" | "meters" | "metre" | "metres" => 1000.0,
        "in" | "inch" | "inches" => 25.4,
        "ft" | "foot" | "feet" => 304.8,
        _ => return None,
    })
}

fn energy_to_kj_factor(unit: &str) -> Option<f64> {
    Some(match normalize_unit(unit).as_str() {
        "j" | "joule" | "joules" => 0.001,
        "kj" | "kilojoule" | "kilojoules" => 1.0,
        "cal" | "calorie" | "calories" => 0.004_184,
        "kcal" | "kilocalorie" | "kilocalories" => 4.184,
        _ => return None,
    })
}

fn temperature_to_celsius(value: f64, unit: &str) -> Result<f64, UnitError> {
    Ok(match normalize_unit(unit).as_str() {
        "c" | "celsius" | "centigrade" => value,
        "f" | "fahrenheit" => (value - 32.0) * 5.0 / 9.0,
        "k" | "kelvin" => value - 273.15,
        _ => return Err(UnitError::UnknownUnit(unit.to_owned())),
    })
}

fn celsius_to_temperature(value: f64, unit: &str) -> Result<f64, UnitError> {
    Ok(match normalize_unit(unit).as_str() {
        "c" | "celsius" | "centigrade" => value,
        "f" | "fahrenheit" => value * 9.0 / 5.0 + 32.0,
        "k" | "kelvin" => value + 273.15,
        _ => return Err(UnitError::UnknownUnit(unit.to_owned())),
    })
}

fn to_canonical(quantity: &Quantity) -> Result<f64, UnitError> {
    match quantity.dimension {
        Dimension::Mass => {
            let factor = mass_to_grams_factor(&quantity.unit)
                .ok_or_else(|| UnitError::UnknownUnit(quantity.unit.clone()))?;
            Ok(quantity.value * factor)
        }
        Dimension::Volume => Ok(quantity.value
            * volume_to_ml_factor(&quantity.unit)
                .ok_or_else(|| UnitError::UnknownUnit(quantity.unit.clone()))?),
        Dimension::Length => Ok(quantity.value
            * length_to_mm_factor(&quantity.unit)
                .ok_or_else(|| UnitError::UnknownUnit(quantity.unit.clone()))?),
        Dimension::Temperature => temperature_to_celsius(quantity.value, &quantity.unit),
        Dimension::Energy => Ok(quantity.value
            * energy_to_kj_factor(&quantity.unit)
                .ok_or_else(|| UnitError::UnknownUnit(quantity.unit.clone()))?),
        Dimension::Time => Ok(quantity.value
            * crate::time_unit_seconds(&quantity.unit)
                .ok_or_else(|| UnitError::UnknownUnit(quantity.unit.clone()))?),
        Dimension::Count if normalize_unit(&quantity.unit) == normalize_unit("each") => {
            Ok(quantity.value)
        }
        _ => Err(UnitError::UnsupportedDimension(quantity.dimension)),
    }
}

fn from_canonical(dimension: Dimension, value: f64, unit: &str) -> Result<Quantity, UnitError> {
    let target_dimension = Dimension::from_unit(unit);
    if target_dimension != dimension {
        return Err(UnitError::DimensionMismatch {
            from: dimension,
            to: target_dimension,
        });
    }

    let converted = match dimension {
        Dimension::Mass => {
            let factor = mass_to_grams_factor(unit)
                .ok_or_else(|| UnitError::UnknownUnit(unit.to_owned()))?;
            value / factor
        }
        Dimension::Volume => {
            let factor =
                volume_to_ml_factor(unit).ok_or_else(|| UnitError::UnknownUnit(unit.to_owned()))?;
            value / factor
        }
        Dimension::Length => {
            let factor =
                length_to_mm_factor(unit).ok_or_else(|| UnitError::UnknownUnit(unit.to_owned()))?;
            value / factor
        }
        Dimension::Temperature => celsius_to_temperature(value, unit)?,
        Dimension::Energy => {
            let factor =
                energy_to_kj_factor(unit).ok_or_else(|| UnitError::UnknownUnit(unit.to_owned()))?;
            value / factor
        }
        Dimension::Time => {
            let factor = crate::time_unit_seconds(unit)
                .ok_or_else(|| UnitError::UnknownUnit(unit.to_owned()))?;
            value / factor
        }
        Dimension::Count if normalize_unit(unit) == normalize_unit("each") => value,
        _ => return Err(UnitError::UnsupportedDimension(dimension)),
    };

    Ok(Quantity {
        value: converted,
        unit: unit.to_owned(),
        dimension,
    })
}

/// Convert a mass quantity to grams when the unit is recognized.
pub fn quantity_as_grams(quantity: &Quantity) -> Option<f64> {
    if quantity.dimension != Dimension::Mass {
        return None;
    }
    mass_to_grams_factor(&quantity.unit).map(|factor| quantity.value * factor)
}

impl Quantity {
    /// Convert this quantity to another unit within the same convertible dimension.
    pub fn convert_to(&self, unit: &str) -> Result<Quantity, UnitError> {
        let target_dimension = Dimension::from_unit(unit);
        if target_dimension != self.dimension {
            return Err(UnitError::DimensionMismatch {
                from: self.dimension,
                to: target_dimension,
            });
        }
        let canonical = to_canonical(self)?;
        from_canonical(self.dimension, canonical, unit)
    }

    /// Convert a volume quantity to mass using density in g/ml, or pass through mass.
    pub fn to_mass(&self, density_g_per_ml: f64) -> Result<Quantity, UnitError> {
        if !density_g_per_ml.is_finite() || density_g_per_ml <= 0.0 {
            return Err(UnitError::InvalidDensity);
        }
        match self.dimension {
            Dimension::Mass => Ok(self.clone()),
            Dimension::Volume => {
                let ml = to_canonical(self)?;
                let grams = ml * density_g_per_ml;
                Ok(Quantity {
                    value: grams,
                    unit: "g".to_owned(),
                    dimension: Dimension::Mass,
                })
            }
            _ => Err(UnitError::DimensionMismatch {
                from: self.dimension,
                to: Dimension::Mass,
            }),
        }
    }

    /// Convert a mass quantity to volume using density in g/ml, or pass through volume.
    pub fn to_volume(&self, density_g_per_ml: f64) -> Result<Quantity, UnitError> {
        if !density_g_per_ml.is_finite() || density_g_per_ml <= 0.0 {
            return Err(UnitError::InvalidDensity);
        }
        match self.dimension {
            Dimension::Volume => Ok(self.clone()),
            Dimension::Mass => {
                let grams = to_canonical(self)?;
                let ml = grams / density_g_per_ml;
                Ok(Quantity {
                    value: ml,
                    unit: "ml".to_owned(),
                    dimension: Dimension::Volume,
                })
            }
            _ => Err(UnitError::DimensionMismatch {
                from: self.dimension,
                to: Dimension::Volume,
            }),
        }
    }
}

/// Built-in and overrideable ingredient densities in g/ml.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IngredientDensity {
    overrides: BTreeMap<String, f64>,
}

impl IngredientDensity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_override(mut self, ingredient: impl Into<String>, density_g_per_ml: f64) -> Self {
        self.overrides.insert(
            normalize_ingredient_key(&ingredient.into()),
            density_g_per_ml,
        );
        self
    }

    pub fn set_override(&mut self, ingredient: impl Into<String>, density_g_per_ml: f64) {
        self.overrides.insert(
            normalize_ingredient_key(&ingredient.into()),
            density_g_per_ml,
        );
    }

    pub fn density_g_per_ml(&self, ingredient: &str) -> Option<f64> {
        let key = normalize_ingredient_key(ingredient);
        if let Some(density) = self.overrides.get(&key) {
            return Some(*density);
        }
        builtin_density(&key)
    }
}

fn normalize_ingredient_key(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace('_', " ")
}

fn builtin_density(key: &str) -> Option<f64> {
    Some(match key {
        "water" => 1.0,
        "flour" | "all purpose flour" | "all-purpose flour" | "ap flour" => 0.59,
        "sugar" | "granulated sugar" | "white sugar" => 0.85,
        "butter" => 0.911,
        _ => return None,
    })
}

fn format_number(value: f64) -> String {
    if (value - value.round()).abs() < 0.001 {
        return format!("{}", value.round() as i64);
    }
    let rounded = (value * 100.0).round() / 100.0;
    let text = format!("{rounded:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn display_unit(unit: &str, locale: Locale) -> String {
    match locale {
        Locale::EnUs => unit.to_owned(),
        Locale::EnGb => match normalize_unit(unit).as_str() {
            "ml" => "ml".to_owned(),
            "l" | "liter" | "liters" => "l".to_owned(),
            "litre" | "litres" => "l".to_owned(),
            other => other.to_owned(),
        },
    }
}

fn metric_mass_unit(grams: f64) -> (&'static str, f64) {
    if grams.abs() >= 1000.0 {
        ("kg", grams / 1000.0)
    } else {
        ("g", grams)
    }
}

fn us_mass_unit(grams: f64) -> (&'static str, f64) {
    let ounces = grams / 28.349_523_125;
    if ounces.abs() >= 16.0 {
        ("lb", ounces / 16.0)
    } else {
        ("oz", ounces)
    }
}

fn metric_volume_unit(ml: f64) -> (&'static str, f64) {
    if ml.abs() >= 1000.0 {
        ("l", ml / 1000.0)
    } else {
        ("ml", ml)
    }
}

fn us_volume_unit(ml: f64) -> (&'static str, f64) {
    const CUP: f64 = 236.588_236_5;
    const TBSP: f64 = 14.786_764_781_25;
    const TSP: f64 = 4.928_921_593_75;
    if ml.abs() >= CUP {
        ("cup", ml / CUP)
    } else if ml.abs() >= TBSP {
        ("tbsp", ml / TBSP)
    } else {
        ("tsp", ml / TSP)
    }
}

fn metric_length_unit(mm: f64) -> (&'static str, f64) {
    if mm.abs() >= 1000.0 {
        ("m", mm / 1000.0)
    } else if mm.abs() >= 10.0 {
        ("cm", mm / 10.0)
    } else {
        ("mm", mm)
    }
}

fn us_length_unit(mm: f64) -> (&'static str, f64) {
    let inches = mm / 25.4;
    if inches.abs() >= 12.0 {
        ("ft", inches / 12.0)
    } else {
        ("in", inches)
    }
}

fn format_for_system(quantity: &Quantity, system: UnitSystem) -> Result<(f64, String), UnitError> {
    let canonical = to_canonical(quantity)?;
    let (unit, value) = match (system, quantity.dimension) {
        (UnitSystem::Metric, Dimension::Mass) => metric_mass_unit(canonical),
        (UnitSystem::UsCustomary, Dimension::Mass) => us_mass_unit(canonical),
        (UnitSystem::Metric, Dimension::Volume) => metric_volume_unit(canonical),
        (UnitSystem::UsCustomary, Dimension::Volume) => us_volume_unit(canonical),
        (UnitSystem::Metric, Dimension::Length) => metric_length_unit(canonical),
        (UnitSystem::UsCustomary, Dimension::Length) => us_length_unit(canonical),
        (UnitSystem::Metric, Dimension::Temperature) => ("c", canonical),
        (UnitSystem::UsCustomary, Dimension::Temperature) => {
            ("f", celsius_to_temperature(canonical, "f")?)
        }
        (UnitSystem::Metric, Dimension::Time) => {
            if canonical >= 3600.0 && (canonical % 3600.0).abs() < 0.001 {
                ("h", canonical / 3600.0)
            } else if canonical >= 60.0 && (canonical % 60.0).abs() < 0.001 {
                ("min", canonical / 60.0)
            } else {
                ("s", canonical)
            }
        }
        (UnitSystem::UsCustomary, Dimension::Time) => {
            if canonical >= 3600.0 && (canonical % 3600.0).abs() < 0.001 {
                ("h", canonical / 3600.0)
            } else if canonical >= 60.0 && (canonical % 60.0).abs() < 0.001 {
                ("min", canonical / 60.0)
            } else {
                ("s", canonical)
            }
        }
        (_, Dimension::Energy) => ("kj", canonical),
        _ => {
            return Ok((quantity.value, quantity.unit.clone()));
        }
    };
    Ok((value, unit.to_owned()))
}

/// Render a quantity using the requested unit system and locale.
pub fn format_quantity(quantity: &Quantity, system: UnitSystem, locale: Locale) -> String {
    match format_for_system(quantity, system) {
        Ok((value, unit)) => {
            let suffix = match quantity.dimension {
                Dimension::Temperature => match normalize_unit(&unit).as_str() {
                    "c" => "°C".to_owned(),
                    "f" => "°F".to_owned(),
                    "k" => " K".to_owned(),
                    _ => format!(" {}", display_unit(&unit, locale)),
                },
                _ => format!(" {}", display_unit(&unit, locale)),
            };
            format!("{}{suffix}", format_number(value))
        }
        Err(_) => format!(
            "{} {}",
            format_number(quantity.value),
            display_unit(&quantity.unit, locale)
        ),
    }
}

#[cfg(test)]
#[path = "units/test.rs"]
mod units_test;
