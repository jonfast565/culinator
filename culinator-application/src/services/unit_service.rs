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
        let quantity = Quantity {
            value: request.value,
            unit: request.from_unit.clone(),
            dimension: from_dimension,
        };
        let converted = quantity
            .convert_to(&request.to_unit)
            .map_err(unit_error_to_application)?;
        Ok(UnitConvertResponse {
            value: converted.value,
            unit: converted.unit,
            dimension: dimension_label(converted.dimension).to_owned(),
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
