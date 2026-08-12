<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { Calculator, Plus, RotateCcw, Save, Trash2 } from "lucide-vue-next";
import type { Formula, FormulaIngredient, FormulaResult } from "../../../domain/types";
import type { UiFormula, UiResource } from "../../recipe-editor/model";
import * as api from "../../../services/api";
import UnitConverter from "../../units/components/UnitConverter.vue";
import {
  applyFormulaToSource,
  applyRounding,
  formulaFromUi,
  massForConcentration,
  massForPanVolume,
  massForReferenceFlour,
  massForRoundPan,
  massForServings,
  parseMassGrams,
} from "../formulaSync";
import {
  chooseReference,
  percentagesFromWeights,
  seedFormulaFromRecipe,
  weighedCount,
} from "../seedFromRecipe";

const props = defineProps<{
  recipeId: string;
  recipeTitle?: string;
  resources?: UiResource[];
  formulas?: UiFormula[];
  source?: string;
}>();

const emit = defineEmits<{ "update:source": [value: string] }>();

type Mode = "percent" | "weight";
const mode = ref<Mode>("percent");

type ScaleMode = "mass" | "flour" | "pieces" | "servings" | "pan" | "concentration";
const scaleMode = ref<ScaleMode>("mass");
const targetMass = ref(1000);
const flourMass = ref(500);
const pieceCount = ref(2);
const pieceMassGrams = ref<number | null>(null);
const servingCount = ref(4);
const gramsPerServing = ref(200);
const panDiameterCm = ref(30);
const panDepthCm = ref(0.4);
const panVolumeMl = ref<number | null>(null);
const doughDensity = ref(1.1);
const concentrationSolute = ref("salt");
const concentrationPercent = ref(2);
const roundIncrement = ref(1);
const applyMins = ref(true);

const result = ref<FormulaResult | null>(null);
const error = ref("");
const status = ref("");
const loading = ref(true);

const formula = reactive<Formula>({
  id: crypto.randomUUID(),
  recipe_id: props.recipeId,
  symbol: "dough",
  name: "Dough",
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

function resolvedTargetMass(): number | null {
  if (scaleMode.value === "mass") return targetMass.value > 0 ? targetMass.value : null;
  if (scaleMode.value === "flour") {
    return massForReferenceFlour(flourMass.value, formula.ingredients);
  }
  if (scaleMode.value === "pieces") {
    const per = pieceMassGrams.value;
    if (per == null || !(pieceCount.value > 0)) return null;
    return pieceCount.value * per;
  }
  if (scaleMode.value === "servings") {
    return massForServings(servingCount.value, gramsPerServing.value);
  }
  if (scaleMode.value === "concentration") {
    const solute = formula.ingredients.find((item) => item.symbol === concentrationSolute.value);
    const grams = solute?.mass_grams;
    if (grams == null) return null;
    return massForConcentration(grams, concentrationPercent.value);
  }
  if (panVolumeMl.value != null && panVolumeMl.value > 0) {
    return massForPanVolume(panVolumeMl.value, doughDensity.value);
  }
  return massForRoundPan(panDiameterCm.value, panDepthCm.value, doughDensity.value);
}

function grams(item: FormulaIngredient): number | null {
  if (item.percentage == null) return item.mass_grams ?? null;
  return massByIngredient.value.get(item.id) ?? item.mass_grams ?? null;
}
function bandOf(item: FormulaIngredient): "flour" | "liquid" | "other" {
  if (item.is_flour) return "flour";
  if (item.water_fraction > 0) return "liquid";
  return "other";
}

type Role = "solid" | "flour" | "liquid" | "salt" | "fat" | "sugar";
const ROLE_OPTIONS: Role[] = ["solid", "flour", "liquid", "salt", "fat", "sugar"];

function roleValue(item: FormulaIngredient): Role {
  if (item.is_flour) return "flour";
  if (item.water_fraction > 0) return "liquid";
  const tagged = item.properties?.role;
  if (tagged === "salt" || tagged === "fat" || tagged === "sugar") return tagged;
  return "solid";
}
function sourceHint(item: FormulaIngredient): string | null {
  const declared = item.properties?.sourceQuantity;
  return typeof declared === "string" ? declared : null;
}
function ingredientName(item: FormulaIngredient): string {
  const savedName = item.name.trim();
  if (savedName) return savedName;
  const recipeName = props.resources
    ?.find((resource) => resource.symbol === item.symbol)
    ?.name.trim();
  return recipeName || item.symbol.replaceAll("_", " ");
}
function fillMissingIngredientNames(): void {
  formula.ingredients.forEach((item) => {
    if (!item.name.trim()) item.name = ingredientName(item);
  });
}
function decimal(value: number | null | undefined, places = 1): string {
  if (value == null || !Number.isFinite(value)) return "—";
  return value.toFixed(places).replace(/\.0+$/, "");
}

function isSolvable(item: FormulaIngredient): boolean {
  if (item.basis === "absolute_mass") {
    return item.mass_grams != null && Number.isFinite(item.mass_grams);
  }
  return item.percentage != null && Number.isFinite(item.percentage);
}

async function calculate(): Promise<void> {
  const ready = formula.ingredients.filter(isSolvable);
  if (!ready.length) {
    result.value = null;
    error.value = "";
    return;
  }
  let target: number;
  if (mode.value === "percent") {
    const mass = resolvedTargetMass();
    if (mass == null || mass <= 0) {
      result.value = null;
      return;
    }
    target = mass;
  } else {
    target = ready.reduce((sum, item) => sum + (item.mass_grams ?? 0), 0);
    if (target <= 0) target = targetMass.value;
  }
  try {
    error.value = "";
    result.value = await api.calculateFormula({ ...formula, ingredients: ready }, target);
    if (roundIncrement.value > 0) {
      result.value = applyRounding(result.value, roundIncrement.value);
    }
    if (applyMins.value) {
      // Client-side floor using the same property the core reads.
      result.value = {
        ...result.value,
        lines: result.value.lines.map((line) => {
          const item = formula.ingredients.find((row) => row.id === line.ingredient_id);
          const min =
            typeof item?.properties?.min_mass === "number"
              ? item.properties.min_mass
              : typeof item?.properties?.min_mass_grams === "number"
                ? item.properties.min_mass_grams
                : 0;
          return min > 0 && line.mass_grams > 0 && line.mass_grams < min
            ? { ...line, mass_grams: min }
            : line;
        }),
      };
    }
    if (
      scaleMode.value === "pieces" &&
      pieceCount.value > 0 &&
      pieceMassGrams.value == null &&
      result.value
    ) {
      pieceMassGrams.value = Math.round(result.value.total_mass_grams / pieceCount.value);
    }
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

function ensureReference(): void {
  if (formula.ingredients.some((item) => item.is_reference)) return;
  const referenceRow = chooseReference(formula.ingredients);
  if (referenceRow) referenceRow.is_reference = true;
}

async function syncFromWeights(): Promise<void> {
  ensureReference();
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

async function setScaleMode(next: ScaleMode): Promise<void> {
  scaleMode.value = next;
  await changed();
}

async function makeReference(item: FormulaIngredient): Promise<void> {
  formula.ingredients.forEach((row) => (row.is_reference = row.id === item.id));
  if (formula.ingredients.every((row) => row.mass_grams != null)) {
    percentagesFromWeights(formula.ingredients);
  } else if (item.percentage && item.percentage > 0) {
    const factor = 100 / item.percentage;
    formula.ingredients.forEach((row) => {
      if (row.percentage != null) row.percentage *= factor;
    });
  } else {
    item.percentage = 100;
  }
  await changed();
}

async function setRole(item: FormulaIngredient, role: Role): Promise<void> {
  item.is_flour = role === "flour";
  item.water_fraction = role === "liquid" ? 1 : 0;
  const properties = { ...item.properties };
  if (role === "salt" || role === "fat" || role === "sugar") {
    properties.role = role;
  } else {
    delete properties.role;
  }
  item.properties = properties;
  await changed();
}

function add(): void {
  const number = formula.ingredients.length + 1;
  formula.ingredients.push({
    id: crypto.randomUUID(),
    symbol: `ingredient_${number}`,
    name: `Ingredient ${number}`,
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
  Object.assign(formula, {
    ...seeded,
    id: formula.id,
    symbol: formula.symbol || "dough",
  });
  mode.value = "percent";
  await calculate();
  status.value = `Filled in ${weighedCount(formula)} of ${formula.ingredients.length} weights from the recipe.`;
}

async function applyToRecipe(): Promise<void> {
  if (!props.source) {
    error.value = "No recipe source to update.";
    return;
  }
  await calculate();
  const mass = resolvedTargetMass() ?? result.value?.total_mass_grams;
  if (mass) {
    formula.properties = {
      ...formula.properties,
      target: `${Math.round(mass)} g`,
    };
  }
  try {
    const next = applyFormulaToSource(props.source, { ...formula }, result.value, {
      pieces: scaleMode.value === "pieces" ? pieceCount.value : null,
      pieceMassGrams: scaleMode.value === "pieces" ? pieceMassGrams.value : null,
    });
    emit("update:source", next);
    status.value = "Applied to recipe.";
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

function loadFromUi(ui: UiFormula): void {
  Object.assign(formula, formulaFromUi(ui, props.recipeId, props.resources));
  fillMissingIngredientNames();
  const target = parseMassGrams(ui.target);
  if (target) targetMass.value = Math.round(target);
  if (ui.pieces != null && ui.pieces > 0) {
    scaleMode.value = "pieces";
    pieceCount.value = ui.pieces;
  }
  const pieceMass = parseMassGrams(ui.pieceMass);
  if (pieceMass) pieceMassGrams.value = pieceMass;
  if (ui.doughDensity != null) doughDensity.value = ui.doughDensity;
  if (ui.panDiameter) {
    const match = ui.panDiameter.match(/([0-9]+(?:\.[0-9]+)?)/);
    if (match) {
      panDiameterCm.value = Number(match[1]);
      scaleMode.value = "pan";
    }
  }
  if (ui.panDepth) {
    const match = ui.panDepth.match(/([0-9]+(?:\.[0-9]+)?)/);
    if (match) panDepthCm.value = Number(match[1]);
  }
}

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
  const fromSource = props.formulas?.[0];
  if (fromSource) {
    loadFromUi(fromSource);
    await calculate();
  } else if ((props.resources ?? []).some((resource) => resource.kind === "ingredient")) {
    await reseed();
    status.value = "";
  }
  loading.value = false;
});

watch(
  () => props.formulas?.[0]?.id,
  async (id, previous) => {
    if (!id || id === previous || !props.formulas?.[0]) return;
    loadFromUi(props.formulas[0]);
    await calculate();
  },
);
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
        <button
          class="primary"
          :disabled="!source"
          title="Write formula and scaled amounts into the recipe"
          @click="applyToRecipe"
        >
          <Save :size="15" /> Apply to recipe
        </button>
      </div>
    </header>

    <p v-if="loading" class="empty">Reading the recipe…</p>

    <div v-else-if="!formula.ingredients.length" class="empty-state">
      <p>
        This recipe has no ingredients to weigh yet. Add a row, or declare ingredients in the
        builder first.
      </p>
      <button class="primary" @click="add"><Plus :size="14" /> Add a row</button>
    </div>

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
          :class="['band', bandOf(item)]"
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

      <div v-if="mode === 'percent'" class="scale">
        <div class="mode scale-mode">
          <button :class="{ on: scaleMode === 'mass' }" @click="setScaleMode('mass')">Mass</button>
          <button :class="{ on: scaleMode === 'flour' }" @click="setScaleMode('flour')">
            Flour
          </button>
          <button :class="{ on: scaleMode === 'pieces' }" @click="setScaleMode('pieces')">
            Pieces
          </button>
          <button :class="{ on: scaleMode === 'servings' }" @click="setScaleMode('servings')">
            Servings
          </button>
          <button :class="{ on: scaleMode === 'pan' }" @click="setScaleMode('pan')">Pan</button>
          <button
            :class="{ on: scaleMode === 'concentration' }"
            @click="setScaleMode('concentration')"
          >
            Conc.
          </button>
        </div>
        <label v-if="scaleMode === 'mass'" class="batch">
          Batch size
          <span class="with-unit">
            <input v-model.number="targetMass" type="number" min="1" step="10" @change="changed" />
            <em>g</em>
          </span>
        </label>
        <label v-else-if="scaleMode === 'flour'" class="batch">
          Flour mass
          <span class="with-unit">
            <input v-model.number="flourMass" type="number" min="1" step="10" @change="changed" />
            <em>g</em>
          </span>
        </label>
        <div v-else-if="scaleMode === 'pieces'" class="batch-grid">
          <label class="batch"
            >Pieces
            <input v-model.number="pieceCount" type="number" min="1" step="1" @change="changed" />
          </label>
          <label class="batch"
            >Each
            <span class="with-unit">
              <input
                v-model.number="pieceMassGrams"
                type="number"
                min="1"
                step="1"
                @change="changed"
              />
              <em>g</em>
            </span>
          </label>
        </div>
        <div v-else-if="scaleMode === 'servings'" class="batch-grid">
          <label class="batch"
            >Servings
            <input v-model.number="servingCount" type="number" min="1" step="1" @change="changed" />
          </label>
          <label class="batch"
            >Each
            <span class="with-unit">
              <input
                v-model.number="gramsPerServing"
                type="number"
                min="1"
                step="1"
                @change="changed"
              />
              <em>g</em>
            </span>
          </label>
        </div>
        <div v-else-if="scaleMode === 'concentration'" class="batch-grid">
          <label class="batch"
            >Solute
            <select v-model="concentrationSolute" @change="changed">
              <option v-for="item in formula.ingredients" :key="item.id" :value="item.symbol">
                {{ ingredientName(item) }}
              </option>
            </select>
          </label>
          <label class="batch"
            >% of batch
            <input
              v-model.number="concentrationPercent"
              type="number"
              min="0.1"
              max="99"
              step="0.1"
              @change="changed"
            />
          </label>
        </div>
        <div v-else class="batch-grid">
          <label class="batch"
            >Diameter
            <span class="with-unit">
              <input
                v-model.number="panDiameterCm"
                type="number"
                min="1"
                step="0.5"
                @change="changed"
              />
              <em>cm</em>
            </span>
          </label>
          <label class="batch"
            >Thickness
            <span class="with-unit">
              <input
                v-model.number="panDepthCm"
                type="number"
                min="0.1"
                step="0.1"
                @change="changed"
              />
              <em>cm</em>
            </span>
          </label>
          <label class="batch"
            >Or volume
            <span class="with-unit">
              <input
                v-model.number="panVolumeMl"
                type="number"
                min="1"
                step="10"
                @change="changed"
              />
              <em>ml</em>
            </span>
          </label>
        </div>
        <p class="batch-readout">
          Target <strong>{{ decimal(resolvedTargetMass()) }} g</strong>
        </p>
        <div class="batch-grid rounding-row">
          <label class="batch"
            >Round to
            <span class="with-unit">
              <input
                v-model.number="roundIncrement"
                type="number"
                min="0"
                step="0.1"
                @change="changed"
              />
              <em>g</em>
            </span>
          </label>
          <label class="batch mins-toggle">
            <input v-model="applyMins" type="checkbox" @change="changed" />
            Enforce minimums
          </label>
        </div>
      </div>
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
              :aria-label="`Use ${ingredientName(item)} as the reference`"
              :title="`Use ${ingredientName(item)} as the reference`"
              @change="makeReference(item)"
            />
            <input
              v-model="item.name"
              class="row-name"
              :aria-label="`Ingredient name: ${ingredientName(item)}`"
              :placeholder="ingredientName(item)"
            />
            <button class="icon" :title="`Remove ${ingredientName(item)}`" @click="remove(index)">
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

            <select
              :class="['role-select', roleValue(item)]"
              :value="roleValue(item)"
              :aria-label="`Role of ${item.name || item.symbol}`"
              title="What this ingredient counts as"
              @change="setRole(item, ($event.target as HTMLSelectElement).value as Role)"
            >
              <option v-for="role in ROLE_OPTIONS" :key="role" :value="role">{{ role }}</option>
            </select>
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
        <div v-if="result.salt_percent">
          <dt>Salt</dt>
          <dd>{{ decimal(result.salt_percent, 2) }}%</dd>
        </div>
        <div v-if="result.fat_percent">
          <dt>Fat</dt>
          <dd>{{ decimal(result.fat_percent, 2) }}%</dd>
        </div>
        <div v-if="result.sugar_percent">
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
            <option value="scald">Scald</option>
            <option value="old_dough">Old dough</option>
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
  padding: 7px 9px;
  border-color: #cbd3cd;
  background: #fff;
  font-size: 12px;
  color: #27342d;
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

.scale {
  display: grid;
  gap: 8px;
  margin-bottom: 8px;
}
.scale-mode {
  margin-bottom: 0;
}
.batch-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
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
  padding: 6px 8px;
  border-color: #cbd3cd;
  background: #fff;
  font-size: 13px;
  color: #27342d;
}
.row-name::placeholder {
  color: #657169;
  opacity: 1;
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
.role-select {
  margin-left: auto;
  padding: 2px 6px;
  border: 1px solid #dde1dd;
  border-radius: 999px;
  background: transparent;
  font-size: 10px;
  color: #8a938c;
  text-transform: capitalize;
}
.role-select.flour {
  border-color: transparent;
  background: var(--flour);
  color: white;
}
.role-select.liquid {
  border-color: transparent;
  background: var(--liquid);
  color: white;
}
.role-select.salt,
.role-select.fat,
.role-select.sugar {
  border-color: #c8cdc7;
  color: #4a544d;
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
  margin: 8px 0 0;
  padding: 16px;
  border: 1px dashed #cfd6d0;
  border-radius: 10px;
  background: #fbfcfa;
  font-size: 13px;
  color: #6d7972;
}
.empty-state p {
  margin: 0;
}
.empty-state .primary {
  display: inline-flex;
  align-items: center;
  gap: 5px;
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
  .role-select {
    transition:
      background 0.12s ease,
      color 0.12s ease;
  }
}
</style>
