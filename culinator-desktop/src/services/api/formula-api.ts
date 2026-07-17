import type {
  DoughTempRequest,
  DoughTempResponse,
  Formula,
  FormulaIngredient,
  FormulaResult,
  PercentageConversion,
  PercentageView,
  PrefermentBuildRequest,
} from "../../domain/types";
import { hasConfiguredService, serviceRequest, serviceRpc } from "../transport/websocket-client";

const formulaKey = "culinator.demo.formulas";
function stored(): Formula[] {
  return JSON.parse(localStorage.getItem(formulaKey) ?? "[]") as Formula[];
}
function persist(formulas: Formula[]): void {
  localStorage.setItem(formulaKey, JSON.stringify(formulas));
}

export async function calculateFormula(
  formula: Formula,
  targetMassGrams: number,
): Promise<FormulaResult> {
  if (hasConfiguredService())
    return serviceRequest("/api/v1/formulas/calculate", {
      method: "POST",
      body: JSON.stringify({ formula, targetMassGrams }),
    });
  const fixed = formula.ingredients
    .filter((item) => item.basis === "absolute_mass")
    .reduce((sum, item) => sum + (item.mass_grams ?? 0), 0);
  const totalPct = formula.ingredients
    .filter((item) => item.basis === "percent_of_total")
    .reduce((sum, item) => sum + (item.percentage ?? 0), 0);
  const referencePct = formula.ingredients
    .filter((item) => item.basis === "reference_percent")
    .reduce((sum, item) => sum + (item.percentage ?? 0), 0);
  const referenceMass = referencePct
    ? (targetMassGrams - fixed - (targetMassGrams * totalPct) / 100) / (referencePct / 100)
    : 0;
  const lines = formula.ingredients.map((item) => {
    const mass =
      item.basis === "absolute_mass"
        ? (item.mass_grams ?? 0)
        : item.basis === "percent_of_total"
          ? (targetMassGrams * (item.percentage ?? 0)) / 100
          : (referenceMass * (item.percentage ?? 0)) / 100;
    return {
      ingredient_id: item.id,
      symbol: item.symbol,
      name: item.name,
      stage: item.stage,
      percentage: item.percentage,
      mass_grams: mass,
      is_reference: item.is_reference,
      is_flour: item.is_flour,
      total_percentage: targetMassGrams ? (mass / targetMassGrams) * 100 : 0,
    };
  });
  const flour = lines
    .filter((line) => line.is_flour)
    .reduce((sum, line) => sum + line.mass_grams, 0);
  const water = formula.ingredients.reduce(
    (sum, item, index) => sum + lines[index].mass_grams * item.water_fraction,
    0,
  );
  const prefermented = lines
    .filter((line) => line.is_flour && line.stage !== "final")
    .reduce((sum, line) => sum + line.mass_grams, 0);
  return {
    target_mass_grams: targetMassGrams,
    reference_mass_grams: referenceMass,
    total_flour_grams: flour,
    total_mass_grams: lines.reduce((sum, line) => sum + line.mass_grams, 0),
    hydration_percent: flour ? (water / flour) * 100 : 0,
    prefermented_flour_percent: flour ? (prefermented / flour) * 100 : 0,
    lines,
  };
}
export async function weightsToPercentages(
  formula: Formula,
  view: PercentageView,
): Promise<PercentageConversion> {
  if (hasConfiguredService())
    return serviceRequest("/api/v1/formulas/percentages", {
      method: "POST",
      body: JSON.stringify({ formula, view }),
    });
  const total = formula.ingredients.reduce((sum, item) => sum + (item.mass_grams ?? 0), 0);
  const reference = formula.ingredients
    .filter((item) => item.is_reference)
    .reduce((sum, item) => sum + (item.mass_grams ?? 0), 0);
  const divisor = view === "reference" ? reference : total;
  return {
    view,
    reference_mass_grams: reference,
    total_mass_grams: total,
    lines: formula.ingredients.map((item) => ({
      ingredient_id: item.id,
      symbol: item.symbol,
      name: item.name,
      stage: item.stage,
      percentage: divisor ? ((item.mass_grams ?? 0) / divisor) * 100 : 0,
      mass_grams: item.mass_grams ?? 0,
      is_reference: item.is_reference,
      is_flour: item.is_flour,
      total_percentage: total ? ((item.mass_grams ?? 0) / total) * 100 : 0,
    })),
  };
}
export async function saveFormula(formula: Formula): Promise<Formula> {
  if (hasConfiguredService())
    return serviceRequest("/api/v1/formulas", { method: "PUT", body: JSON.stringify({ formula }) });
  const formulas = stored().filter((item) => item.id !== formula.id);
  persist([...formulas, formula]);
  return formula;
}
export async function listRecipeFormulas(recipeId: string): Promise<Formula[]> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/recipes/${encodeURIComponent(recipeId)}/formulas`);
  return stored().filter((item) => item.recipe_id === recipeId);
}

export async function buildPreferment(
  request: PrefermentBuildRequest,
): Promise<FormulaIngredient[]> {
  if (hasConfiguredService())
    return serviceRpc("formulas.preferment", { ...request } as Record<string, unknown>);
  return [];
}

export async function calculateDoughTemp(request: DoughTempRequest): Promise<DoughTempResponse> {
  if (hasConfiguredService())
    return serviceRpc("formulas.doughTemp", { ...request } as Record<string, unknown>);
  const temps = [
    request.roomTemp,
    request.flourTemp,
    request.frictionFactor,
    request.prefermentTemp ?? request.roomTemp,
  ];
  const factor = request.prefermentTemp == null ? 3 : 4;
  return {
    waterTemp: (factor * request.desiredDoughTemp - temps.reduce((a, b) => a + b, 0)) / 1,
  };
}
