<script setup lang="ts">
import { computed, toRef } from "vue";
import {
  Beaker,
  Carrot,
  ListOrdered,
  Scale,
  SlidersHorizontal,
  TriangleAlert,
} from "lucide-vue-next";
import type { UiRecipeModel } from "../../recipe-editor/model";
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

const sections = [
  { id: "builder-details", label: "Details", icon: SlidersHorizontal },
  { id: "builder-resources", label: "Resources", icon: Carrot },
  { id: "builder-steps", label: "Method", icon: ListOrdered },
  { id: "builder-yields", label: "Yield", icon: Scale },
  { id: "builder-formulas", label: "Formulas", icon: Beaker },
];

function jumpTo(id: string): void {
  document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
}
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
          @click="jumpTo(section.id)"
        >
          <component :is="section.icon" :size="15" />
          {{ section.label }}
        </button>
      </nav>

      <div class="builder-stage">
        <MetadataSection
          :metadata="metadata"
          :recipe-id="recipeId"
          :disabled="outlineFailed"
          @commit="setMetadata"
        />
        <ResourcesSection
          :resources="resources"
          :disabled="outlineFailed"
          @string="setResourceString"
          @quantity="setResourceQuantity"
          @flag="setResourceFlag"
          @kind="setResourceKind"
          @measurement="setResourceMeasurement"
          @substitutes="setResourceSubstitutes"
          @notes="setResourceNotes"
          @rename="renameDeclaration"
          @duplicate="duplicateResource"
          @add="addResource"
          @remove="removeResource"
          @move="moveResource"
        />
        <OperationsSection
          :processes="processes"
          :resource-symbols="symbols.resources"
          :operation-symbols="symbols.operations"
          :recipe-id="recipeId"
          :disabled="outlineFailed"
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
          @rename="renameDeclaration"
          @duplicate-op="duplicateOperation"
          @remove-op="removeOperation"
          @move-op="moveOperation"
          @add-op="addOperation"
          @add-process="addProcess"
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
.builder-stage {
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: 22px clamp(16px, 3vw, 32px);
  display: grid;
  gap: 20px;
  align-content: start;
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
