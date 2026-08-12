/**
 * Bridge between the live FormulaCalculator state and the recipe `.cg` source.
 * The formula block in the recipe is the source of truth; the side-table API is
 * no longer used for persistence.
 */

import type { Formula, FormulaIngredient, FormulaResult } from "../../domain/types";
import type { UiFormula, UiFormulaIngredient, UiResource } from "../recipe-editor/model";
import { emitFormula, emitStatement, type FormulaDraft } from "../recipe-builder/emit";
import {
  applyPatches,
  deleteDeclaration,
  insertDeclaration,
  setStatement,
  type SourcePatch,
} from "../recipe-builder/edits";
import { parseOutline, recipeNode, type OutlineNode } from "../recipe-builder/outline";
import { appendToRecipeBlock } from "../recipe-editor/sourcePatch";

function basisClause(basis: Formula["basis"]): string {
  return basis === "percent_of_total" ? "of total" : "relative to flour";
}

function typeFor(item: FormulaIngredient): string {
  if (item.is_flour) return "Flour<BakersPercent>";
  if (item.water_fraction > 0) return "Liquid<BakersPercent>";
  const role = item.properties?.role;
  if (role === "salt" || role === "fat" || role === "sugar") {
    return "Ingredient<BakersPercent>";
  }
  return "Ingredient<BakersPercent>";
}

function percentageText(value: number | null | undefined): string | undefined {
  if (value == null || !Number.isFinite(value)) return undefined;
  const rounded = Math.round(value * 1000) / 1000;
  return `${rounded}%`;
}

/** Map a WASM-projected formula into the calculator's Formula shape. */
export function formulaFromUi(ui: UiFormula, recipeId: string, resources?: UiResource[]): Formula {
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

function toDraft(formula: Formula): FormulaDraft {
  const target =
    typeof formula.properties?.target === "string"
      ? formula.properties.target
      : formula.properties?.target != null
        ? String(formula.properties.target)
        : undefined;
  return {
    symbol: formula.symbol || "dough",
    basis: basisClause(formula.basis),
    target,
    ingredients: formula.ingredients.map((item) => ({
      symbol: item.symbol,
      type: typeFor(item),
      stage: item.stage || "final",
      baker: percentageText(item.percentage),
    })),
  };
}

function formulaNode(source: string, symbol: string): OutlineNode | undefined {
  const recipe = recipeNode(parseOutline(source));
  return recipe?.children.find((child) => child.keyword === "formula" && child.symbol === symbol);
}

/**
 * Replace or insert the formula block, then patch matching ingredient
 * quantities from a solved batch.
 */
export function applyFormulaToSource(
  source: string,
  formula: Formula,
  result: FormulaResult | null,
  options?: { pieces?: number | null; pieceMassGrams?: number | null },
): string {
  let next = upsertFormulaBlock(source, formula, options);
  if (result) next = patchIngredientQuantities(next, result);
  return next;
}

function upsertFormulaBlock(
  source: string,
  formula: Formula,
  options?: { pieces?: number | null; pieceMassGrams?: number | null },
): string {
  const draft = toDraft(formula);
  const body = emitFormula(draft);
  const existing = formulaNode(source, formula.symbol);
  const recipe = recipeNode(parseOutline(source));
  let next: string;
  if (existing) {
    // Keep leading trivia (blank lines / indent before the block). Replacing
    // `range` and trimStart'ing the body ate the separator and produced
    // `}formula …` — an unclosed-delimiter parse failure after Apply.
    next = `${source.slice(0, existing.codeRange.start)}${body.trimStart()}${source.slice(existing.range.end)}`;
  } else if (recipe) {
    next = insertDeclaration(source, recipe, body, "formula");
  } else {
    next = appendToRecipeBlock(source, body);
  }

  // Re-resolve after every splice — ranges are only valid for the snapshot
  // they came from (see edits.ts).
  if (options?.pieces != null && options.pieces > 0) {
    const node = formulaNode(next, formula.symbol);
    if (node) next = setStatement(next, node, "pieces", `${options.pieces} count`);
  }
  if (options?.pieceMassGrams != null && options.pieceMassGrams > 0) {
    const node = formulaNode(next, formula.symbol);
    if (node) {
      next = setStatement(next, node, "piece_mass", `${Math.round(options.pieceMassGrams)} g`);
    }
  }
  for (const item of formula.ingredients) {
    if (item.is_reference) {
      const ingredient = formulaIngredientNode(next, formula.symbol, item.symbol);
      if (ingredient) next = setStatement(next, ingredient, "reference", "true");
    }
    if (item.is_flour) {
      const ingredient = formulaIngredientNode(next, formula.symbol, item.symbol);
      if (ingredient) next = setStatement(next, ingredient, "flour", "true");
    }
    if (item.water_fraction > 0) {
      const ingredient = formulaIngredientNode(next, formula.symbol, item.symbol);
      if (ingredient) {
        next = setStatement(next, ingredient, "water_fraction", String(item.water_fraction));
      }
    }
    const role = item.properties?.role;
    if (typeof role === "string") {
      const ingredient = formulaIngredientNode(next, formula.symbol, item.symbol);
      if (ingredient) next = setStatement(next, ingredient, "role", role);
    }
  }
  return next;
}

function formulaIngredientNode(
  source: string,
  formulaSymbol: string,
  ingredientSymbol: string,
): OutlineNode | undefined {
  return formulaNode(source, formulaSymbol)?.children.find(
    (child) => child.keyword === "ingredient" && child.symbol === ingredientSymbol,
  );
}

function patchIngredientQuantities(source: string, result: FormulaResult): string {
  const recipe = recipeNode(parseOutline(source));
  if (!recipe) return source;

  // Patch every existing quantity in one end-first pass so earlier offsets
  // stay valid when amounts change length (`7 g` → `11.1 g`).
  const patches: SourcePatch[] = [];
  const missing: { symbol: string; quantity: string }[] = [];
  for (const line of result.lines) {
    if (!Number.isFinite(line.mass_grams) || line.mass_grams <= 0) continue;
    const resource = recipe.children.find(
      (child) => child.keyword === "ingredient" && child.symbol === line.symbol,
    );
    if (!resource) continue;
    const grams = Math.round(line.mass_grams * 10) / 10;
    const quantity = `${grams} g`;
    const existing = resource.children.find((child) => child.keyword === "quantity");
    if (existing) {
      patches.push({
        start: existing.codeRange.start,
        end: existing.codeRange.end,
        replacement: emitStatement("quantity", quantity),
      });
    } else {
      missing.push({ symbol: line.symbol, quantity });
    }
  }

  let next = applyPatches(source, patches);
  for (const item of missing) {
    const recipeNow = recipeNode(parseOutline(next));
    const resource = recipeNow?.children.find(
      (child) => child.keyword === "ingredient" && child.symbol === item.symbol,
    );
    if (resource) next = setStatement(next, resource, "quantity", item.quantity);
  }
  return next;
}

/** Build a minimal dough formula block for a recipe that has none yet. */
export function emitNewFormulaBlock(symbol = "dough"): string {
  return emitFormula({
    symbol,
    basis: "relative to flour",
    target: "1000 g",
    ingredients: [
      {
        symbol: "flour",
        type: "Flour<BakersPercent>",
        stage: "final",
        baker: "100%",
      },
    ],
  });
}

export function removeFormulaBlock(source: string, symbol: string): string {
  const node = formulaNode(source, symbol);
  return node ? deleteDeclaration(source, node) : source;
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
