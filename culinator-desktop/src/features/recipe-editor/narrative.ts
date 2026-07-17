import { computed, type ComputedRef, type Ref } from "vue";
import type { UiOperation, UiRecipeModel, UiResource } from "./model";
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

export interface RecipeNarrative {
  ingredients: ComputedRef<UiResource[]>;
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
};

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

export function useRecipeNarrative(
  model: Ref<UiRecipeModel> | ComputedRef<UiRecipeModel>,
): RecipeNarrative {
  // Only true ingredients belong on the shopping-style list; intermediate
  // `material` products and equipment/containers are part of the method instead.
  const ingredients = computed(() =>
    model.value.resources.filter((resource) => resource.kind === "ingredient"),
  );

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
    if (operation.produces) parts.push(`makes ${humanize(operation.produces)}`);
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

  return { ingredients, operations, rows, summary, describe, stepTime, stepMeta };
}

function labelMap(model: UiRecipeModel): Map<string, string> {
  const map = new Map<string, string>();
  for (const resource of model.resources) {
    map.set(resource.symbol, resource.name || humanize(resource.symbol));
  }
  return map;
}

/** Human-readable sentence for a step (verb, inputs, heat, doneness). */
export function describeOperation(model: UiRecipeModel, operation: UiOperation): string {
  const labels = labelMap(model);
  const inputNames = operation.inputs
    .map((symbol) => labels.get(symbol) ?? humanize(symbol))
    .filter(Boolean);
  let inputs = "";
  if (inputNames.length === 1) inputs = inputNames[0];
  else if (inputNames.length > 1) {
    inputs = `${inputNames.slice(0, -1).join(", ")} and ${inputNames[inputNames.length - 1]}`;
  }
  const verb = verbs[operation.action] ?? capitalize(humanize(operation.action));
  const sentence = inputs ? `${verb} ${inputs}` : `${verb} ${humanize(operation.symbol)}`;
  let heat = "";
  if (operation.targetTemperature) heat = ` at ${operation.targetTemperature}`;
  else if (operation.heatLevel) heat = ` over ${humanize(operation.heatLevel)} heat`;
  let doneness = "";
  if (operation.doneness?.length) {
    const phrases = operation.doneness.map((cue) =>
      cue.kind === "internal_temp" ? `it reaches ${cue.value} internal` : cue.value,
    );
    doneness = `, until ${phrases.join(" and ")}`;
  }
  return `${sentence}${heat}${doneness}.`;
}

/** First N method steps in dependency order for compact previews. */
export function previewSteps(model: UiRecipeModel, count = 4): string[] {
  return sortOperationsForDisplay(model.operations ?? [])
    .slice(0, count)
    .map((operation) => describeOperation(model, operation));
}
