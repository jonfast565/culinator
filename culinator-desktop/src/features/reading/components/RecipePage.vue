<script setup lang="ts">
import { computed, inject, ref, toRef, watch } from "vue";
import { Clock } from "lucide-vue-next";
import type { UiOperation, UiRecipeModel } from "../../recipe-editor/model";
import { useRecipeNarrative, formatIngredientDescription } from "../../recipe-editor/narrative";
import {
  formatOperationTemperature,
  UNIT_DISPLAY_KEY,
} from "../../units/composables/useUnitDisplay";
import RecipeImage from "./RecipeImage.vue";

const props = defineProps<{ model: UiRecipeModel; recipeId?: string }>();

const units = inject(UNIT_DISPLAY_KEY, null);

const { ingredientGroups, operations, rows, summary, describe, stepTime, stepMeta } =
  useRecipeNarrative(toRef(props, "model"));

const formattedQuantities = ref<Record<string, string>>({});
const formattedTemperatures = ref<Record<string, string>>({});

watch(
  [() => ingredientGroups.value.flatMap((group) => group.items), () => units?.unitSystem.value],
  async () => {
    if (!units) {
      formattedQuantities.value = {};
      return;
    }
    const map: Record<string, string> = {};
    await Promise.all(
      ingredientGroups.value.flatMap((group) =>
        group.items.map(async (ingredient) => {
          map[ingredient.symbol] = await units.formatQuantity(ingredient.quantity);
        }),
      ),
    );
    formattedQuantities.value = map;
  },
  { immediate: true },
);

watch(
  [operations, () => units?.unitSystem.value],
  async () => {
    if (!units) {
      formattedTemperatures.value = {};
      return;
    }
    const map: Record<string, string> = {};
    await Promise.all(
      operations.value.map(async (operation) => {
        if (!operation.targetTemperature) return;
        map[operation.symbol] = await formatOperationTemperature(
          operation.targetTemperature,
          units.unitSystem.value,
        );
      }),
    );
    formattedTemperatures.value = map;
  },
  { immediate: true },
);

function displayQuantity(symbol: string, fallback?: string): string {
  return formattedQuantities.value[symbol] || fallback || "—";
}

function ingredientLine(ingredient: (typeof ingredientGroups.value)[number]["items"][number]): string {
  return formatIngredientDescription(
    ingredient,
    displayQuantity(ingredient.symbol, ingredient.quantity),
  );
}

function describeStep(operation: UiOperation): string {
  const base = describe(operation);
  const formatted = formattedTemperatures.value[operation.symbol];
  if (!formatted || !operation.targetTemperature) return base;
  return base.replace(operation.targetTemperature, formatted);
}

const hasSteps = computed(() => operations.value.length > 0);
const eyebrow = computed(() => props.model.attribution || props.model.source || "Recipe");
</script>

<template>
  <article class="leaf">
    <figure v-if="model.coverImage" class="leaf-cover">
      <RecipeImage :image-ref="model.coverImage" :recipe-id="recipeId" :alt="model.title" />
    </figure>
    <header class="leaf-head">
      <p class="eyebrow">{{ eyebrow }}</p>
      <h1 class="leaf-title">{{ model.title || "Untitled recipe" }}</h1>
      <p class="leaf-summary">{{ summary }}</p>
    </header>

    <section class="leaf-section ingredients">
      <h2 class="section-label">Ingredients</h2>
      <div v-if="ingredientGroups.length" class="ingredient-groups">
        <div v-for="group in ingredientGroups" :key="group.label ?? 'base'" class="ingredient-group">
          <h3 v-if="group.label" class="variant-heading">{{ group.label }} finish</h3>
          <ul class="ingredient-list">
            <li v-for="ingredient in group.items" :key="ingredient.symbol">
              {{ ingredientLine(ingredient) }}
            </li>
          </ul>
        </div>
      </div>
      <p v-else class="empty">No ingredients listed yet.</p>
    </section>

    <section class="leaf-section method">
      <h2 class="section-label">Method</h2>
      <div v-if="hasSteps" class="steps">
        <template v-for="row in rows" :key="row.key">
          <h3 v-if="row.kind === 'heading'" class="process-heading">{{ row.label }}</h3>
          <div v-else class="step">
            <span class="step-number">{{ row.number }}</span>
            <div class="step-body">
              <p class="step-text">{{ describeStep(row.operation!) }}</p>
              <small v-if="stepMeta(row.operation!)" class="step-meta">{{
                stepMeta(row.operation!)
              }}</small>
              <figure v-if="row.operation!.photo" class="step-photo">
                <RecipeImage :image-ref="row.operation!.photo" :recipe-id="recipeId" />
              </figure>
            </div>
            <span v-if="stepTime(row.operation!)" class="step-time">
              <Clock :size="12" />{{ stepTime(row.operation!) }}
            </span>
          </div>
        </template>
      </div>
      <p v-else class="empty">No steps yet.</p>
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
.leaf {
  --serif: "Iowan Old Style", "Palatino Linotype", Palatino, "Book Antiqua", Georgia, serif;
  --ink: #23302a;
  --muted: #6d7972;
  --herb: #28643b;
  --rule: #e0ded2;
  position: relative;
  max-width: 720px;
  margin: 0 auto;
  padding: clamp(28px, 5vw, 60px) clamp(24px, 5vw, 64px);
  background: #fbf9f3;
  color: var(--ink);
  border-radius: 3px;
  /* A paper leaf: soft outer drop + a faint binding shadow down the left edge. */
  box-shadow:
    inset 14px 0 22px -18px rgba(60, 50, 30, 0.45),
    0 1px 2px rgba(40, 40, 30, 0.1),
    0 22px 50px -28px rgba(40, 40, 30, 0.45);
}

.leaf-cover {
  margin: 0 0 24px;
  aspect-ratio: 16 / 9;
  overflow: hidden;
  border-radius: 4px;
  box-shadow: 0 12px 30px -18px rgba(40, 40, 30, 0.5);
}
.leaf-head {
  padding-bottom: 20px;
  border-bottom: 2px solid var(--ink);
}
.step-photo {
  margin: 12px 0 2px;
  max-width: 340px;
  aspect-ratio: 4 / 3;
  overflow: hidden;
  border-radius: 4px;
  box-shadow: 0 8px 20px -14px rgba(40, 40, 30, 0.5);
}
.eyebrow {
  margin: 0 0 10px;
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--herb);
  font-weight: 600;
}
.leaf-title {
  margin: 0;
  font-family: var(--serif);
  font-weight: 600;
  font-size: clamp(30px, 5vw, 46px);
  line-height: 1.05;
  letter-spacing: -0.01em;
}
.leaf-summary {
  margin: 12px 0 0;
  font-size: 12px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--muted);
}

.leaf-section {
  margin-top: 34px;
}
.section-label {
  margin: 0 0 16px;
  font-family: var(--serif);
  font-size: 15px;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: var(--herb);
  display: flex;
  align-items: center;
  gap: 12px;
}
.section-label::after {
  content: "";
  flex: 1;
  height: 1px;
  background: var(--rule);
}

.ingredient-groups {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.variant-heading {
  margin: 0 0 8px;
  font-family: var(--serif);
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--muted);
}
.ingredient-list {
  list-style: none;
  margin: 0;
  padding: 0;
}
.ingredient-list li {
  padding: 7px 0;
  border-bottom: 1px dotted #e2e0d4;
  line-height: 1.45;
}

.steps {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.process-heading {
  margin: 8px 0 -4px;
  font-family: var(--serif);
  font-size: 16px;
  font-weight: 600;
  color: var(--ink);
}
.step {
  display: grid;
  grid-template-columns: 34px 1fr auto;
  gap: 16px;
  align-items: start;
}
.step-number {
  font-family: var(--serif);
  font-size: 26px;
  font-weight: 600;
  line-height: 1;
  color: var(--herb);
  text-align: right;
  font-variant-numeric: lining-nums;
}
.step-text {
  margin: 0;
  font-size: 16px;
  line-height: 1.55;
}
.step-meta {
  display: block;
  margin-top: 4px;
  font-size: 12px;
  text-transform: capitalize;
  color: var(--muted);
}
.step-time {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  justify-self: end;
  white-space: nowrap;
  padding: 3px 11px;
  border-radius: 999px;
  background: #e8f0e6;
  color: var(--herb);
  font-size: 12px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
.step-time svg {
  opacity: 0.7;
}

.empty {
  color: var(--muted);
  font-style: italic;
}

.leaf-credit {
  margin-top: 40px;
  padding-top: 18px;
  border-top: 1px solid var(--rule);
  font-size: 12px;
  color: var(--muted);
}
.leaf-credit p {
  margin: 0 0 4px;
}
.leaf-credit a {
  color: var(--herb);
  word-break: break-all;
}
</style>
