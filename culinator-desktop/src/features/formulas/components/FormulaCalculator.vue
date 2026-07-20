<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { Calculator, Plus, RotateCcw, Save, Trash2 } from "lucide-vue-next";
import type { Formula, FormulaIngredient, FormulaResult } from "../../../domain/types";
import type { UiResource } from "../../recipe-editor/model";
import * as api from "../../../services/api";
import UnitConverter from "../../units/components/UnitConverter.vue";
import { percentagesFromWeights, seedFormulaFromRecipe, weighedCount } from "../seedFromRecipe";

const props = defineProps<{
  recipeId: string;
  recipeTitle?: string;
  resources?: UiResource[];
}>();

/**
 * The two directions a formula is read. Percentages mode scales a known ratio
 * up to a batch size; weights mode takes the weights you actually have and
 * tells you the ratio. Both used to be present but unlabelled — the batch field
 * drove one and two "Weights → …%" buttons drove the other.
 */
type Mode = "percent" | "weight";
const mode = ref<Mode>("percent");

const targetMass = ref(1000);
const result = ref<FormulaResult | null>(null);
const error = ref("");
const status = ref("");
const loading = ref(true);

const formula = reactive<Formula>({
  id: crypto.randomUUID(),
  recipe_id: props.recipeId,
  symbol: "main_formula",
  name: "Main formula",
  basis: "reference_percent",
  ingredients: [],
  properties: {},
});

const reference = computed(() => formula.ingredients.find((item) => item.is_reference) ?? null);
const hasFlour = computed(() => formula.ingredients.some((item) => item.is_flour));
const massByIngredient = computed(() => {
  const map = new Map<string, number>();
  result.value?.lines.forEach((line) => map.set(line.ingredient_id, line.mass_grams));
  return map;
});
const shareByIngredient = computed(() => {
  const map = new Map<string, number>();
  result.value?.lines.forEach((line) => map.set(line.ingredient_id, line.total_percentage));
  return map;
});

function grams(item: FormulaIngredient): number | null {
  // A row with no percentage yet scales to a real 0 g in the result. Report it
  // as unknown instead, so it reads as "still needs a weight" rather than a
  // deliberate zero.
  if (item.percentage == null) return item.mass_grams ?? null;
  return massByIngredient.value.get(item.id) ?? item.mass_grams ?? null;
}
function roleOf(item: FormulaIngredient): "flour" | "liquid" | "other" {
  if (item.is_flour) return "flour";
  if (item.water_fraction > 0) return "liquid";
  return "other";
}
function sourceHint(item: FormulaIngredient): string | null {
  const declared = item.properties?.sourceQuantity;
  return typeof declared === "string" ? declared : null;
}
function decimal(value: number | null | undefined, places = 1): string {
  if (value == null || !Number.isFinite(value)) return "—";
  return value.toFixed(places).replace(/\.0+$/, "");
}

async function calculate(): Promise<void> {
  if (!formula.ingredients.length) {
    result.value = null;
    return;
  }
  try {
    error.value = "";
    result.value = await api.calculateFormula(formula, targetMass.value);
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

/** Weights mode: the weights are authoritative, so restate the ratio from them. */
async function syncFromWeights(): Promise<void> {
  percentagesFromWeights(formula.ingredients);
  const total = formula.ingredients.reduce((sum, item) => sum + (item.mass_grams ?? 0), 0);
  if (total > 0) targetMass.value = Math.round(total);
  await calculate();
}

async function changed(): Promise<void> {
  status.value = "";
  await (mode.value === "weight" ? syncFromWeights() : calculate());
}

async function setMode(next: Mode): Promise<void> {
  mode.value = next;
  await changed();
}

async function makeReference(item: FormulaIngredient): Promise<void> {
  formula.ingredients.forEach((row) => (row.is_reference = row.id === item.id));
  // Percentages are stated against the reference, so moving it restates them.
  if (formula.ingredients.every((row) => row.mass_grams != null)) {
    percentagesFromWeights(formula.ingredients);
  }
  await changed();
}

async function setRole(item: FormulaIngredient, role: "flour" | "liquid"): Promise<void> {
  if (role === "flour") {
    item.is_flour = !item.is_flour;
    if (item.is_flour) item.water_fraction = 0;
  } else {
    item.water_fraction = item.water_fraction > 0 ? 0 : 1;
    if (item.water_fraction > 0) item.is_flour = false;
  }
  await changed();
}

function add(): void {
  formula.ingredients.push({
    id: crypto.randomUUID(),
    symbol: `ingredient_${formula.ingredients.length + 1}`,
    name: "",
    stage: "final",
    basis: "reference_percent",
    percentage: null,
    mass_grams: null,
    is_reference: false,
    is_flour: false,
    water_fraction: 0,
    scalable: true,
    properties: {},
  });
}

async function remove(index: number): Promise<void> {
  const [dropped] = formula.ingredients.splice(index, 1);
  // Every percentage is stated against the reference, so losing it would blank
  // the whole formula. Hand the role to the heaviest row still standing.
  if (dropped?.is_reference && formula.ingredients.length) {
    const heaviest = formula.ingredients.reduce((best, item) =>
      (grams(item) ?? 0) > (grams(best) ?? 0) ? item : best,
    );
    await makeReference(heaviest);
    return;
  }
  await changed();
}

async function reseed(): Promise<void> {
  const seeded = await seedFormulaFromRecipe(
    props.recipeId,
    props.recipeTitle ?? "",
    props.resources ?? [],
  );
  // Keep the saved formula's identity so reseeding updates it rather than
  // orphaning the old row.
  Object.assign(formula, { ...seeded, id: formula.id });
  mode.value = "percent";
  await calculate();
  status.value = `Filled in ${weighedCount(formula)} of ${formula.ingredients.length} weights from the recipe.`;
}

async function save(): Promise<void> {
  try {
    await api.saveFormula(formula);
    status.value = "Saved.";
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

// --- Bread-specific helpers, kept out of the way until asked for -----------
const prefermentKind = ref("poolish");
const prefermentFlourPct = ref(20);
const prefermentHydration = ref(100);
const prefermentInoculation = ref(0.1);
const ddt = ref({
  desired: 24,
  friction: 2,
  flour: 21,
  room: 21,
  preferment: null as number | null,
});
const waterTemp = ref<number | null>(null);

async function addPreferment(): Promise<void> {
  const lines = await api.buildPreferment({
    kind: prefermentKind.value,
    flourPct: prefermentFlourPct.value,
    hydration: prefermentHydration.value,
    inoculation: prefermentInoculation.value,
  });
  formula.ingredients.push(...lines);
  await calculate();
}

async function computeWaterTemp(): Promise<void> {
  const response = await api.calculateDoughTemp({
    desiredDoughTemp: ddt.value.desired,
    frictionFactor: ddt.value.friction,
    flourTemp: ddt.value.flour,
    roomTemp: ddt.value.room,
    prefermentTemp: ddt.value.preferment,
  });
  waterTemp.value = response.waterTemp;
}

onMounted(async () => {
  const existing = await api.listRecipeFormulas(props.recipeId);
  if (existing[0]) {
    Object.assign(formula, existing[0]);
    await calculate();
  } else if ((props.resources ?? []).some((resource) => resource.kind === "ingredient")) {
    await reseed();
    status.value = "";
  }
  loading.value = false;
});
</script>

<template>
  <section class="formula panel">
    <header class="formula-head">
      <div>
        <h3><Calculator :size="17" /> Formula</h3>
        <input v-model="formula.name" class="formula-name" aria-label="Formula name" />
      </div>
      <div class="head-actions">
        <button
          v-if="resources?.length"
          class="ghost"
          title="Rebuild from the recipe's ingredients"
          @click="reseed"
        >
          <RotateCcw :size="14" /> Reset
        </button>
        <button class="primary" @click="save"><Save :size="15" /> Save</button>
      </div>
    </header>

    <p v-if="loading" class="empty">Reading the recipe…</p>

    <p v-else-if="!formula.ingredients.length" class="empty-state">
      This recipe has no ingredients to weigh yet.
      <button class="ghost" @click="add"><Plus :size="14" /> Add a row</button>
    </p>

    <template v-else>
      <!-- The ratio at a glance: every ingredient's share of the batch, keyed
           by the role that drives the metrics below. -->
      <div
        v-if="result"
        class="ribbon"
        role="img"
        :aria-label="`Batch composition: ${result.total_mass_grams.toFixed(0)} grams total`"
      >
        <span
          v-for="item in formula.ingredients"
          :key="item.id"
          :class="['band', roleOf(item)]"
          :style="{ flexGrow: shareByIngredient.get(item.id) ?? 0 }"
          :title="`${item.name || item.symbol} — ${decimal(shareByIngredient.get(item.id))}% of batch`"
        />
      </div>

      <div class="mode">
        <button :class="{ on: mode === 'percent' }" @click="setMode('percent')">
          From percentages
        </button>
        <button :class="{ on: mode === 'weight' }" @click="setMode('weight')">From weights</button>
      </div>

      <label v-if="mode === 'percent'" class="batch">
        Batch size
        <span class="with-unit">
          <input v-model.number="targetMass" type="number" min="1" step="10" @change="changed" />
          <em>g</em>
        </span>
      </label>
      <p v-else class="batch-readout">
        Batch totals <strong>{{ decimal(result?.total_mass_grams) }} g</strong>
      </p>

      <p class="rule">
        Percentages are relative to
        <strong>{{ reference?.name || reference?.symbol || "no reference yet" }}</strong>
        at 100%. Pick a different one with the dot.
      </p>

      <ul class="rows">
        <li v-for="(item, index) in formula.ingredients" :key="item.id" class="row">
          <div class="row-main">
            <input
              class="ref-dot"
              type="radio"
              :checked="item.is_reference"
              :name="`reference-${formula.id}`"
              :aria-label="`Use ${item.name || item.symbol} as the reference`"
              :title="`Use ${item.name || item.symbol} as the reference`"
              @change="makeReference(item)"
            />
            <input v-model="item.name" class="row-name" :placeholder="item.symbol" />
            <button
              class="icon"
              :title="`Remove ${item.name || item.symbol}`"
              @click="remove(index)"
            >
              <Trash2 :size="14" />
            </button>
          </div>
          <div class="row-values">
            <span v-if="mode === 'percent'" class="with-unit editable">
              <input
                v-model.number="item.percentage"
                type="number"
                step="0.1"
                :aria-label="`${item.name || item.symbol} percentage`"
                @change="changed"
              />
              <em>%</em>
            </span>
            <span v-else class="derived">{{ decimal(item.percentage) }} %</span>

            <span v-if="mode === 'weight'" class="with-unit editable">
              <input
                v-model.number="item.mass_grams"
                type="number"
                step="1"
                min="0"
                :aria-label="`${item.name || item.symbol} weight in grams`"
                @change="changed"
              />
              <em>g</em>
            </span>
            <span v-else class="derived">{{ decimal(grams(item)) }} g</span>

            <span class="roles">
              <button
                :class="{ on: item.is_flour }"
                :aria-pressed="item.is_flour"
                title="Counts as flour"
                @click="setRole(item, 'flour')"
              >
                flour
              </button>
              <button
                :class="{ on: item.water_fraction > 0 }"
                :aria-pressed="item.water_fraction > 0"
                title="Counts as liquid"
                @click="setRole(item, 'liquid')"
              >
                liquid
              </button>
            </span>
          </div>
          <p v-if="item.mass_grams == null && sourceHint(item)" class="row-hint">
            Needs a weight — recipe says {{ sourceHint(item) }}
          </p>
        </li>
      </ul>

      <button class="ghost wide" @click="add"><Plus :size="14" /> Add ingredient</button>

      <dl v-if="result" class="metrics">
        <div>
          <dt>Total</dt>
          <dd>{{ decimal(result.total_mass_grams) }} g</dd>
        </div>
        <template v-if="hasFlour">
          <div>
            <dt>Flour</dt>
            <dd>{{ decimal(result.total_flour_grams) }} g</dd>
          </div>
          <div>
            <dt>Hydration</dt>
            <dd>{{ decimal(result.hydration_percent) }}%</dd>
          </div>
          <div v-if="result.effective_hydration_percent != null">
            <dt>Effective hydration</dt>
            <dd>{{ decimal(result.effective_hydration_percent) }}%</dd>
          </div>
          <div v-if="result.prefermented_flour_percent > 0">
            <dt>Prefermented flour</dt>
            <dd>{{ decimal(result.prefermented_flour_percent) }}%</dd>
          </div>
        </template>
        <div v-if="result.salt_percent != null">
          <dt>Salt</dt>
          <dd>{{ decimal(result.salt_percent, 2) }}%</dd>
        </div>
        <div v-if="result.fat_percent != null">
          <dt>Fat</dt>
          <dd>{{ decimal(result.fat_percent, 2) }}%</dd>
        </div>
        <div v-if="result.sugar_percent != null">
          <dt>Sugar</dt>
          <dd>{{ decimal(result.sugar_percent, 2) }}%</dd>
        </div>
      </dl>
    </template>

    <p v-if="status" class="status">{{ status }}</p>
    <p v-if="error" class="error">{{ error }}</p>

    <details class="extra">
      <summary>Preferment builder</summary>
      <div class="extra-body">
        <label
          >Kind
          <select v-model="prefermentKind">
            <option value="poolish">Poolish</option>
            <option value="biga">Biga</option>
            <option value="levain">Levain</option>
            <option value="sponge">Sponge</option>
            <option value="soaker">Soaker</option>
            <option value="tangzhong">Tangzhong</option>
          </select>
        </label>
        <label>Flour <input v-model.number="prefermentFlourPct" type="number" /></label>
        <label>Hydration <input v-model.number="prefermentHydration" type="number" /></label>
        <label
          >Inoculation
          <input v-model.number="prefermentInoculation" type="number" step="0.01" />
        </label>
        <button class="ghost wide" @click="addPreferment">Add preferment stage</button>
      </div>
    </details>

    <details class="extra">
      <summary>Water temperature</summary>
      <div class="extra-body">
        <label>Target dough <input v-model.number="ddt.desired" type="number" /></label>
        <label>Friction <input v-model.number="ddt.friction" type="number" /></label>
        <label>Flour <input v-model.number="ddt.flour" type="number" /></label>
        <label>Room <input v-model.number="ddt.room" type="number" /></label>
        <button class="ghost wide" @click="computeWaterTemp">Calculate water temperature</button>
        <p v-if="waterTemp != null" class="readout">Use water at {{ waterTemp.toFixed(1) }} °C</p>
      </div>
    </details>

    <details class="extra">
      <summary>Unit converter</summary>
      <div class="extra-body">
        <UnitConverter compact />
      </div>
    </details>
  </section>
</template>

<style scoped>
.formula {
  --flour: #c08b2c;
  --liquid: #5d8aa8;
  --other: #a3ada1;
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.formula-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 10px;
}
.formula-head h3 {
  margin-bottom: 4px;
}
.formula-name {
  width: 100%;
  padding: 2px 4px;
  border: 1px solid transparent;
  border-radius: 5px;
  background: transparent;
  font-size: 12px;
  color: #657169;
}
.formula-name:hover {
  border-color: #d9dedb;
}
.head-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

/* Signature: the batch as one proportional strip. */
.ribbon {
  display: flex;
  gap: 2px;
  height: 10px;
  overflow: hidden;
  border-radius: 999px;
  background: #e7e9e5;
}
.band {
  min-width: 2px;
  background: var(--other);
}
.band.flour {
  background: var(--flour);
}
.band.liquid {
  background: var(--liquid);
}

.mode {
  display: flex;
  gap: 3px;
  padding: 3px;
  border-radius: 8px;
  background: #e9ebe7;
}
.mode button {
  flex: 1;
  padding: 6px 8px;
  border-radius: 6px;
  background: transparent;
  font-size: 12px;
  color: #5c6862;
}
.mode button.on {
  background: white;
  font-weight: 600;
  color: #1f2925;
  box-shadow: 0 1px 2px rgb(31 41 37 / 12%);
}

.batch {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin: 0;
}
.batch-readout {
  margin: 0;
  font-size: 13px;
  color: #5c6862;
}
.with-unit {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
}
.with-unit input {
  width: 78px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.with-unit em {
  font-style: normal;
  font-size: 11px;
  color: #8a938c;
}

.rule {
  margin: 0;
  font-size: 12px;
  line-height: 1.5;
  color: #6d7972;
}

.rows {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
}
.row {
  padding: 8px 10px;
  border: 1px solid #e0e4e0;
  border-radius: 8px;
  background: white;
}
.row-main {
  display: flex;
  align-items: center;
  gap: 8px;
}
.ref-dot {
  flex-shrink: 0;
  width: 13px;
  height: 13px;
  accent-color: #8a5e10;
}
.row-name {
  flex: 1;
  min-width: 0;
  padding: 3px 5px;
  border: 1px solid transparent;
  border-radius: 5px;
  background: transparent;
  font-size: 13px;
}
.row-name:hover {
  border-color: #e0e4e0;
}
.icon {
  flex-shrink: 0;
  padding: 3px;
  border-radius: 5px;
  background: transparent;
  color: #97a19a;
}
.icon:hover {
  background: #f2e4e4;
  color: #a93434;
}
.row-values {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 6px;
  padding-left: 21px;
}
.row-values .with-unit input {
  width: 62px;
  padding: 3px 5px;
  font-size: 12px;
}
.derived {
  min-width: 62px;
  font-size: 12px;
  color: #5c6862;
  font-variant-numeric: tabular-nums;
}
.roles {
  display: flex;
  gap: 4px;
  margin-left: auto;
}
.roles button {
  padding: 2px 7px;
  border: 1px solid #dde1dd;
  border-radius: 999px;
  background: transparent;
  font-size: 10px;
  color: #8a938c;
}
.roles button.on {
  border-color: transparent;
  color: white;
}
.roles button:first-child.on {
  background: var(--flour);
}
.roles button:last-child.on {
  background: var(--liquid);
}
.row-hint {
  margin: 6px 0 0;
  padding-left: 21px;
  font-size: 11px;
  color: #97803f;
}

.ghost {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 6px 10px;
  border: 1px solid #d5dad6;
  border-radius: 7px;
  background: white;
  font-size: 12px;
}
.ghost.wide {
  justify-content: center;
  width: 100%;
}
.empty-state {
  display: grid;
  justify-items: start;
  gap: 10px;
  margin: 0;
  font-size: 13px;
  color: #6d7972;
}
.metrics {
  padding: 4px 12px;
  border-radius: 9px;
  background: #f0f2ee;
  font-size: 13px;
}
.metrics dd {
  font-variant-numeric: tabular-nums;
}
.status {
  margin: 0;
  font-size: 12px;
  color: #3f7a52;
}
.error {
  margin: 0;
  font-size: 12px;
  color: #a93434;
}

.extra {
  border-top: 1px solid #e2e6e1;
  padding-top: 10px;
}
.extra summary {
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  color: #5c6862;
}
.extra-body {
  display: grid;
  gap: 8px;
  margin-top: 10px;
}
.extra-body label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin: 0;
}
.extra-body input,
.extra-body select {
  width: 110px;
}
.readout {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
}

@media (prefers-reduced-motion: no-preference) {
  .mode button,
  .roles button {
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }
}
</style>
