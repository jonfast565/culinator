<script setup lang="ts">
import { computed, inject, toRef } from "vue";
import type { UiOperation, UiRecipeModel } from "../../recipe-editor/model";
import { useRecipeNarrative, type NarrativeStep } from "../../recipe-editor/narrative";
import { deleteOperationFromSource } from "../../recipe-editor/sourcePatch";
import { UNIT_DISPLAY_KEY } from "../../units/composables/useUnitDisplay";
import { useAppDialog } from "../../../shared/composables/useAppDialog";
import { VIEW_SETTINGS_KEY } from "../composables/useViewSettings";
import RecipeImage from "./RecipeImage.vue";
import MiseBlock from "./MiseBlock.vue";
import IngredientGroupList from "./IngredientGroupList.vue";
import RecipeStepRow from "./RecipeStepRow.vue";

const props = defineProps<{
  model: UiRecipeModel;
  source: string;
  recipeId?: string;
  editable?: boolean;
}>();

const emit = defineEmits<{ "update:source": [value: string] }>();

const dialog = useAppDialog();
const units = inject(UNIT_DISPLAY_KEY, null);
const viewSettings = inject(VIEW_SETTINGS_KEY, null);

// Prose, amounts, times, and mise all come from the shared Rust generator, in
// the reader's chosen units and number style. Nothing is derived here.
const { summary, ingredientGroups, equipment, sections } = useRecipeNarrative(
  toRef(props, "source"),
  {
    unitSystem: computed(() => units?.unitSystem.value ?? "as_authored"),
    numberStyle: computed(() => viewSettings?.numberStyle.value ?? "fractions"),
  },
);

const colocated = computed(() => viewSettings?.misePlacement.value === "colocated");
const hasSteps = computed(() => sections.value.some((section) => section.steps.length > 0));

/** The parsed operation behind a step, for its photo and for source patching. */
function operationFor(step: NarrativeStep): UiOperation | undefined {
  return props.model.operations?.find((operation) => operation.symbol === step.symbol);
}

async function removeStep(step: NarrativeStep): Promise<void> {
  const operation = operationFor(step);
  if (!props.editable || !operation) return;
  if (!(await dialog.confirm(`Delete this step?\n\n${step.text}`))) return;
  const next = deleteOperationFromSource(props.source, operation);
  if (next != null) emit("update:source", next);
}

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

    <template v-if="!colocated">
      <section class="leaf-section ingredients">
        <h2 class="section-label">Ingredients</h2>
        <IngredientGroupList v-if="ingredientGroups.length" :groups="ingredientGroups" />
        <p v-else class="empty">No ingredients listed yet.</p>
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
          <MiseBlock :mise="section.mise" />
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
              @delete="removeStep(step)"
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
            @delete="removeStep(step)"
          />
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
.equipment-list {
  list-style: none;
  margin: 0;
  padding: 0;
}
.equipment-list li {
  padding: 7px 0;
  border-bottom: 1px dotted #e2e0d4;
  line-height: 1.45;
}

.steps {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.method.colocated {
  display: flex;
  flex-direction: column;
  gap: 28px;
}
.method-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.method-section .section-label.process-heading {
  margin-bottom: 0;
}
.method-section .steps {
  gap: 16px;
}
.process-heading {
  margin: 8px 0 -4px;
  font-family: var(--serif);
  font-size: 16px;
  font-weight: 600;
  color: var(--ink);
}
.method-section .process-heading {
  margin: 0;
  font-size: 15px;
  color: var(--herb);
}

.section-note {
  margin: 0 0 4px;
  font-size: 13px;
  font-style: italic;
  color: var(--muted);
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
