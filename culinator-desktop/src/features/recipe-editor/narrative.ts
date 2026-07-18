import { computed, type ComputedRef, type Ref } from "vue";
import type { UiInputBinding, UiOperation, UiRecipeModel, UiResource } from "./model";
import { sortOperationsForDisplay } from "./operation-order";

export { sortOperationsForDisplay } from "./operation-order";

// Shared recipe-narrative derivation. Both the inspector's RecipeNarrative and
// the full-screen reading page (RecipePage) render from this single source of
// truth: it turns a parsed UiRecipeModel into printable ingredients, numbered
// method rows, and the human-readable prose + time chip for each step.

export interface StepRow {
  kind: "heading" | "step";
  key: string;
  label?: string;
  number?: number;
  operation?: UiOperation;
}

export interface IngredientGroup {
  /** Variant label, e.g. "sweet" — omitted for the base ingredient list. */
  label?: string;
  items: UiResource[];
}

export interface RecipeNarrative {
  ingredients: ComputedRef<UiResource[]>;
  ingredientGroups: ComputedRef<IngredientGroup[]>;
  operations: ComputedRef<UiOperation[]>;
  rows: ComputedRef<StepRow[]>;
  summary: ComputedRef<string>;
  /** Human-readable sentence for a step (verb, inputs, heat, doneness). */
  describe: (operation: UiOperation) => string;
  /** The step's time as a bare chip label ("25 min", "8–10 min", "up to 8 h"). */
  stepTime: (operation: UiOperation) => string;
  /** Secondary line under a step ("hands-on · makes roux"). */
  stepMeta: (operation: UiOperation) => string;
}

function humanize(symbol: string): string {
  return symbol.replaceAll("_", " ").trim();
}

function capitalize(value: string): string {
  return value ? value.charAt(0).toUpperCase() + value.slice(1) : value;
}

const verbs: Record<string, string> = {
  heat: "Heat",
  cook: "Cook",
  bake: "Bake",
  simmer: "Simmer",
  boil: "Boil",
  mix: "Combine",
  combine: "Combine",
  blend: "Blend",
  whisk: "Whisk",
  fold: "Fold",
  rest: "Rest",
  cool: "Cool",
  chill: "Chill",
  cut: "Cut",
  prepare: "Prepare",
  setstate: "Set up",
  pit: "Pit",
  dice: "Dice",
  chop: "Chop",
  mince: "Mince",
  mash: "Mash",
  grate: "Grate",
  drain: "Drain",
  strain: "Strain",
  grease: "Grease",
  coat: "Coat",
  move: "Transfer",
  transfer: "Transfer",
};

/** Generic `does` actions where the operation symbol carries the real verb. */
const GENERIC_ACTIONS = new Set(["mix", "heat", "rest", "move", "strain", "coat", "operation"]);

// Sentence heuristics mirrored from the Rust exporter
// (culinator-export/src/content.rs) so the reading page and the exported
// documents phrase steps the same way.

const PARTICLES = new Set([
  "up",
  "down",
  "in",
  "out",
  "off",
  "on",
  "over",
  "together",
  "back",
  "through",
]);

/** Creation verbs keep their object: "Make roux with the butter". */
const CREATION_VERBS = new Set(["make", "build", "form", "shape", "create", "prepare"]);

/** Verbs that lay later ingredients onto the first one, with their preposition. */
const LAY_ON_PREPOSITION: Record<string, string> = {
  top: "with",
  coat: "with",
  cover: "with",
  garnish: "with",
  sprinkle: "with",
  drizzle: "with",
  brush: "with",
  baste: "with",
  glaze: "with",
  rub: "with",
  spread: "with",
  dust: "with",
  season: "with",
  oil: "with",
  dip: "in",
};

/** Loose word equality that tolerates simple plurals ("pancakes" ~ "pancake"). */
function wordEq(a: string, b: string): boolean {
  const lowerA = a.toLowerCase();
  const lowerB = b.toLowerCase();
  return (
    lowerA === lowerB ||
    `${lowerA}s` === lowerB ||
    `${lowerB}s` === lowerA ||
    `${lowerA}es` === lowerB ||
    `${lowerB}es` === lowerA
  );
}

function anyWordMatches(text: string, word: string): boolean {
  return text.split(/[^a-zA-Z0-9]+/).some((candidate) => candidate && wordEq(candidate, word));
}

const COUNT_UNITS = new Set(["count", "each", "ea"]);

/** Count nouns that pluralize past one ("3 cloves garlic"). */
const COUNT_NOUN_PLURALS: Record<string, string> = {
  clove: "cloves",
  slice: "slices",
  stick: "sticks",
  piece: "pieces",
  cube: "cubes",
  can: "cans",
  sprig: "sprigs",
  stalk: "stalks",
  wedge: "wedges",
  sheet: "sheets",
  scoop: "scoops",
  handful: "handfuls",
  fillet: "fillets",
  strip: "strips",
  ear: "ears",
  head: "heads",
  bulb: "bulbs",
  bunch: "bunches",
  loaf: "loaves",
  leaf: "leaves",
};

/** Cook-style number: quarters render as fractions ("1/4", "1 1/2"). */
function formatNumber(value: number): string {
  const whole = Math.trunc(value);
  const quarters = Math.round((value - whole) * 4);
  if (Math.abs((value - whole) * 4 - quarters) < 1e-9 && quarters > 0 && quarters < 4) {
    const fraction = quarters === 1 ? "1/4" : quarters === 2 ? "1/2" : "3/4";
    return whole >= 1 ? `${whole} ${fraction}` : fraction;
  }
  return String(value);
}

/** The displayed unit, or nothing for bare counters ("2 eggs", not "2 count eggs"). */
function displayUnit(value: number, unit: string): string | undefined {
  const lower = unit.toLowerCase();
  if (COUNT_UNITS.has(lower)) return undefined;
  const plural = COUNT_NOUN_PLURALS[lower];
  if (plural) return value > 1 ? plural : lower;
  return unit;
}

/**
 * Cook-style rendering of a raw source quantity ("0.5 tsp" → "1/2 tsp",
 * "4 count to 5 count" → "4–5"). Unrecognized shapes pass through unchanged.
 * A unit the ingredient name already spells out is dropped ("1 garlic clove").
 */
export function formatQuantity(raw: string | undefined, name?: string): string {
  if (!raw) return "";
  const range = raw.match(/^([\d.]+)\s+(\S+)\s+to\s+([\d.]+)\s+(\S+)$/i);
  if (range && range[2].toLowerCase() === range[4].toLowerCase()) {
    const high = Number(range[3]);
    const numbers = `${formatNumber(Number(range[1]))}–${formatNumber(high)}`;
    const unit = displayUnit(high, range[4]);
    return unit ? `${numbers} ${unit}` : numbers;
  }
  const single = raw.match(/^([\d.]+)\s+(\S+)$/);
  if (single) {
    const value = Number(single[1]);
    if (name && anyWordMatches(name, single[2])) return formatNumber(value);
    const unit = displayUnit(value, single[2]);
    return unit ? `${formatNumber(value)} ${unit}` : formatNumber(value);
  }
  return raw;
}

/** "350 fahrenheit" → "350 °F", "68 celsius" → "68 °C"; anything else as-is. */
function formatTemperature(value: string): string {
  const match = value.trim().match(/^(\d+(?:\.\d+)?)\s*(fahrenheit|f|celsius|c)$/i);
  if (!match) return value;
  const unit = match[2].toLowerCase().startsWith("f") ? "°F" : "°C";
  return `${match[1]} ${unit}`;
}

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

function laborLabel(labor: string): string {
  switch (labor) {
    case "passive":
      return "unattended";
    case "monitor":
      return "keep an eye on it";
    case "automated":
      return "hands-off";
    case "active":
      return "hands-on";
    default:
      return "";
  }
}

/** Printable ingredient line, folding in size/state/notes/to_taste/divided. */
export function formatIngredientDescription(
  ingredient: UiResource,
  formattedQuantity?: string,
): string {
  const qty = formattedQuantity ?? formatQuantity(ingredient.quantity, ingredient.name);
  const parts: string[] = [];
  if (qty) parts.push(qty);
  if (ingredient.size) parts.push(ingredient.size);
  if (ingredient.state) parts.push(ingredient.state);
  parts.push(ingredient.name);
  let line = parts.filter(Boolean).join(" ");
  if (ingredient.divided) {
    line += qty ? ", divided" : " (divided)";
  }
  if (ingredient.notes?.length) {
    for (const note of ingredient.notes) {
      line += `, ${note}`;
    }
  }
  if (ingredient.toTaste) {
    // "Plus more" implies a written base amount; without one the whole
    // quantity is the cook's call.
    line += qty ? ", plus more to taste" : ", to taste";
  }
  if (ingredient.optional) {
    line += " (optional)";
  }
  return line;
}

/** Group variant ingredients under labeled sub-lists; base ingredients come first. */
export function groupIngredients(ingredients: UiResource[]): IngredientGroup[] {
  const base: UiResource[] = [];
  const variants = new Map<string, UiResource[]>();
  for (const ingredient of ingredients) {
    if (ingredient.variant) {
      const list = variants.get(ingredient.variant) ?? [];
      list.push(ingredient);
      variants.set(ingredient.variant, list);
    } else {
      base.push(ingredient);
    }
  }
  const groups: IngredientGroup[] = [];
  if (base.length) groups.push({ items: base });
  for (const [label, items] of variants) {
    groups.push({ label: capitalize(label), items });
  }
  return groups;
}

export function useRecipeNarrative(
  model: Ref<UiRecipeModel> | ComputedRef<UiRecipeModel>,
): RecipeNarrative {
  // Only true ingredients belong on the shopping-style list; intermediate
  // `material` products and equipment/containers are part of the method instead.
  const ingredients = computed(() =>
    model.value.resources.filter((resource) => resource.kind === "ingredient"),
  );

  const ingredientGroups = computed(() => groupIngredients(ingredients.value));

  const operations = computed(() => sortOperationsForDisplay(model.value.operations ?? []));

  function describe(operation: UiOperation): string {
    return describeOperation(model.value, operation);
  }

  // The step's time as a bare chip: a fixed time, an inclusive range
  // (8–10 min), or an open-ended "up to" ceiling. Empty when no duration.
  function stepTime(operation: UiOperation): string {
    const min = formatDuration(operation.durationMinutes);
    if (
      operation.durationMaxMinutes &&
      operation.durationMaxMinutes !== operation.durationMinutes
    ) {
      const max = formatDuration(operation.durationMaxMinutes);
      if (operation.durationMinutes <= 0) return `up to ${max}`;
      // Collapse a shared trailing unit so "8 min"–"10 min" reads "8–10 min".
      const lo = min.match(/^(\d+)\s+(\S+)$/);
      const hi = max.match(/^(\d+)\s+(\S+)$/);
      if (lo && hi && lo[2] === hi[2]) return `${lo[1]}–${hi[1]} ${hi[2]}`;
      return `${min}–${max}`;
    }
    return min;
  }

  function stepMeta(operation: UiOperation): string {
    const parts: string[] = [];
    const labor = laborLabel(operation.labor);
    if (labor) parts.push(labor);
    if (operation.repeat && operation.repeat > 1) {
      parts.push(`repeat ${operation.repeat}×`);
    }
    if (operation.produces) {
      const product = model.value.resources.find(
        (resource) => resource.symbol === operation.produces,
      );
      const productName = humanize(operation.produces);
      // Skip a state the product name already spells out, so
      // "caramelized_onions" doesn't read "caramelized caramelized onions".
      if (product?.state && !productName.toLowerCase().includes(product.state.toLowerCase())) {
        parts.push(`makes ${product.state} ${productName}`);
      } else {
        parts.push(`makes ${productName}`);
      }
    }
    return parts.join(" · ");
  }

  // Flatten operations into printable rows, inserting a subheading each time the
  // method moves into a new process so the printout reads like a recipe card.
  const rows = computed<StepRow[]>(() => {
    const result: StepRow[] = [];
    const distinctProcesses = new Set(operations.value.map((operation) => operation.process));
    const showHeadings = distinctProcesses.size > 1;
    let lastProcess: string | null = null;
    let step = 0;
    for (const operation of operations.value) {
      if (showHeadings && operation.process !== lastProcess) {
        result.push({
          kind: "heading",
          key: `head-${operation.process}-${step}`,
          label: capitalize(humanize(operation.process)),
        });
        lastProcess = operation.process;
      }
      step += 1;
      result.push({ kind: "step", key: operation.symbol, number: step, operation });
    }
    return result;
  });

  const summary = computed(() => {
    const totalMinutes = operations.value.reduce(
      (sum, operation) => sum + (operation.durationMinutes || 0),
      0,
    );
    const parts = [
      `${ingredients.value.length} ingredient${ingredients.value.length === 1 ? "" : "s"}`,
      `${operations.value.length} step${operations.value.length === 1 ? "" : "s"}`,
    ];
    const time = formatDuration(totalMinutes);
    if (time) parts.push(`~${time} total`);
    return parts.join(" · ");
  });

  return {
    ingredients,
    ingredientGroups,
    operations,
    rows,
    summary,
    describe,
    stepTime,
    stepMeta,
  };
}

function labelMap(model: UiRecipeModel): Map<string, string> {
  const map = new Map<string, string>();
  for (const resource of model.resources) {
    map.set(resource.symbol, resource.name || humanize(resource.symbol));
  }
  return map;
}

function stepVerb(operation: UiOperation): string {
  if (GENERIC_ACTIONS.has(operation.action)) {
    return capitalize(humanize(operation.symbol));
  }
  return verbs[operation.action] ?? capitalize(humanize(operation.action));
}

function formatInputLabel(model: UiRecipeModel, binding: UiInputBinding): string {
  const labels = labelMap(model);
  const name = labels.get(binding.symbol) ?? humanize(binding.symbol);
  return binding.quantity ? `${binding.quantity} ${name}` : name;
}

function joinList(items: string[]): string {
  if (items.length === 0) return "";
  if (items.length === 1) return items[0];
  return `${items.slice(0, -1).join(", ")} and ${items[items.length - 1]}`;
}

/** Human-readable sentence for a step (verb, inputs, heat, doneness). */
export function describeOperation(model: UiRecipeModel, operation: UiOperation): string {
  const bindings =
    operation.inputBindings?.length > 0
      ? operation.inputBindings
      : operation.inputs.map((symbol) => ({ symbol }));
  const entries = bindings
    .map((binding) => ({
      label: formatInputLabel(model, binding),
      hasQuantity: Boolean((binding as UiInputBinding).quantity),
    }))
    .filter((entry) => entry.label);

  // Multi-word symbol verbs ("mix_dry", "warm_up", "bake_covered") would
  // repeat or swallow their object; decide what the trailing word is doing
  // before assembling the sentence. Mirrors the Rust exporter.
  let verb = stepVerb(operation);
  let suffix = "";
  let connector = " ";
  const outputName = operation.produces ? humanize(operation.produces) : "";
  const words = verb.split(" ");
  if (words.length > 1 && entries.length > 0) {
    const head = words.slice(0, -1);
    const last = words[words.length - 1];
    const lower = last.toLowerCase();
    const inInputs = entries.some((entry) => anyWordMatches(entry.label, last));
    const inOutput = outputName ? anyWordMatches(outputName, last) : false;
    const creation = CREATION_VERBS.has(head[0].toLowerCase());
    if (lower === "again") {
      verb = head.join(" ");
      suffix = " again";
    } else if (lower === "covered" || lower === "uncovered" || lower === "warm") {
      // A state adverb reads best after the object: "Bake the dough covered".
      verb = head.join(" ");
      suffix = ` ${lower}`;
    } else if (inInputs || (inOutput && !creation)) {
      // The trailing word restates something already in the sentence
      // ("cook_pancakes" over pancake batter).
      verb = head.join(" ");
    } else if (
      PARTICLES.has(lower) ||
      head.some((word) => word.toLowerCase() === "and") ||
      lower.endsWith("ing")
    ) {
      // Phrasal and compound verbs ("warm_up", "rinse_and_dry",
      // "finish_baking") take their objects directly.
    } else {
      connector = " with ";
    }
  }

  let sentence: string;
  if (entries.length === 0) {
    // Avoid echoing the symbol the verb already came from ("Preheat preheat").
    const echo = humanize(operation.symbol);
    sentence = echo.toLowerCase() === verb.toLowerCase() ? verb : `${verb} ${echo}`;
  } else {
    const article = entries[0].hasQuantity ? "" : "the ";
    const layOn = LAY_ON_PREPOSITION[verb.toLowerCase()];
    if (entries.length >= 2 && layOn) {
      // "Top the dish with the panko", "Dip the bread in the custard".
      const additions = entries.slice(1).map((entry) => entry.label);
      sentence = `${verb} ${article}${entries[0].label}${suffix} ${layOn} the ${joinList(additions)}`;
    } else {
      const names = entries.map((entry) => entry.label);
      sentence = `${verb}${connector}${article}${joinList(names)}${suffix}`;
    }
  }
  let heat = "";
  if (operation.targetTemperature) heat = ` at ${formatTemperature(operation.targetTemperature)}`;
  else if (operation.heatLevel) heat = ` over ${humanize(operation.heatLevel)} heat`;
  let doneness = "";
  if (operation.doneness?.length) {
    const phrases = operation.doneness.map((cue) =>
      cue.kind === "internal_temp"
        ? `it reaches ${formatTemperature(cue.value)} internal`
        : cue.value,
    );
    doneness = `, until ${phrases.join(" and ")}`;
  }
  sentence = `${sentence}${heat}${doneness}`;
  if (operation.notes?.length) {
    sentence += `. ${operation.notes.join(" ")}`;
  } else {
    sentence += ".";
  }
  return sentence;
}

/** First N method steps in dependency order for compact previews. */
export function previewSteps(model: UiRecipeModel, count = 4): string[] {
  return sortOperationsForDisplay(model.operations ?? [])
    .slice(0, count)
    .map((operation) => describeOperation(model, operation));
}

/** Compact ingredient lines for book previews and exports. */
export function previewIngredients(model: UiRecipeModel, count = 5): string[] {
  return model.resources
    .filter((resource) => resource.kind === "ingredient")
    .slice(0, count)
    .map((resource) => formatIngredientDescription(resource));
}
