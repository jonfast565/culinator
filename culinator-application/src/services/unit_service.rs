use crate::{
    ApplicationError, UnitConvertRequest, UnitConvertResponse, UnitFormatRequest,
    UnitFormatResponse,
};
use culinator_core::{
    Dimension, IngredientDensity, Locale as CoreLocale, Quantity, UnitError,
    UnitSystem as CoreUnitSystem, dimension_label, format_quantity,
};

#[derive(Clone, Default)]
pub struct UnitService {
    densities: IngredientDensity,
}

impl UnitService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_densities(densities: IngredientDensity) -> Self {
        Self { densities }
    }

    pub fn densities(&self) -> &IngredientDensity {
        &self.densities
    }

    pub fn densities_mut(&mut self) -> &mut IngredientDensity {
        &mut self.densities
    }

    pub fn convert(
        &self,
        request: &UnitConvertRequest,
    ) -> Result<UnitConvertResponse, ApplicationError> {
        let from_dimension = Dimension::from_unit(&request.from_unit);
        let to_dimension = Dimension::from_unit(&request.to_unit);
        let quantity = Quantity {
            value: request.value,
            unit: request.from_unit.clone(),
            dimension: from_dimension,
        };

        // Volume ↔ mass needs an ingredient density; same-dimension stays exact.
        let converted = if from_dimension == Dimension::Volume && to_dimension == Dimension::Mass {
            let density = self.density_for_convert(request)?;
            let mass = quantity
                .to_mass(density)
                .map_err(unit_error_to_application)?;
            mass.convert_to(&request.to_unit)
                .map_err(unit_error_to_application)?
        } else if from_dimension == Dimension::Mass && to_dimension == Dimension::Volume {
            let density = self.density_for_convert(request)?;
            let volume = quantity
                .to_volume(density)
                .map_err(unit_error_to_application)?;
            volume
                .convert_to(&request.to_unit)
                .map_err(unit_error_to_application)?
        } else {
            quantity
                .convert_to(&request.to_unit)
                .map_err(unit_error_to_application)?
        };

        Ok(UnitConvertResponse {
            value: converted.value,
            unit: converted.unit,
            dimension: dimension_label(converted.dimension).to_owned(),
        })
    }

    fn density_for_convert(&self, request: &UnitConvertRequest) -> Result<f64, ApplicationError> {
        let hint = request
            .ingredient
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| {
                ApplicationError::InvalidInput(
                    "volume/mass conversion needs an ingredient for density".into(),
                )
            })?;
        self.densities.density_g_per_ml(hint).ok_or_else(|| {
            ApplicationError::InvalidInput(format!(
                "no density registered for ingredient `{hint}`"
            ))
        })
    }

    pub fn format(
        &self,
        request: &UnitFormatRequest,
    ) -> Result<UnitFormatResponse, ApplicationError> {
        let quantity = Quantity {
            value: request.value,
            unit: request.unit.clone(),
            dimension: Dimension::from_unit(&request.unit),
        };
        Ok(UnitFormatResponse {
            formatted: format_quantity(
                &quantity,
                to_core_unit_system(request.unit_system),
                to_core_locale(request.locale),
            ),
        })
    }
}

fn to_core_unit_system(system: crate::UnitSystem) -> CoreUnitSystem {
    match system {
        crate::UnitSystem::Metric => CoreUnitSystem::Metric,
        crate::UnitSystem::UsCustomary => CoreUnitSystem::UsCustomary,
    }
}

fn to_core_locale(locale: crate::Locale) -> CoreLocale {
    match locale {
        crate::Locale::EnUs => CoreLocale::EnUs,
        crate::Locale::EnGb => CoreLocale::EnGb,
    }
}

fn unit_error_to_application(error: UnitError) -> ApplicationError {
    ApplicationError::InvalidInput(error.to_string())
}

#[cfg(test)]
#[path = "unit_service/test.rs"]
mod unit_service_test;
