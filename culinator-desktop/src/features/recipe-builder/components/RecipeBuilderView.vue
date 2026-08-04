<script setup lang="ts">
/* global CSS, HTMLElement, HTMLInputElement, HTMLTextAreaElement, HTMLSelectElement, IntersectionObserver, navigator */
import { computed, nextTick, onBeforeUnmount, onMounted, ref, toRef } from "vue";
import {
  Beaker,
  Carrot,
  GitBranch,
  ListOrdered,
  Scale,
  SlidersHorizontal,
  TriangleAlert,
} from "lucide-vue-next";
import type { UiRecipeModel } from "../../recipe-editor/model";
import VisualAuthoringPanel from "../../visual-authoring/components/VisualAuthoringPanel.vue";
import { useAppDialog } from "../../../shared/composables/useAppDialog";
import { useRecipeBuilder } from "../composables/useRecipeBuilder";
import MetadataSection from "./MetadataSection.vue";
import ResourcesSection from "./ResourcesSection.vue";
import OperationsSection from "./OperationsSection.vue";
import YieldsSection from "./YieldsSection.vue";
import FormulasSection from "./FormulasSection.vue";

/**
 * The full-screen structured recipe builder.
 *
 * This owns only layout and the toolbar; every section is a dumb child that
 * takes the current model and emits an edit. `source` is the editor's buffer,
 * exposed here as a writable computed so `useRecipeBuilder` can splice through
 * it — which keeps a single owner of dirty state and autosave in the editor.
 */
const props = defineProps<{
  source: string;
  model: UiRecipeModel;
  recipeId?: string;
}>();

const emit = defineEmits<{
  "update:source": [value: string];
  "edit-source": [];
  "focus-symbol": [symbol: string];
}>();

const sourceRef = computed({
  get: () => props.source,
  set: (value) => emit("update:source", value),
});
const modelRef = toRef(props, "model");

const {
  outlineFailed,
  metadata,
  setMetadata,
  resources,
  setResourceString,
  setResourceQuantity,
  setResourceFlag,
  setResourceKind,
  setResourceMeasurement,
  setResourceSubstitutes,
  setResourceNotes,
  addResource,
  removeResource,
  duplicateResource,
  moveResource,
  processes,
  symbols,
  setOperationVerb,
  setOperationInputs,
  setOperationProduces,
  setOperationAfter,
  setOperationField,
  setOperationFlag,
  setOperationNotes,
  setOperationDoneness,
  setOperationEquipment,
  setOperationPhotoRef,
  addOperation,
  addPrep,
  addFirstStep,
  createIngredientFromName,
  createResourceFromName,
  removeOperation,
  duplicateOperation,
  moveOperation,
  addProcess,
  yields,
  setYieldAmount,
  setYieldMeasurement,
  addYield,
  removeYield,
  formulas,
  setFormulaTarget,
  setFormulaIngredientBaker,
  addFormula,
  removeFormula,
  addFormulaIngredient,
  removeFormulaIngredient,
  renameDeclaration,
} = useRecipeBuilder(sourceRef, modelRef);

const dialog = useAppDialog();
const focusedSymbol = ref<string | null>(null);
const activeSection = ref("builder-details");
const builderStage = ref<HTMLElement | null>(null);
const showGraph = ref(
  typeof window !== "undefined" ? window.localStorage.getItem("cg:builder-graph") !== "0" : true,
);
const modLabel =
  typeof navigator !== "undefined" && navigator.platform.toLowerCase().includes("mac")
    ? "⌘"
    : "Ctrl+";

const sections = [
  { id: "builder-details", label: "Details", icon: SlidersHorizontal },
  { id: "builder-resources", label: "Resources", icon: Carrot },
  { id: "builder-steps", label: "Method", icon: ListOrdered },
  { id: "builder-yields", label: "Yield", icon: Scale },
  { id: "builder-formulas", label: "Formulas", icon: Beaker },
  { id: "builder-workflow", label: "Graph", icon: GitBranch },
];

/** Resources wired into a step (or listed as a substitute) — others get an Unused badge. */
const usedResourceSymbols = computed(() => {
  const used = new Set<string>();
  for (const group of processes.value) {
    for (const operation of group.operations) {
      for (const binding of operation.inputs) {
        if (binding.symbol.trim()) used.add(binding.symbol.trim());
      }
      if (operation.produces.trim()) used.add(operation.produces.trim());
      for (const equipment of operation.equipment) {
        if (equipment.symbol.trim()) used.add(equipment.symbol.trim());
      }
    }
  }
  for (const resource of resources.value) {
    for (const substitute of resource.substitutes) {
      if (substitute.trim()) used.add(substitute.trim());
    }
  }
  return used;
});

let sectionObserver: IntersectionObserver | null = null;

function jumpTo(id: string): void {
  activeSection.value = id;
  document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
}

function markFocus(symbol: string): void {
  focusedSymbol.value = symbol;
  emit("focus-symbol", symbol);
}

function preferredFocus(
  card: HTMLElement,
): HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement | null {
  const preferred = card.querySelector<HTMLElement>("[data-focus-priority]");
  if (preferred instanceof HTMLInputElement || preferred instanceof HTMLTextAreaElement) {
    return preferred;
  }
  if (preferred instanceof HTMLSelectElement) return preferred;
  return card.querySelector<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>(
    "input:not([type='checkbox']):not([type='file']), textarea, select",
  );
}

function focusSymbol(symbol: string): void {
  markFocus(symbol);
  void nextTick(() => {
    const card = document.querySelector(
      `[data-builder-symbol="${CSS.escape(symbol)}"]`,
    ) as HTMLElement | null;
    if (!card) return;
    card.scrollIntoView({ behavior: "smooth", block: "nearest" });
    preferredFocus(card)?.focus({ preventScroll: true });
  });
}

async function onRemoveResource(symbol: string): Promise<void> {
  const resource = resources.value.find((item) => item.symbol === symbol);
  const label = resource?.name || symbol.replace(/_/g, " ");
  if (
    !(await dialog.confirm(`Remove “${label}” from this recipe?`, {
      title: "Remove resource",
      confirmLabel: "Remove",
    }))
  ) {
    return;
  }
  removeResource(symbol);
}

async function onRemoveOperation(symbol: string): Promise<void> {
  const label = symbol.replace(/_/g, " ");
  if (
    !(await dialog.confirm(`Delete step “${label}”?`, {
      title: "Delete step",
      confirmLabel: "Delete",
    }))
  ) {
    return;
  }
  removeOperation(symbol);
}

onMounted(() => {
  const root = builderStage.value;
  if (!root) return;
  sectionObserver = new IntersectionObserver(
    (entries) => {
      const visible = entries
        .filter((entry) => entry.isIntersecting)
        .sort((a, b) => b.intersectionRatio - a.intersectionRatio);
      const top = visible[0]?.target as HTMLElement | undefined;
      if (top?.id) activeSection.value = top.id;
    },
    { root, rootMargin: "-10% 0px -55% 0px", threshold: [0.15, 0.4, 0.7] },
  );
  for (const section of sections) {
    const el = document.getElementById(section.id);
    if (el) sectionObserver.observe(el);
  }
});

onBeforeUnmount(() => {
  sectionObserver?.disconnect();
  sectionObserver = null;
});

function afterAdd(symbol: string | undefined): void {
  if (symbol) focusSymbol(symbol);
}

function onAddResource(kind: string): void {
  afterAdd(addResource(kind));
}

function onAddOp(process: string): void {
  afterAdd(addOperation(process));
}

function onAddPrep(process: string, verb: string, ingredient: string): void {
  afterAdd(addPrep(process, verb, ingredient));
}

function onAddFirstStep(): void {
  afterAdd(addFirstStep());
}

function onAddNamed(name: string): void {
  afterAdd(createIngredientFromName(name));
}

/** Create-from-use: declare the resource, then scroll it into view for details. */
function onCreateIngredient(name: string): string | undefined {
  const symbol = createIngredientFromName(name);
  if (symbol) afterAdd(symbol);
  return symbol;
}

function onCreateEquipment(name: string, role: string): string | undefined {
  const kind = role === "container" || role === "target" ? "container" : "equipment";
  const symbol = createResourceFromName(kind, name);
  if (symbol) afterAdd(symbol);
  return symbol;
}

function toggleGraph(): void {
  showGraph.value = !showGraph.value;
  window.localStorage.setItem("cg:builder-graph", showGraph.value ? "1" : "0");
  if (showGraph.value) jumpTo("builder-workflow");
}

defineExpose({ focusSymbol });
</script>

<template>
  <aside class="builder">
    <p v-if="outlineFailed" class="outline-banner">
      <TriangleAlert :size="15" />
      The source can't be parsed right now, so structured editing is paused. Fix it in the
      <button class="link" @click="emit('edit-source')">source editor</button> to continue.
    </p>

    <div class="builder-body">
      <nav class="builder-rail">
        <button
          v-for="section in sections"
          :key="section.id"
          class="rail-link"
          :class="{ active: activeSection === section.id }"
          @click="section.id === 'builder-workflow' ? toggleGraph() : jumpTo(section.id)"
        >
          <component :is="section.icon" :size="15" />
          {{ section.label }}
        </button>
        <p class="rail-hint">{{ modLabel }}S save · {{ modLabel }}E source</p>
      </nav>

      <div ref="builderStage" class="builder-stage">
        <MetadataSection
          :metadata="metadata"
          :recipe-id="recipeId"
          :disabled="outlineFailed"
          @commit="setMetadata"
        />
        <ResourcesSection
          :resources="resources"
          :disabled="outlineFailed"
          :focused-symbol="focusedSymbol"
          :used-symbols="usedResourceSymbols"
          @string="setResourceString"
          @quantity="setResourceQuantity"
          @flag="setResourceFlag"
          @kind="setResourceKind"
          @measurement="setResourceMeasurement"
          @substitutes="setResourceSubstitutes"
          @notes="setResourceNotes"
          @rename="renameDeclaration"
          @duplicate="duplicateResource"
          @add="onAddResource"
          @add-named="onAddNamed"
          @remove="onRemoveResource"
          @move="moveResource"
          @focus="markFocus"
        />
        <OperationsSection
          :processes="processes"
          :resource-symbols="symbols.resources"
          :operation-symbols="symbols.operations"
          :recipe-id="recipeId"
          :disabled="outlineFailed"
          :focused-symbol="focusedSymbol"
          :create-ingredient="onCreateIngredient"
          :create-resource="onCreateEquipment"
          @verb="setOperationVerb"
          @inputs="setOperationInputs"
          @produces="setOperationProduces"
          @after="setOperationAfter"
          @field="setOperationField"
          @flag="setOperationFlag"
          @notes="setOperationNotes"
          @doneness="setOperationDoneness"
          @equipment="setOperationEquipment"
          @photo="setOperationPhotoRef"
          @rename="(from, to) => renameDeclaration(from, to)"
          @duplicate-op="duplicateOperation"
          @remove-op="onRemoveOperation"
          @move-op="moveOperation"
          @add-op="onAddOp"
          @add-prep="onAddPrep"
          @add-first-step="onAddFirstStep"
          @add-process="addProcess"
          @focus="markFocus"
        />
        <YieldsSection
          :yields="yields"
          :disabled="outlineFailed"
          @amount="setYieldAmount"
          @measurement="setYieldMeasurement"
          @add="addYield"
          @remove="removeYield"
        />
        <FormulasSection
          :formulas="formulas"
          :disabled="outlineFailed"
          @target="setFormulaTarget"
          @ingredient-baker="setFormulaIngredientBaker"
          @add="addFormula"
          @remove="removeFormula"
          @add-ingredient="addFormulaIngredient"
          @remove-ingredient="removeFormulaIngredient"
        />

        <section v-if="showGraph" id="builder-workflow" class="panel builder-section workflow-dock">
          <div class="panel-header">
            <h3>Workflow graph</h3>
            <button type="button" class="ghost" @click="toggleGraph">Hide</button>
          </div>
          <p class="workflow-hint">
            Solid arrows are ingredients flowing into steps; dashed arrows are
            <code>after</code> dependencies the scheduler uses.
          </p>
          <VisualAuthoringPanel
            embedded
            :source="source"
            :model="model"
            @update:source="emit('update:source', $event)"
            @select-symbol="focusSymbol"
          />
        </section>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.builder {
  flex: 1;
  width: 100%;
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #f7f6f2;
}
.outline-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  padding: 10px 18px;
  background: #fbf6e7;
  color: #8a6d1f;
  font-size: 13px;
  border-bottom: 1px solid #ece3c4;
}
.outline-banner .link {
  padding: 0;
  border: 0;
  background: transparent;
  color: #28643b;
  text-decoration: underline;
  font: inherit;
  cursor: pointer;
}
.builder-body {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template-columns: 148px minmax(320px, 1fr);
  overflow: hidden;
}
.builder-rail {
  border-right: 1px solid #dde1dc;
  padding: 16px 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  background: #f7f6f2;
}
.rail-link {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: #45524b;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}
.rail-link:hover {
  background: #eceee9;
}
.rail-link.active {
  background: #e4efe6;
  color: #28643b;
  font-weight: 600;
}
.rail-hint {
  margin: auto 0 0;
  padding: 10px 8px 4px;
  font-size: 10px;
  line-height: 1.35;
  color: #8a938c;
}
.builder-stage {
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: 22px clamp(16px, 3vw, 32px);
  display: grid;
  gap: 20px;
  align-content: start;
}
.workflow-dock {
  min-height: 420px;
}
.workflow-dock .panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.workflow-hint {
  margin: 0 0 12px;
  font-size: 12px;
  color: #6d7972;
}
.workflow-hint code {
  font-size: 11px;
}
.workflow-dock :deep(.graph-scroll) {
  min-height: 360px;
  max-height: min(70vh, 560px);
}
@media (max-width: 720px) {
  .builder-body {
    grid-template-columns: 1fr;
  }
  .builder-rail {
    display: none;
  }
}
</style>
