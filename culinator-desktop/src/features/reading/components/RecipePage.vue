<script setup lang="ts">
/* global CSS */
import { computed, inject, nextTick, toRef, watch } from "vue";
import { CheckCircle2, ChefHat } from "lucide-vue-next";
import type { TryOperation } from "../../../domain/types";
import {
  useKitchenExecution,
  type LiveTimerState,
} from "../../kitchen-mode/composables/useKitchenExecution";
import type { UiOperation, UiRecipeModel } from "../../recipe-editor/model";
import { useRecipeNarrative, type NarrativeStep } from "../../recipe-editor/narrative";
import { deleteOperationFromSource } from "../../recipe-editor/sourcePatch";
import { UNIT_DISPLAY_KEY } from "../../units/composables/useUnitDisplay";
import { useAppDialog } from "../../../shared/composables/useAppDialog";
import { VIEW_SETTINGS_KEY } from "../composables/useViewSettings";
import type { IndexCardFormat } from "../indexCardFormat";
import { collectRecipeAllergens } from "../allergens";
import AllergenBadges from "./AllergenBadges.vue";
import SubstitutionAssistant from "./SubstitutionAssistant.vue";
import RecipeImage from "./RecipeImage.vue";
import MiseBlock from "./MiseBlock.vue";
import IngredientGroupList from "./IngredientGroupList.vue";
import RecipeStepRow from "./RecipeStepRow.vue";
import IndexCardRecipe from "./IndexCardRecipe.vue";

const props = defineProps<{
  model: UiRecipeModel;
  source: string;
  recipeId?: string;
  editable?: boolean;
  kitchenMode?: boolean;
  /** Symbol currently focused in the builder — preview rows pulse to match. */
  highlightedSymbol?: string | null;
}>();

const emit = defineEmits<{
  "update:source": [value: string];
  "kitchen-finished": [];
  "select-symbol": [symbol: string];
}>();

const dialog = useAppDialog();
const units = inject(UNIT_DISPLAY_KEY, null);
const viewSettings = inject(VIEW_SETTINGS_KEY, null);

// Prose, amounts, times, and mise all come from the shared Rust generator, in
// the reader's chosen units and number style. Nothing is derived here.
const { summary, ingredientGroups, equipment, sections } = useRecipeNarrative(
  toRef(props, "source"),
  {
    unitSystem: computed(() => units?.unitSystem.value ?? "as_authored"),
    temperatureScale: computed(() => units?.temperatureScale.value ?? "as_authored"),
    numberStyle: computed(() => viewSettings?.numberStyle.value ?? "fractions"),
    decimalPlaces: computed(() => viewSettings?.decimalPlaces.value ?? 2),
  },
);

const colocated = computed(() => viewSettings?.misePlacement.value === "colocated");
const indexCardFormat = computed<IndexCardFormat>(
  () => viewSettings?.indexCardFormat.value ?? "full",
);
const cardFormat = computed(() => {
  const format = indexCardFormat.value;
  return format === "full" ? null : format;
});
const hasSteps = computed(() => sections.value.some((section) => section.steps.length > 0));
const allergens = computed(() => collectRecipeAllergens(props.model));
const bakerBySymbol = computed(() => {
  const map: Record<string, number> = {};
  for (const formula of props.model.formulas ?? []) {
    for (const item of formula.ingredients) {
      if (item.percentage != null) map[item.symbol] = item.percentage;
    }
  }
  return map;
});
const formulaMetrics = computed(() => {
  const formula = props.model.formulas?.[0];
  if (!formula) return null;
  const flour = formula.ingredients
    .filter((item) => item.isFlour)
    .reduce((sum, item) => sum + (item.percentage ?? 0), 0);
  if (!(flour > 0)) return null;
  const water = formula.ingredients.reduce(
    (sum, item) => sum + (item.percentage ?? 0) * (item.waterFraction || 0),
    0,
  );
  const salt = formula.ingredients
    .filter((item) => item.role === "salt")
    .reduce((sum, item) => sum + (item.percentage ?? 0), 0);
  return {
    hydration: Math.round((water / flour) * 1000) / 10,
    salt: salt > 0 ? Math.round((salt / flour) * 1000) / 10 : null,
    pieces: formula.pieces ?? null,
  };
});
const {
  activeTry,
  error: kitchenError,
  liveTimers,
  refresh: refreshKitchen,
  startOperation,
  completeOperation,
  completeTry,
  startClock,
} = useKitchenExecution(toRef(props, "recipeId"));

watch(
  [() => props.kitchenMode, () => props.recipeId],
  async ([enabled]) => {
    if (!enabled) return;
    try {
      await refreshKitchen();
      if (activeTry.value?.status === "active") startClock();
    } catch (cause) {
      kitchenError.value = cause instanceof Error ? cause.message : String(cause);
    }
  },
  { immediate: true },
);

/** The parsed operation behind a step, for its photo and for source patching. */
function operationFor(step: NarrativeStep): UiOperation | undefined {
  return props.model.operations?.find((operation) => operation.symbol === step.symbol);
}

function kitchenOperationFor(step: NarrativeStep): TryOperation | undefined {
  if (!props.kitchenMode) return undefined;
  return activeTry.value?.operations.find((operation) => operation.operationSymbol === step.symbol);
}

function kitchenTimerFor(step: NarrativeStep): LiveTimerState | undefined {
  const operation = kitchenOperationFor(step);
  return operation ? liveTimers.value[operation.operationId] : undefined;
}

async function runKitchenAction(action: () => Promise<void>): Promise<void> {
  kitchenError.value = "";
  try {
    await action();
  } catch (cause) {
    kitchenError.value = cause instanceof Error ? cause.message : String(cause);
  }
}

async function finishKitchen(): Promise<void> {
  await runKitchenAction(completeTry);
  if (!kitchenError.value) emit("kitchen-finished");
}

async function removeStep(step: NarrativeStep): Promise<void> {
  const operation = operationFor(step);
  if (!props.editable || !operation) return;
  if (!(await dialog.confirm(`Delete this step?\n\n${step.text}`))) return;
  const next = deleteOperationFromSource(props.source, operation);
  if (next != null) emit("update:source", next);
}

const eyebrow = computed(() => props.model.attribution || props.model.source || "Recipe");

const kitchenProgress = computed(() => {
  const ops = activeTry.value?.operations;
  if (!ops?.length) return "";
  const done = ops.filter(
    (operation) => operation.status === "completed" || operation.status === "skipped",
  ).length;
  return `${done} of ${ops.length} steps done`;
});

const kitchenProgressRatio = computed(() => {
  const ops = activeTry.value?.operations;
  if (!ops?.length) return null;
  const done = ops.filter(
    (operation) => operation.status === "completed" || operation.status === "skipped",
  ).length;
  return done / ops.length;
});

watch(
  () => props.highlightedSymbol,
  (symbol) => {
    if (!symbol) return;
    void nextTick(() => {
      document
        .querySelector(`[data-preview-symbol="${CSS.escape(symbol)}"]`)
        ?.scrollIntoView({ behavior: "smooth", block: "nearest" });
    });
  },
);
</script>

<template>
  <IndexCardRecipe
    v-if="cardFormat && !kitchenMode"
    :format="cardFormat"
    :model="model"
    :recipe-id="recipeId"
    :editable="editable"
    :highlighted-symbol="highlightedSymbol"
    :summary="summary"
    :eyebrow="eyebrow"
    :allergens="allergens"
    :ingredient-groups="ingredientGroups"
    :equipment="equipment"
    :sections="sections"
    :operation-for="operationFor"
    @select-symbol="emit('select-symbol', $event)"
    @delete="removeStep"
  />
  <article v-else class="leaf" :class="{ 'kitchen-leaf': kitchenMode }">
    <div v-if="kitchenMode" class="kitchen-strip">
      <div class="kitchen-progress">
        <span><ChefHat :size="15" /> Kitchen mode</span>
        <small v-if="kitchenProgress">{{ kitchenProgress }}</small>
        <div v-if="kitchenProgressRatio != null" class="progress-track" aria-hidden="true">
          <div class="progress-fill" :style="{ width: `${kitchenProgressRatio * 100}%` }"></div>
        </div>
      </div>
      <button type="button" class="finish-kitchen" :disabled="!activeTry" @click="finishKitchen">
        <CheckCircle2 :size="14" /> Finish cooking
      </button>
    </div>
    <p v-if="kitchenMode && kitchenError" class="kitchen-error">{{ kitchenError }}</p>
    <figure v-if="model.coverImage" class="leaf-cover">
      <RecipeImage :image-ref="model.coverImage" :recipe-id="recipeId" :alt="model.title" />
    </figure>
    <header class="leaf-head">
      <p class="eyebrow">{{ eyebrow }}</p>
      <h1 class="leaf-title">{{ model.title || "Untitled recipe" }}</h1>
      <p class="leaf-summary">{{ summary }}</p>
      <AllergenBadges v-if="allergens.length" :allergens="allergens" />
      <p v-if="formulaMetrics" class="formula-chip">
        <span>{{ formulaMetrics.hydration }}% hydration</span>
        <span v-if="formulaMetrics.salt != null">{{ formulaMetrics.salt }}% salt</span>
        <span v-if="formulaMetrics.pieces != null">{{ formulaMetrics.pieces }} pieces</span>
      </p>
      <SubstitutionAssistant v-if="!kitchenMode" :resources="model.resources" />
    </header>

    <template v-if="!colocated">
      <section class="leaf-section ingredients">
        <h2 class="section-label">Ingredients</h2>
        <IngredientGroupList
          v-if="ingredientGroups.length"
          :groups="ingredientGroups"
          :selectable="editable"
          :highlighted-symbol="highlightedSymbol"
          :baker-by-symbol="bakerBySymbol"
          @select="emit('select-symbol', $event)"
        />
        <p v-else class="empty">
          {{
            editable
              ? "No ingredients yet — add some in Resources, or click Quick-add."
              : "No ingredients listed yet."
          }}
        </p>
      </section>

      <section v-if="equipment.length" class="leaf-section equipment">
        <h2 class="section-label">Equipment</h2>
        <ul class="equipment-list">
          <li v-for="item in equipment" :key="item">{{ item }}</li>
        </ul>
      </section>
    </template>

    <section class="leaf-section method" :class="{ colocated }">
      <h2 v-if="!colocated" class="section-label">Method</h2>

      <!-- Mise layout: each process is its own block (heading → mise → steps). -->
      <template v-if="colocated && hasSteps">
        <section v-for="section in sections" :key="section.process" class="method-section">
          <h3 v-if="section.title" class="section-label process-heading">{{ section.title }}</h3>
          <MiseBlock
            :mise="section.mise"
            :selectable="editable"
            :highlighted-symbol="highlightedSymbol"
            @select="emit('select-symbol', $event)"
          />
          <p v-if="section.note" class="section-note">{{ section.note }}</p>
          <div class="steps">
            <RecipeStepRow
              v-for="step in section.steps"
              :key="step.symbol"
              :number="step.number"
              :operation="operationFor(step)"
              :text="step.text"
              :meta="step.meta"
              :time="step.time"
              :recipe-id="recipeId"
              :editable="editable"
              :highlighted="highlightedSymbol === step.symbol"
              :kitchen-operation="kitchenOperationFor(step)"
              :kitchen-timer="kitchenTimerFor(step)"
              @delete="removeStep(step)"
              @select="emit('select-symbol', $event)"
              @start-timer="runKitchenAction(() => startOperation($event))"
              @complete-timer="runKitchenAction(() => completeOperation($event))"
            />
          </div>
        </section>
      </template>

      <!-- List layout: one Method section, optional inline process headings. -->
      <div v-else-if="hasSteps" class="steps">
        <template v-for="section in sections" :key="section.process">
          <h3 v-if="section.title" class="process-heading">{{ section.title }}</h3>
          <p v-if="section.note" class="section-note">{{ section.note }}</p>
          <RecipeStepRow
            v-for="step in section.steps"
            :key="step.symbol"
            :number="step.number"
            :operation="operationFor(step)"
            :text="step.text"
            :meta="step.meta"
            :time="step.time"
            :recipe-id="recipeId"
            :editable="editable"
            :highlighted="highlightedSymbol === step.symbol"
            :kitchen-operation="kitchenOperationFor(step)"
            :kitchen-timer="kitchenTimerFor(step)"
            @delete="removeStep(step)"
            @select="emit('select-symbol', $event)"
            @start-timer="runKitchenAction(() => startOperation($event))"
            @complete-timer="runKitchenAction(() => completeOperation($event))"
          />
        </template>
      </div>
      <p v-else class="empty">
        {{ editable ? "No steps yet — add a first step in Method on the right." : "No steps yet." }}
      </p>
    </section>

    <footer v-if="model.attribution || model.source" class="leaf-credit">
      <p v-if="model.attribution">{{ model.attribution }}</p>
      <p v-else>Recipe from {{ model.source }}.</p>
      <a v-if="model.sourceUrl" :href="model.sourceUrl" target="_blank" rel="noopener noreferrer">
        {{ model.sourceUrl }}
      </a>
    </footer>
  </article>
</template>

<style scoped>
.formula-chip {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 14px;
  margin: 8px 0 0;
  font-family: var(--sans);
  font-size: calc(13px * var(--reading-scale, 1));
  color: #7a5a16;
}
.formula-chip span {
  background: #f7efd8;
  padding: 2px 8px;
  border-radius: 999px;
}
.kitchen-leaf {
  padding-top: 22px;
  box-shadow:
    inset 5px 0 0 #28643b,
    inset 14px 0 22px -18px rgba(60, 50, 30, 0.45),
    0 1px 2px rgba(40, 40, 30, 0.1),
    0 22px 50px -28px rgba(40, 40, 30, 0.45);
}
.kitchen-strip {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin: 0 0 22px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--rule);
}
.kitchen-progress {
  flex: 1;
  min-width: 0;
  display: grid;
  gap: 6px;
}
.kitchen-progress > span,
.finish-kitchen {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  font-family: var(--sans);
}
.kitchen-progress > span {
  color: var(--herb);
  font-size: calc(12px * var(--reading-scale, 1));
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}
.kitchen-progress small {
  font-family: var(--sans);
  font-size: calc(12px * var(--reading-scale, 1));
  color: #6d7972;
}
.progress-track {
  height: 4px;
  max-width: 220px;
  border-radius: 999px;
  background: #dfe6df;
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  border-radius: inherit;
  background: #28643b;
  transition: width 0.25s ease;
}
.finish-kitchen {
  padding: 6px 10px;
  border: 1px solid #b9cbbd;
  border-radius: 7px;
  background: #f1f6ef;
  color: var(--herb);
  font-size: 12px;
  font-weight: 700;
}
.kitchen-error {
  margin: -10px 0 18px;
  color: #a83737;
  font-size: 13px;
}
</style>
