<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { Calculator, Plus, Save, Trash2 } from "lucide-vue-next";
import type {
  Formula,
  FormulaIngredient,
  FormulaResult,
  PercentageView,
} from "../../../domain/types";
import * as api from "../../../services/api";
import UnitConverter from "../../units/components/UnitConverter.vue";

const props = defineProps<{ recipeId: string }>();
const targetMass = ref(1000);
const result = ref<FormulaResult | null>(null);
const error = ref("");
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
function ingredient(name: string, percentage: number, reference = false): FormulaIngredient {
  return {
    id: crypto.randomUUID(),
    symbol: name.toLowerCase().replace(/\W+/g, "_"),
    name,
    stage: "final",
    basis: "reference_percent",
    percentage,
    mass_grams: null,
    is_reference: reference,
    is_flour: reference,
    water_fraction: name.toLowerCase() === "water" ? 1 : 0,
    scalable: true,
    properties: {},
  };
}
const formula = reactive<Formula>({
  id: crypto.randomUUID(),
  recipe_id: props.recipeId,
  symbol: "main_formula",
  name: "Main formula",
  basis: "reference_percent",
  ingredients: [ingredient("Flour", 100, true), ingredient("Water", 68), ingredient("Salt", 2)],
  properties: {},
});
async function calculate(): Promise<void> {
  try {
    error.value = "";
    result.value = await api.calculateFormula(formula, targetMass.value);
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}
async function convert(view: PercentageView): Promise<void> {
  const converted = await api.weightsToPercentages(formula, view);
  converted.lines.forEach((line, index) => {
    formula.ingredients[index].percentage = line.percentage;
  });
}
async function save(): Promise<void> {
  await api.saveFormula(formula);
}
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
function add(): void {
  formula.ingredients.push(ingredient("Ingredient", 0));
}
onMounted(async () => {
  const existing = await api.listRecipeFormulas(props.recipeId);
  if (existing[0]) Object.assign(formula, existing[0]);
  await calculate();
});
</script>
<template>
  <section class="panel formula-panel">
    <header class="panel-header">
      <div>
        <h3><Calculator :size="17" /> Formula calculator</h3>
        <small>Scale recipes and convert weights back to percentages.</small>
      </div>
      <button class="primary" @click="save"><Save :size="15" /> Save</button>
    </header>
    <label>Formula name<input v-model="formula.name" /></label
    ><label
      >Target mass (g)<input v-model.number="targetMass" type="number" min="1" @change="calculate"
    /></label>
    <div class="formula-actions">
      <button @click="convert('reference')">Weights → reference %</button
      ><button @click="convert('total')">Weights → total %</button
      ><button @click="add"><Plus :size="14" /> Add</button>
    </div>
    <div class="formula-table">
      <div class="formula-row header">
        <span>Ingredient</span><span>%</span><span>Mass g</span><span>Ref.</span><span></span>
      </div>
      <div v-for="(item, index) in formula.ingredients" :key="item.id" class="formula-row">
        <input v-model="item.name" /><input
          v-model.number="item.percentage"
          type="number"
          step="0.1"
          @change="calculate"
        /><input v-model.number="item.mass_grams" type="number" step="0.1" /><input
          v-model="item.is_reference"
          type="checkbox"
          @change="calculate"
        /><button
          title="Remove"
          @click="
            formula.ingredients.splice(index, 1);
            calculate();
          "
        >
          <Trash2 :size="14" />
        </button>
      </div>
    </div>
    <button class="primary wide" @click="calculate">Calculate</button>
    <section class="preferment-box">
      <h4>Preferment builder</h4>
      <div class="pref-grid">
        <select v-model="prefermentKind">
          <option value="poolish">Poolish</option>
          <option value="biga">Biga</option>
          <option value="levain">Levain</option>
          <option value="sponge">Sponge</option>
          <option value="soaker">Soaker</option>
          <option value="tangzhong">Tangzhong</option>
        </select>
        <input v-model.number="prefermentFlourPct" type="number" placeholder="Flour %" />
        <input v-model.number="prefermentHydration" type="number" placeholder="Hydration %" />
        <input
          v-model.number="prefermentInoculation"
          type="number"
          step="0.01"
          placeholder="Inoculation"
        />
        <button @click="addPreferment">Add preferment</button>
      </div>
    </section>
    <section class="preferment-box">
      <h4>Dough temperature (DDT)</h4>
      <div class="pref-grid">
        <input v-model.number="ddt.desired" type="number" placeholder="Target °C" />
        <input v-model.number="ddt.friction" type="number" placeholder="Friction" />
        <input v-model.number="ddt.flour" type="number" placeholder="Flour °C" />
        <input v-model.number="ddt.room" type="number" placeholder="Room °C" />
        <button @click="computeWaterTemp">Water temp</button>
      </div>
      <p v-if="waterTemp != null">Water: {{ waterTemp.toFixed(1) }} °C</p>
    </section>
    <UnitConverter />
    <p v-if="error" class="error">{{ error }}</p>
    <dl v-if="result" class="metrics">
      <div>
        <dt>Total</dt>
        <dd>{{ result.total_mass_grams.toFixed(1) }} g</dd>
      </div>
      <div>
        <dt>Hydration</dt>
        <dd>{{ result.hydration_percent.toFixed(1) }}%</dd>
      </div>
      <div v-if="result.effective_hydration_percent != null">
        <dt>Effective hydration</dt>
        <dd>{{ result.effective_hydration_percent.toFixed(1) }}%</dd>
      </div>
      <div>
        <dt>Flour</dt>
        <dd>{{ result.total_flour_grams.toFixed(1) }} g</dd>
      </div>
      <div v-if="result.salt_percent != null">
        <dt>Salt</dt>
        <dd>{{ result.salt_percent.toFixed(2) }}%</dd>
      </div>
      <div v-if="result.fat_percent != null">
        <dt>Fat</dt>
        <dd>{{ result.fat_percent.toFixed(2) }}%</dd>
      </div>
      <div v-if="result.sugar_percent != null">
        <dt>Sugar</dt>
        <dd>{{ result.sugar_percent.toFixed(2) }}%</dd>
      </div>
      <div>
        <dt>Prefermented flour</dt>
        <dd>{{ result.prefermented_flour_percent.toFixed(1) }}%</dd>
      </div>
    </dl>
  </section>
</template>
