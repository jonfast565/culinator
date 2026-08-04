<script setup lang="ts">
/* global Event, HTMLInputElement */
import { computed, ref } from "vue";
import { ChevronsDownUp, ChevronsUpDown, Plus, Scissors } from "lucide-vue-next";
import type {
  BuilderBinding,
  BuilderDoneness,
  BuilderEquipment,
  BuilderProcess,
} from "../composables/useRecipeBuilder";
import OperationCard from "./OperationCard.vue";

/**
 * Steps, grouped by the process that contains them. The grouping is a display
 * concern only — every edit re-emits with the step's symbol so the composable
 * can resolve it in the current outline.
 */
const props = defineProps<{
  processes: BuilderProcess[];
  resourceSymbols: string[];
  operationSymbols: string[];
  recipeId?: string;
  disabled?: boolean;
  focusedSymbol?: string | null;
  createIngredient?: (name: string) => string | undefined;
  createResource?: (name: string, role: string) => string | undefined;
}>();

const emit = defineEmits<{
  verb: [symbol: string, value: string];
  inputs: [symbol: string, bindings: BuilderBinding[]];
  produces: [symbol: string, value: string];
  after: [symbol: string, predecessors: string[]];
  field: [symbol: string, key: string, value: string];
  flag: [symbol: string, key: string, value: boolean];
  notes: [symbol: string, value: string[]];
  doneness: [symbol: string, cues: BuilderDoneness[]];
  equipment: [symbol: string, bindings: BuilderEquipment[]];
  photo: [symbol: string, value: string];
  rename: [from: string, to: string];
  duplicateOp: [symbol: string];
  removeOp: [symbol: string];
  moveOp: [symbol: string, direction: "up" | "down"];
  addOp: [process: string];
  addPrep: [process: string, verb: string, ingredient: string];
  addFirstStep: [];
  addProcess: [];
  focus: [symbol: string];
}>();

const prepFor = ref<string | null>(null);
const prepVerb = ref("dice");
const prepIngredient = ref("");
/** Bumped to force-expand or force-collapse every card. */
const expandToken = ref(0);
const collapseToken = ref(0);

const stepCount = computed(() =>
  props.processes.reduce((total, group) => total + group.operations.length, 0),
);

function processName(symbol: string): string {
  return symbol ? symbol.replace(/_/g, " ") : "Steps";
}

function expandAll(): void {
  expandToken.value += 1;
}

function collapseAll(): void {
  collapseToken.value += 1;
}

function openPrep(process: string): void {
  prepFor.value = process;
  prepVerb.value = "dice";
  prepIngredient.value = "";
}

function commitPrep(): void {
  if (!prepFor.value || !prepIngredient.value.trim()) return;
  emit("addPrep", prepFor.value, prepVerb.value.trim() || "prep", prepIngredient.value.trim());
  prepFor.value = null;
  prepIngredient.value = "";
}

function renameProcess(from: string, event: Event): void {
  const value = (event.target as HTMLInputElement).value.trim() || from;
  emit("rename", from, value);
}
</script>

<template>
  <section id="builder-steps" class="panel builder-section">
    <div class="panel-header">
      <h3>Method</h3>
      <div v-if="stepCount > 1" class="panel-tools">
        <button
          type="button"
          class="ghost"
          :disabled="disabled"
          title="Expand all"
          @click="expandAll"
        >
          <ChevronsUpDown :size="14" /> Expand
        </button>
        <button
          type="button"
          class="ghost"
          :disabled="disabled"
          title="Collapse all"
          @click="collapseAll"
        >
          <ChevronsDownUp :size="14" /> Collapse
        </button>
      </div>
    </div>

    <div v-if="!processes.length" class="empty-cta">
      <p class="empty">No steps yet. Add a first step to start the method.</p>
      <button class="primary" :disabled="disabled" @click="emit('addFirstStep')">
        <Plus :size="14" /> Add first step
      </button>
    </div>

    <div v-for="group in processes" :key="group.symbol" class="process-group">
      <div class="process-head">
        <input
          v-if="group.symbol"
          class="process-title"
          :value="processName(group.symbol)"
          :disabled="disabled"
          aria-label="Process name"
          @change="renameProcess(group.symbol, $event)"
        />
        <h4 v-else>{{ processName(group.symbol) }}</h4>
        <div class="process-actions">
          <button :disabled="disabled" @click="emit('addOp', group.symbol)">
            <Plus :size="14" /> Step
          </button>
          <button :disabled="disabled" @click="openPrep(group.symbol)">
            <Scissors :size="14" /> Prep
          </button>
        </div>
      </div>

      <form v-if="prepFor === group.symbol" class="prep-form" @submit.prevent="commitPrep">
        <input
          v-model="prepVerb"
          list="prep-verbs"
          placeholder="verb (dice, grate…)"
          :disabled="disabled"
          aria-label="Prep verb"
        />
        <input
          v-model="prepIngredient"
          :list="`prep-ingredients-${group.symbol}`"
          placeholder="ingredient"
          :disabled="disabled"
          aria-label="Prep ingredient"
        />
        <datalist :id="`prep-ingredients-${group.symbol}`">
          <option v-for="symbol in resourceSymbols" :key="symbol" :value="symbol" />
        </datalist>
        <button type="submit" class="primary" :disabled="disabled || !prepIngredient.trim()">
          Add prep
        </button>
        <button type="button" :disabled="disabled" @click="prepFor = null">Cancel</button>
      </form>

      <div class="cards">
        <OperationCard
          v-for="(operation, index) in group.operations"
          :key="operation.symbol"
          :operation="operation"
          :resource-symbols="resourceSymbols"
          :operation-symbols="operationSymbols"
          :previous-step="index > 0 ? group.operations[index - 1]?.symbol : undefined"
          :recipe-id="recipeId"
          :disabled="disabled"
          :can-move-up="index > 0"
          :can-move-down="index < group.operations.length - 1"
          :force-expanded="focusedSymbol === operation.symbol"
          :expand-generation="expandToken"
          :collapse-generation="collapseToken"
          :start-expanded="group.operations.length === 1"
          :focused="focusedSymbol === operation.symbol"
          :create-ingredient="createIngredient"
          :create-resource="createResource"
          @verb="(value) => emit('verb', operation.symbol, value)"
          @inputs="(bindings) => emit('inputs', operation.symbol, bindings)"
          @produces="(value) => emit('produces', operation.symbol, value)"
          @after="(preds) => emit('after', operation.symbol, preds)"
          @field="(key, value) => emit('field', operation.symbol, key, value)"
          @flag="(key, value) => emit('flag', operation.symbol, key, value)"
          @notes="(value) => emit('notes', operation.symbol, value)"
          @doneness="(cues) => emit('doneness', operation.symbol, cues)"
          @equipment="(bindings) => emit('equipment', operation.symbol, bindings)"
          @photo="(value) => emit('photo', operation.symbol, value)"
          @rename="(value) => emit('rename', operation.symbol, value)"
          @duplicate="emit('duplicateOp', operation.symbol)"
          @remove="emit('removeOp', operation.symbol)"
          @move="(direction) => emit('moveOp', operation.symbol, direction)"
          @focus="emit('focus', operation.symbol)"
        />
      </div>
    </div>

    <datalist id="prep-verbs">
      <option value="dice" />
      <option value="chop" />
      <option value="mince" />
      <option value="slice" />
      <option value="grate" />
      <option value="peel" />
      <option value="zest" />
      <option value="crush" />
    </datalist>

    <div class="add-row">
      <button :disabled="disabled" @click="emit('addProcess')"><Plus :size="14" /> Process</button>
    </div>
  </section>
</template>

<style scoped>
.empty-cta {
  display: grid;
  gap: 10px;
  justify-items: start;
  margin-bottom: 12px;
}
.empty {
  color: #8a938c;
  font-size: 13px;
  margin: 0;
}
.empty-cta .primary {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.panel-tools {
  display: flex;
  gap: 4px;
}
.panel-tools .ghost {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 4px 8px;
}
.process-group {
  margin-bottom: 20px;
}
.process-head {
  position: sticky;
  top: 0;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin: 0 -4px 10px;
  padding: 8px 4px;
  background: linear-gradient(#f7f6f2 70%, rgb(247 246 242 / 0%));
}
.process-head h4,
.process-title {
  margin: 0;
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #55635b;
}
.process-title {
  border: 1px solid transparent;
  background: transparent;
  padding: 4px 6px;
  border-radius: 6px;
}
.process-title:hover:not(:disabled),
.process-title:focus-visible {
  border-color: #cbd3cd;
  background: #fff;
}
.process-actions {
  display: flex;
  gap: 6px;
}
.process-head button,
.add-row button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 13px;
}
.prep-form {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
  padding: 10px;
  border: 1px dashed #c5cec5;
  border-radius: 8px;
  background: #f7faf6;
}
.prep-form input {
  flex: 1;
  min-width: 120px;
}
.prep-form button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 13px;
}
.cards {
  display: grid;
  gap: 12px;
}
.add-row {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}
</style>
