use crate::{output::OutputFormat, runtime::Runtime};
use anyhow::{Context, Result, bail};
use culinator_models::{
    AutoLinkRequest, CalculateRecipeNutritionRequest, LinkResourceNutritionRequest,
    NewHaccpMonitoringRecord, NewHaccpPlan, NewRecipeTry, NewTryObservation, SaveHaccpPlanRequest,
    SaveIngredientManualNutritionRequest, SaveRecipeNutritionRequest, UpdateRecipeTry,
    UpdateTryOperation,
};
use serde::de::DeserializeOwned;
use std::{fs, path::Path};
use uuid::Uuid;

pub fn nutrition_status(runtime: &Runtime, output: OutputFormat) -> Result<()> {
    output.value(&serde_json::json!({
        "available": runtime.state.nutrition().catalog_available()
    }))
}

pub fn nutrition_search(
    runtime: &Runtime,
    query: &str,
    limit: usize,
    output: OutputFormat,
) -> Result<()> {
    let foods = runtime.state.nutrition().search_foods(query, limit)?;
    output.values(&foods, |food| {
        format!("{}\t{}\t{}", food.fdc_id, food.data_type, food.description)
    })
}

pub fn nutrition_state(runtime: &Runtime, recipe: &str, output: OutputFormat) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    output.value(&runtime.state.nutrition().get_state(recipe.id)?)
}

pub fn nutrition_links(runtime: &Runtime, recipe: &str, output: OutputFormat) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    let links = runtime.state.nutrition().list_links(recipe.id)?;
    output.values(&links, |link| {
        format!(
            "{}\t{}\t{}",
            link.resource_symbol, link.fdc_id, link.food_description
        )
    })
}

pub fn nutrition_link(
    runtime: &Runtime,
    recipe: &str,
    resource_symbol: String,
    fdc_id: i64,
    output: OutputFormat,
) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    output.value(&runtime.state.nutrition().link_resource(
        recipe.id,
        LinkResourceNutritionRequest {
            resource_symbol,
            fdc_id,
        },
    )?)
}

pub fn nutrition_unlink(runtime: &Runtime, recipe: &str, resource_symbol: &str) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    runtime
        .state
        .nutrition()
        .unlink_resource(recipe.id, resource_symbol)?;
    println!("unlinked {resource_symbol}");
    Ok(())
}

pub fn nutrition_auto_link(
    runtime: &Runtime,
    recipe: &str,
    min_score: f64,
    dry_run: bool,
    output: OutputFormat,
) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    output.value(
        &runtime
            .state
            .nutrition()
            .auto_link(recipe.id, AutoLinkRequest { min_score, dry_run })?,
    )
}

pub fn nutrition_calculate(
    runtime: &Runtime,
    recipe: &str,
    servings: Option<f64>,
    serving_size: Option<String>,
    serving_size_grams: Option<f64>,
    output: OutputFormat,
) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    output.value(&runtime.state.nutrition().calculate(
        recipe.id,
        CalculateRecipeNutritionRequest {
            servings_per_container: servings,
            serving_size,
            serving_size_grams,
        },
    )?)
}

pub fn nutrition_save(
    runtime: &Runtime,
    recipe: &str,
    input: &Path,
    output: OutputFormat,
) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    output.value(
        &runtime
            .state
            .nutrition()
            .save_recipe_nutrition(recipe.id, read_json::<SaveRecipeNutritionRequest>(input)?)?,
    )
}

pub fn nutrition_save_manual(
    runtime: &Runtime,
    recipe: &str,
    input: &Path,
    output: OutputFormat,
) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    output.value(&runtime.state.nutrition().save_manual_ingredient(
        recipe.id,
        read_json::<SaveIngredientManualNutritionRequest>(input)?,
    )?)
}

pub fn nutrition_delete_manual(
    runtime: &Runtime,
    recipe: &str,
    resource_symbol: &str,
) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    runtime
        .state
        .nutrition()
        .delete_manual_ingredient(recipe.id, resource_symbol)?;
    println!("deleted manual nutrition for {resource_symbol}");
    Ok(())
}

pub fn haccp_list(runtime: &Runtime, recipe: &str, output: OutputFormat) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    let plans = runtime.state.haccp().list_for_recipe(recipe.id)?;
    output.values(&plans, |plan| {
        format!("{}\t{:?}\t{}", plan.id, plan.status, plan.title)
    })
}

pub fn haccp_get(runtime: &Runtime, plan: Uuid, output: OutputFormat) -> Result<()> {
    output.value(&runtime.state.haccp().get(plan)?)
}

pub fn haccp_create(
    runtime: &Runtime,
    recipe: &str,
    title: String,
    description: Option<String>,
    output: OutputFormat,
) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    output.value(
        &runtime
            .state
            .haccp()
            .create(recipe.id, NewHaccpPlan { title, description })?,
    )
}

pub fn haccp_save(runtime: &Runtime, plan: Uuid, input: &Path, output: OutputFormat) -> Result<()> {
    let request = read_json::<SaveHaccpPlanRequest>(input)?;
    output.value(&runtime.state.haccp().save(plan, request)?)
}

pub fn haccp_record(
    runtime: &Runtime,
    ccp: Uuid,
    input: &Path,
    output: OutputFormat,
) -> Result<()> {
    let request = read_json::<NewHaccpMonitoringRecord>(input)?;
    output.value(&runtime.state.haccp().record_monitoring(ccp, request)?)
}

pub fn haccp_delete(runtime: &Runtime, plan: Uuid, confirmed: bool) -> Result<()> {
    require_confirmation(confirmed)?;
    runtime.state.haccp().delete(plan)?;
    println!("deleted HACCP plan {plan}");
    Ok(())
}

pub fn cook_list(runtime: &Runtime, recipe: &str, output: OutputFormat) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    let tries = runtime.state.kitchen().list_tries(recipe.id)?;
    output.values(&tries, |attempt| {
        format!(
            "{}\t{:?}\t{}",
            attempt.id,
            attempt.status,
            attempt.title.as_deref().unwrap_or("Untitled try")
        )
    })
}

pub fn cook_get(runtime: &Runtime, attempt: Uuid, output: OutputFormat) -> Result<()> {
    output.value(&runtime.state.kitchen().get(attempt)?)
}

pub fn cook_start(
    runtime: &Runtime,
    recipe: &str,
    title: Option<String>,
    notes: Option<String>,
    scale_factor: Option<f64>,
    output: OutputFormat,
) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    output.value(&runtime.state.kitchen().start(
        recipe.id,
        NewRecipeTry {
            title,
            notes,
            scale_factor,
        },
    )?)
}

pub fn cook_update(
    runtime: &Runtime,
    attempt: Uuid,
    input: &Path,
    output: OutputFormat,
) -> Result<()> {
    output.value(
        &runtime
            .state
            .kitchen()
            .update(attempt, read_json::<UpdateRecipeTry>(input)?)?,
    )
}

pub fn cook_update_operation(
    runtime: &Runtime,
    attempt: Uuid,
    operation: Uuid,
    input: &Path,
    output: OutputFormat,
) -> Result<()> {
    output.value(&runtime.state.kitchen().update_operation(
        attempt,
        operation,
        read_json::<UpdateTryOperation>(input)?,
    )?)
}

pub fn cook_observe(
    runtime: &Runtime,
    attempt: Uuid,
    input: &Path,
    output: OutputFormat,
) -> Result<()> {
    output.value(
        &runtime
            .state
            .kitchen()
            .observe(attempt, read_json::<NewTryObservation>(input)?)?,
    )
}

pub fn cook_delete(runtime: &Runtime, attempt: Uuid, confirmed: bool) -> Result<()> {
    require_confirmation(confirmed)?;
    runtime.state.kitchen().delete(attempt)?;
    println!("deleted recipe try {attempt}");
    Ok(())
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))
}

fn require_confirmation(confirmed: bool) -> Result<()> {
    if !confirmed {
        bail!("refusing to delete without --yes");
    }
    Ok(())
}
