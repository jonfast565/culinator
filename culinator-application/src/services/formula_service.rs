use crate::{ApplicationError, FormulaRepository};
use culinator_core::{
    Formula, FormulaIngredient, FormulaResult, PercentageConversion, PercentageView, Preferment,
    PrefermentKind, desired_dough_temperature,
};
use culinator_models::{DoughTempRequest, DoughTempResponse, PrefermentBuildRequest};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct FormulaService {
    repository: Arc<dyn FormulaRepository>,
}

impl FormulaService {
    pub fn new(repository: Arc<dyn FormulaRepository>) -> Self {
        Self { repository }
    }

    pub fn calculate(
        &self,
        formula: &Formula,
        target_mass_grams: f64,
    ) -> Result<FormulaResult, ApplicationError> {
        formula
            .solve_for_target_mass(target_mass_grams)
            .map_err(|error| ApplicationError::InvalidInput(error.to_string()))
    }

    pub fn percentages(
        &self,
        formula: &Formula,
        view: PercentageView,
    ) -> Result<PercentageConversion, ApplicationError> {
        formula
            .weights_to_percentages(view)
            .map_err(|error| ApplicationError::InvalidInput(error.to_string()))
    }

    pub fn save(&self, formula: &Formula) -> Result<(), ApplicationError> {
        self.repository.save_formula(formula)
    }

    pub fn get(&self, id: Uuid) -> Result<Formula, ApplicationError> {
        self.repository
            .get_formula(id)?
            .ok_or_else(|| ApplicationError::not_found("formula"))
    }

    pub fn list_for_recipe(&self, recipe_id: Uuid) -> Result<Vec<Formula>, ApplicationError> {
        self.repository.list_formulas_for_recipe(recipe_id)
    }

    pub fn calculate_and_record(
        &self,
        formula_id: Uuid,
        target_mass_grams: f64,
    ) -> Result<FormulaResult, ApplicationError> {
        let formula = self.get(formula_id)?;
        let result = self.calculate(&formula, target_mass_grams)?;
        self.repository
            .save_formula_run(formula_id, target_mass_grams, &result)?;
        Ok(result)
    }

    pub fn build_preferment(
        &self,
        request: PrefermentBuildRequest,
    ) -> Result<Vec<FormulaIngredient>, ApplicationError> {
        let preferment = Preferment {
            kind: PrefermentKind::parse(&request.kind)
                .map_err(|error| ApplicationError::InvalidInput(error.to_string()))?,
            flour_pct: request.flour_pct,
            hydration: request.hydration,
            inoculation: request.inoculation,
            stage: request.stage,
        };
        Ok(preferment.build_stage())
    }

    pub fn dough_temp(
        &self,
        request: DoughTempRequest,
    ) -> Result<DoughTempResponse, ApplicationError> {
        Ok(DoughTempResponse {
            water_temp: desired_dough_temperature(
                request.desired_dough_temp,
                request.friction_factor,
                request.flour_temp,
                request.room_temp,
                request.preferment_temp,
            ),
        })
    }
}

#[cfg(test)]
mod test;
