/**
 * Helpers for presenting formulas in the calculator. Source planning and edits
 * live in the shared Rust batch planner.
 */

import type { Formula, FormulaIngredient, FormulaResult } from "../../domain/types";
import type { UiFormula, UiFormulaIngredient, UiResource } from "../recipe-editor/model";

/** Map a WASM-projected formula into the calculator's Formula shape. */
export function formulaFromUi(ui: UiFormula, recipeId: string, resources?: UiResource[]): Formula {
  if (ui.raw) return { ...structuredClone(ui.raw), recipe_id: recipeId };
  return {
    id: ui.id || crypto.randomUUID(),
    recipe_id: recipeId,
    symbol: ui.symbol,
    name: ui.name || ui.symbol,
    basis: (ui.basis as Formula["basis"]) || "reference_percent",
    ingredients: ui.ingredients.map((item) => uiIngredientToFormula(item, resources)),
    properties: {
      ...(ui.target ? { target: ui.target } : {}),
      ...(ui.pieces != null ? { pieces: ui.pieces } : {}),
      ...(ui.pieceMass ? { piece_mass: ui.pieceMass } : {}),
      ...(ui.panDiameter ? { pan_diameter: ui.panDiameter } : {}),
      ...(ui.panDepth ? { pan_depth: ui.panDepth } : {}),
      ...(ui.doughDensity != null ? { dough_density: ui.doughDensity } : {}),
    },
  };
}

function uiIngredientToFormula(
  item: UiFormulaIngredient,
  resources?: UiResource[],
): FormulaIngredient {
  const recipeName = resources?.find((resource) => resource.symbol === item.symbol)?.name;
  const role = item.role;
  const properties: Record<string, unknown> = {};
  if (role === "salt" || role === "fat" || role === "sugar") properties.role = role;
  return {
    id: item.id || crypto.randomUUID(),
    symbol: item.symbol,
    name: item.name || recipeName || item.symbol,
    stage: item.stage || "final",
    basis: (item.basis as FormulaIngredient["basis"]) || "reference_percent",
    percentage: item.percentage ?? null,
    mass_grams: item.massGrams ?? null,
    is_reference: item.isReference,
    is_flour: item.isFlour || role === "flour",
    water_fraction: item.waterFraction || (role === "liquid" ? 1 : 0),
    scalable: item.scalable,
    properties,
  };
}

/** Parse a mass like "310 g" into grams. */
export function parseMassGrams(text: string | undefined | null): number | null {
  if (!text) return null;
  const match = String(text)
    .trim()
    .match(/^([0-9]+(?:\.[0-9]+)?)\s*(g|gram|grams|kg)?$/i);
  if (!match) return null;
  const value = Number(match[1]);
  if (!Number.isFinite(value)) return null;
  const unit = (match[2] ?? "g").toLowerCase();
  return unit.startsWith("kg") ? value * 1000 : value;
}

/** Round-pan dough mass: π·(d/2)²·depth_cm · density_g_per_ml. */
export function massForRoundPan(diameterCm: number, depthCm: number, density = 1.1): number | null {
  if (!(diameterCm > 0) || !(depthCm > 0) || !(density > 0)) return null;
  const radius = diameterCm / 2;
  return Math.PI * radius * radius * depthCm * density;
}

export function massForPanVolume(volumeMl: number, density = 1.1): number | null {
  if (!(volumeMl > 0) || !(density > 0)) return null;
  return volumeMl * density;
}

/** Baker's formula: known flour (reference) mass → batch target. */
export function massForReferenceFlour(
  flourGrams: number,
  ingredients: { percentage?: number | null; basis?: string; is_reference?: boolean }[],
): number | null {
  if (!(flourGrams > 0)) return null;
  const referencePct = ingredients
    .filter((item) => item.is_reference && item.basis !== "absolute_mass")
    .reduce((sum, item) => sum + (item.percentage ?? 0), 0);
  const linePct = ingredients
    .filter((item) => item.basis !== "absolute_mass" && item.basis !== "percent_of_total")
    .reduce((sum, item) => sum + (item.percentage ?? 0), 0);
  const members = referencePct > 0 ? referencePct : 100;
  if (!(members > 0) || !(linePct > 0)) return null;
  const referenceBasis = flourGrams / (members / 100);
  return referenceBasis * (linePct / 100);
}

export function massForServings(count: number, gramsEach: number): number | null {
  if (!(count > 0) || !(gramsEach > 0)) return null;
  return count * gramsEach;
}

/** Absolute solute mass at a desired % of total → batch target. */
export function massForConcentration(soluteGrams: number, percentOfTotal: number): number | null {
  if (!(soluteGrams > 0) || !(percentOfTotal > 0) || percentOfTotal >= 100) return null;
  return soluteGrams / (percentOfTotal / 100);
}

export function applyRounding(result: FormulaResult, incrementGrams: number): FormulaResult {
  if (!(incrementGrams > 0)) return result;
  const lines = result.lines.map((line) => ({
    ...line,
    mass_grams: Math.round(line.mass_grams / incrementGrams) * incrementGrams,
  }));
  const total = lines.reduce((sum, line) => sum + line.mass_grams, 0);
  const flour = lines
    .filter((line) => line.is_flour)
    .reduce((sum, line) => sum + line.mass_grams, 0);
  return {
    ...result,
    lines: lines.map((line) => ({
      ...line,
      total_percentage: total > 0 ? (line.mass_grams / total) * 100 : 0,
    })),
    total_mass_grams: total,
    target_mass_grams: total,
    total_flour_grams: flour,
  };
}

/** Heuristic: flour + a liquid suggests a bread/dough formula tool is useful. */
export function looksLikeBreadRecipe(resources: UiResource[], hasFormula: boolean): boolean {
  if (hasFormula) return true;
  const names = resources
    .filter((resource) => resource.kind === "ingredient")
    .map((resource) => `${resource.name} ${resource.symbol}`.toLowerCase());
  const hasFlour = names.some((text) =>
    ["flour", "semolina", "rye", "spelt"].some((word) => text.includes(word)),
  );
  const hasLiquid = names.some((text) =>
    ["water", "milk", "whey", "beer"].some((word) => text.includes(word)),
  );
  return hasFlour && hasLiquid;
}
