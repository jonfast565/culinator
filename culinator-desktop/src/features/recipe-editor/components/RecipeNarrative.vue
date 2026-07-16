<script setup lang="ts">
import { computed } from "vue";
import type { UiOperation, UiRecipeModel } from "../model";

const props = defineProps<{ model: UiRecipeModel }>();

// Only true ingredients belong on the shopping-style list; intermediate
// `material` products and equipment/containers are part of the method instead.
const ingredients = computed(() =>
  props.model.resources.filter((resource) => resource.kind === "ingredient"),
);

const operations = computed(() => props.model.operations ?? []);

// Map every declared resource symbol to a human-readable label so steps can
// refer to ingredients and intermediate products by name rather than by symbol.
const labelFor = computed(() => {
  const map = new Map<string, string>();
  for (const resource of props.model.resources) {
    map.set(resource.symbol, resource.name || humanize(resource.symbol));
  }
  return map;
});

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

function formatDuration(minutes: number): string {
  if (!minutes || minutes <= 0) return "";
  if (minutes < 1) return `${Math.round(minutes * 60)} sec`;
  const total = Math.round(minutes);
  const hours = Math.floor(total / 60);
  const mins = total % 60;
  if (hours && mins) return `${hours} h ${mins} min`;
  if (hours) return `${hours} h`;
  return `${mins} min`;
}

function inputNames(operation: UiOperation): string {
  const names = operation.inputs.map((symbol) => labelFor.value.get(symbol) ?? humanize(symbol));
  if (names.length === 0) return "";
  if (names.length === 1) return names[0];
  return `${names.slice(0, -1).join(", ")} and ${names[names.length - 1]}`;
}

// A temperature setpoint reads "at 350 f"; a stovetop level reads "over medium heat".
function heatClause(operation: UiOperation): string {
  if (operation.targetTemperature) return ` at ${operation.targetTemperature}`;
  if (operation.heatLevel) return ` over ${humanize(operation.heatLevel)} heat`;
  return "";
}

// Render a fixed time, an inclusive range (25 min–30 min), or an open-ended
// "up to" ceiling.
function durationClause(operation: UiOperation): string {
  const min = formatDuration(operation.durationMinutes);
  if (operation.durationMaxMinutes && operation.durationMaxMinutes !== operation.durationMinutes) {
    const max = formatDuration(operation.durationMaxMinutes);
    if (operation.durationMinutes <= 0) return ` for up to ${max}`;
    return ` for ${min}–${max}`;
  }
  return min ? ` for ${min}` : "";
}

// Fold structured doneness cues into "…, until golden brown and it reaches 165 f internal".
function donenessClause(operation: UiOperation): string {
  if (!operation.doneness?.length) return "";
  const phrases = operation.doneness.map((cue) =>
    cue.kind === "internal_temp" ? `it reaches ${cue.value} internal` : cue.value,
  );
  return `, until ${phrases.join(" and ")}`;
}

function describe(operation: UiOperation): string {
  const verb = verbs[operation.action] ?? capitalize(humanize(operation.action));
  const inputs = inputNames(operation);
  const sentence = inputs ? `${verb} ${inputs}` : `${verb} ${humanize(operation.symbol)}`;
  return `${sentence}${heatClause(operation)}${durationClause(operation)}${donenessClause(operation)}.`;
}

function stepMeta(operation: UiOperation): string {
  const parts: string[] = [];
  const labor = laborLabel(operation.labor);
  if (labor) parts.push(labor);
  if (operation.produces) parts.push(`makes ${humanize(operation.produces)}`);
  return parts.join(" · ");
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

function humanizeProcess(symbol: string): string {
  return capitalize(humanize(symbol));
}

// Flatten operations into printable rows, inserting a subheading each time the
// method moves into a new process so the printout reads like a recipe card.
interface StepRow {
  kind: "heading" | "step";
  key: string;
  label?: string;
  number?: number;
  operation?: UiOperation;
}

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
        label: humanizeProcess(operation.process),
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
</script>

<template>
  <section class="panel narrative">
    <header class="narrative-head">
      <h3>{{ model.title || "Untitled recipe" }}</h3>
      <small>{{ summary }}</small>
    </header>

    <div class="narrative-section">
      <h4>Ingredients</h4>
      <ul v-if="ingredients.length" class="ingredient-list">
        <li v-for="ingredient in ingredients" :key="ingredient.symbol">
          <span class="qty">{{ ingredient.quantity || "—" }}</span>
          <span class="name"
            >{{ ingredient.name }}<em v-if="ingredient.optional" class="opt"> (optional)</em></span
          >
        </li>
      </ul>
      <p v-else class="empty">No ingredients yet.</p>
    </div>

    <div class="narrative-section">
      <h4>Method</h4>
      <div v-if="operations.length" class="method">
        <template v-for="row in rows" :key="row.key">
          <h5 v-if="row.kind === 'heading'" class="method-heading">{{ row.label }}</h5>
          <div v-else class="method-step">
            <span class="step-number">{{ row.number }}</span>
            <div class="step-body">
              <p>{{ describe(row.operation!) }}</p>
              <small v-if="stepMeta(row.operation!)">{{ stepMeta(row.operation!) }}</small>
            </div>
          </div>
        </template>
      </div>
      <p v-else class="empty">No steps yet.</p>
    </div>

    <footer v-if="model.attribution || model.source" class="narrative-credit">
      <p v-if="model.attribution">{{ model.attribution }}</p>
      <p v-else>Recipe from {{ model.source }}.</p>
      <a v-if="model.sourceUrl" :href="model.sourceUrl" target="_blank" rel="noopener noreferrer">
        {{ model.sourceUrl }}
      </a>
    </footer>
  </section>
</template>

<style scoped>
.opt {
  font-style: italic;
  opacity: 0.7;
}
</style>
