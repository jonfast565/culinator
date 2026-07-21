<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { UiRecipeModel } from "../model";
import FormulaCalculator from "../../formulas/components/FormulaCalculator.vue";
import ExportPanel from "../../export/components/ExportPanel.vue";
import VisualAuthoringPanel from "../../visual-authoring/components/VisualAuthoringPanel.vue";
import GanttSchedule from "../../scheduling/components/GanttSchedule.vue";
import HaccpPanel from "../../haccp/components/HaccpPanel.vue";
import KitchenModePanel from "../../kitchen-mode/components/KitchenModePanel.vue";
import NutritionPanel from "../../nutrition/components/NutritionPanel.vue";
import RecipeNarrative from "./RecipeNarrative.vue";
import { ingredientPartsBySymbol } from "../narrative";
import IngredientListRow from "../../reading/components/IngredientListRow.vue";

export type InspectorTabId =
  | "narrative"
  | "outline"
  | "ingredients"
  | "author"
  | "timeline"
  | "formula"
  | "haccp"
  | "kitchen"
  | "nutrition"
  | "export";

const props = defineProps<{
  model: UiRecipeModel;
  recipeId?: string;
  source: string;
  initialTab?: InspectorTabId;
  showNavigation?: boolean;
}>();

// Ingredient lines are rendered by the shared narrative generator; look each
// one up by symbol rather than formatting it here.
const ingredientParts = computed(() => ingredientPartsBySymbol(props.source));
function partsFor(symbol: string) {
  return (
    ingredientParts.value.get(symbol) ?? {
      symbol,
      amount: "",
      description: symbol,
      aside: undefined,
    }
  );
}

const emit = defineEmits<{
  "update:source": [value: string];
  "kitchen-started": [];
}>();

const tabGroups = [
  {
    label: "Preview",
    tabs: [
      { id: "narrative" as const, label: "Narrative" },
      { id: "outline" as const, label: "Outline" },
      { id: "ingredients" as const, label: "Ingredients" },
    ],
  },
  {
    label: "Authoring",
    tabs: [{ id: "author" as const, label: "Workflow graph" }],
  },
  {
    label: "Planning",
    tabs: [
      { id: "timeline" as const, label: "Timeline" },
      { id: "formula" as const, label: "Formulas" },
    ],
  },
  {
    label: "Production",
    tabs: [
      { id: "kitchen" as const, label: "Cook mode" },
      { id: "haccp" as const, label: "Food safety" },
      { id: "nutrition" as const, label: "Nutrition" },
    ],
  },
  {
    label: "Output",
    tabs: [{ id: "export" as const, label: "Export" }],
  },
];

function defaultTab(): InspectorTabId {
  if (props.initialTab) return props.initialTab;
  return (props.model.operations?.length ?? 0) > 0 ? "author" : "narrative";
}

const tab = ref<InspectorTabId>(defaultTab());

watch(
  () => props.initialTab,
  (next) => {
    if (next) tab.value = next;
  },
);

watch(
  () => props.model.operations?.length ?? 0,
  (count, previous) => {
    if (!props.initialTab && previous === 0 && count > 0 && tab.value === "narrative") {
      tab.value = "author";
    }
  },
);

const operations = computed(() => props.model.operations ?? []);
const operationSymbols = computed(() => operations.value.map((item) => item.symbol));
</script>

<template>
  <aside class="inspector">
    <nav v-if="showNavigation !== false" class="tab-groups">
      <div v-for="group in tabGroups" :key="group.label" class="tab-group">
        <span class="tab-group-label">{{ group.label }}</span>
        <div class="tabs">
          <button
            v-for="item in group.tabs"
            :key="item.id"
            :class="{ active: tab === item.id }"
            @click="tab = item.id"
          >
            {{ item.label }}
          </button>
        </div>
      </div>
    </nav>
    <RecipeNarrative v-if="tab === 'narrative'" :model="model" :source="source" />
    <section v-else-if="tab === 'outline'" class="panel">
      <h3>{{ model.title || "Untitled recipe" }}</h3>
      <dl>
        <div>
          <dt>Symbol</dt>
          <dd>{{ model.symbol }}</dd>
        </div>
        <div>
          <dt>Resources</dt>
          <dd>{{ model.resources.length }}</dd>
        </div>
        <div>
          <dt>Processes</dt>
          <dd>{{ model.processes.length }}</dd>
        </div>
        <div>
          <dt>Operations</dt>
          <dd>{{ operations.length }}</dd>
        </div>
      </dl>
    </section>
    <section v-else-if="tab === 'ingredients'" class="panel">
      <h3>Ingredients</h3>
      <ul v-if="model.resources.some((r) => r.kind === 'ingredient')" class="inspector-ingredients">
        <IngredientListRow
          v-for="resource in model.resources.filter((r) => r.kind === 'ingredient')"
          :key="resource.symbol"
          :parts="partsFor(resource.symbol)"
        />
      </ul>
      <p v-else class="empty">No ingredients declared.</p>
      <h3 v-if="model.resources.some((r) => r.kind !== 'ingredient')" class="other-resources">
        Other resources
      </h3>
      <article
        v-for="resource in model.resources.filter((r) => r.kind !== 'ingredient')"
        :key="resource.symbol"
        class="card"
      >
        <strong
          >{{ resource.name || resource.symbol
          }}<em v-if="resource.state" class="state-tag">{{ resource.state }}</em
          ><em v-if="resource.optional" class="state-tag">optional</em
          ><em v-if="resource.divided" class="state-tag">divided</em></strong
        ><small>{{ resource.kind }} · {{ resource.measurement || "untyped" }}</small
        ><span v-if="resource.quantity">{{ resource.quantity }}</span
        ><small v-if="resource.substitutes?.length" class="substitutes"
          >or {{ resource.substitutes.join(", ") }}</small
        >
      </article>
    </section>
    <VisualAuthoringPanel
      v-else-if="tab === 'author'"
      :source="source"
      :model="model"
      @update:source="emit('update:source', $event)"
    />
    <GanttSchedule v-else-if="tab === 'timeline'" :source="source" />
    <FormulaCalculator
      v-else-if="tab === 'formula' && recipeId"
      :recipe-id="recipeId"
      :recipe-title="model.title"
      :resources="model.resources"
    />
    <HaccpPanel
      v-else-if="tab === 'haccp' && recipeId"
      :recipe-id="recipeId"
      :operation-symbols="operationSymbols"
    />
    <KitchenModePanel
      v-else-if="tab === 'kitchen' && recipeId"
      :recipe-id="recipeId"
      @started="emit('kitchen-started')"
    />
    <NutritionPanel
      v-else-if="tab === 'nutrition' && recipeId"
      :recipe-id="recipeId"
      :resources="model.resources"
    />
    <ExportPanel
      v-else-if="tab === 'export' && recipeId"
      :recipe-id="recipeId"
      :recipe-title="model.title"
    />
  </aside>
</template>

<style scoped>
.tab-groups {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 10px 12px;
  border-bottom: 1px solid #e2e6e1;
  background: #fff;
}
.tab-group-label {
  display: block;
  margin-bottom: 4px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #8a938c;
}
.tab-group .tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.tab-group .tabs button {
  padding: 5px 9px;
  font-size: 12px;
}
.state-tag {
  margin-left: 0.4rem;
  padding: 0.05rem 0.4rem;
  border-radius: 999px;
  font-style: normal;
  font-size: 0.68rem;
  font-weight: 600;
  text-transform: lowercase;
  color: #7a5a12;
  background: #f5e6c3;
}
.substitutes {
  font-style: italic;
  opacity: 0.75;
}
.inspector-ingredients {
  list-style: none;
  margin: 0 0 16px;
  padding: 0;
}
.other-resources {
  margin-top: 8px;
}
.empty {
  color: #6d7972;
  font-style: italic;
}
</style>
