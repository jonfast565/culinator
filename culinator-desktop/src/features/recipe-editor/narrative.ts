import { computed, type ComputedRef, type Ref, unref } from "vue";
import {
  narrativeWasm,
  type WasmNumberStyle,
  type WasmUnitSystem,
} from "../../services/wasm/parser";

export { sortOperationsForDisplay } from "./operation-order";

/**
 * Recipe narrative for the reading page.
 *
 * Every sentence, ingredient line, time chip, and mise-en-place list here is
 * rendered by `culinator-narrative` (via WASM) — the same generator the EPUB,
 * print, and plain-text exporters use. This file used to derive all of it in
 * TypeScript, which meant the heuristics existed twice and drifted: the reading
 * page dropped step destinations ("Transfer the sauce mix." instead of "…into
 * the casserole") and rendered "0.5 tsp" where the exporter said "1/2 tsp".
 *
 * Amounts are converted and formatted in Rust too, so the page no longer makes
 * an async round-trip per quantity.
 */

/** An ingredient as displayed, split so a UI can column-align the amount. */
export interface IngredientDisplayParts {
  /** Resource symbol, for views that key rows by declaration. */
  symbol: string;
  /** "3 tbsp", "1/2", or empty when the amount is the cook's call. */
  amount: string;
  /** Size, state, and name: "large Hass avocados". */
  description: string;
  /** Trailing modifiers: divided, notes, to taste, optional. */
  aside?: string;
}

export interface IngredientGroup {
  /** Variant label, e.g. "Sweet" — omitted for the base ingredient list. */
  label?: string;
  items: IngredientDisplayParts[];
}

/** What one method section needs on hand before its steps start. */
export interface SectionMise {
  ingredients: IngredientDisplayParts[];
  equipment: string[];
}

export interface NarrativeStep {
  /** Operation symbol, so the editor can act on the right declaration. */
  symbol: string;
  number: number;
  /** Full instruction prose, equipment and doneness woven in. */
  text: string;
  time?: string;
  /** "hands-on · makes roux". */
  meta?: string;
  tools: string[];
}

export interface MethodSection {
  process: string;
  /** Humanized heading; absent when the recipe has a single process. */
  title?: string;
  /** Parallelism guidance for the section as a whole. */
  note?: string;
  steps: NarrativeStep[];
  mise: SectionMise;
}

export interface RecipeNarrative {
  summary: ComputedRef<string>;
  ingredientGroups: ComputedRef<IngredientGroup[]>;
  /** Whole-recipe equipment for the traditional top-matter layout. */
  equipment: ComputedRef<string[]>;
  sections: ComputedRef<MethodSection[]>;
  /** Flat step list, for layouts that number straight through. */
  steps: ComputedRef<NarrativeStep[]>;
}

export interface NarrativeOptions {
  unitSystem?: Ref<WasmUnitSystem> | ComputedRef<WasmUnitSystem>;
  numberStyle?: Ref<WasmNumberStyle> | ComputedRef<WasmNumberStyle>;
}

// The WASM boundary hands back plain JSON; these mirror its serde shape.
interface WasmLine {
  symbol: string;
  quantity: string;
  description: string;
  aside?: string;
}
interface WasmNarrative {
  summary: string;
  ingredientGroups: { label?: string; items: WasmLine[] }[];
  equipment: string[];
  sections: {
    process: string;
    title?: string;
    note?: string;
    steps: NarrativeStep[];
    mise: { ingredients: WasmLine[]; equipment: string[] };
  }[];
}

function toParts(line: WasmLine): IngredientDisplayParts {
  return {
    symbol: line.symbol,
    amount: line.quantity,
    description: line.description,
    aside: line.aside,
  };
}

/** Rendered ingredient lines keyed by resource symbol. */
export function ingredientPartsBySymbol(source: string): Map<string, IngredientDisplayParts> {
  return new Map(
    buildNarrative(source)
      .ingredientGroups.flatMap((group) => group.items)
      .map((item) => [item.symbol, item]),
  );
}

export interface BuiltNarrative {
  summary: string;
  ingredientGroups: IngredientGroup[];
  equipment: string[];
  sections: MethodSection[];
}

/** Render a recipe's narrative in one call. */
export function buildNarrative(
  source: string,
  unitSystem: WasmUnitSystem = "as_authored",
  numberStyle: WasmNumberStyle = "fractions",
): BuiltNarrative {
  const raw = narrativeWasm(source, unitSystem, numberStyle) as WasmNarrative;
  return {
    summary: raw.summary,
    ingredientGroups: raw.ingredientGroups.map((group) => ({
      label: group.label,
      items: group.items.map(toParts),
    })),
    equipment: raw.equipment,
    sections: raw.sections.map((section) => ({
      process: section.process,
      title: section.title,
      note: section.note,
      steps: section.steps,
      mise: {
        ingredients: section.mise.ingredients.map(toParts),
        equipment: section.mise.equipment,
      },
    })),
  };
}

export function useRecipeNarrative(
  source: Ref<string> | ComputedRef<string>,
  options: NarrativeOptions = {},
): RecipeNarrative {
  const narrative = computed(() =>
    buildNarrative(
      unref(source) ?? "",
      options.unitSystem ? unref(options.unitSystem) : "as_authored",
      options.numberStyle ? unref(options.numberStyle) : "fractions",
    ),
  );

  return {
    summary: computed(() => narrative.value.summary),
    ingredientGroups: computed(() => narrative.value.ingredientGroups),
    equipment: computed(() => narrative.value.equipment),
    sections: computed(() => narrative.value.sections),
    steps: computed(() => narrative.value.sections.flatMap((section) => section.steps)),
  };
}

/** First N method steps, for compact book previews. */
export function previewSteps(source: string, count = 4): string[] {
  return buildNarrative(source)
    .sections.flatMap((section) => section.steps)
    .slice(0, count)
    .map((step) => step.text);
}

/** Compact ingredient lines for book previews. */
export function previewIngredientParts(source: string, count = 5): IngredientDisplayParts[] {
  return buildNarrative(source)
    .ingredientGroups.flatMap((group) => group.items)
    .slice(0, count);
}

/** The recipe's "10 ingredients · 9 steps · ~2 h" line. */
export function narrativeSummary(source: string): string {
  return buildNarrative(source).summary;
}

/**
 * Durations in minutes → "1 h 20 min". Kept in TypeScript because the
 * scheduling views compute their own totals from live timer state rather than
 * from a parsed recipe.
 */
export function formatDuration(minutes: number): string {
  if (!minutes || minutes <= 0) return "";
  if (minutes < 1) return `${Math.round(minutes * 60)} sec`;
  const total = Math.round(minutes);
  const hours = Math.floor(total / 60);
  const mins = total % 60;
  if (hours && mins) return `${hours} h ${mins} min`;
  if (hours) return `${hours} h`;
  return `${mins} min`;
}
