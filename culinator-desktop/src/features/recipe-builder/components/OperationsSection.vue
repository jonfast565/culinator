<script setup lang="ts">
import { Plus } from "lucide-vue-next";
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
defineProps<{
  processes: BuilderProcess[];
  resourceSymbols: string[];
  operationSymbols: string[];
  recipeId?: string;
  disabled?: boolean;
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
  rename: [symbol: string, value: string];
  duplicateOp: [symbol: string];
  removeOp: [symbol: string];
  moveOp: [symbol: string, direction: "up" | "down"];
  addOp: [process: string];
  addProcess: [];
}>();

function processName(symbol: string): string {
  return symbol ? symbol.replace(/_/g, " ") : "Steps";
}
</script>

<template>
  <section id="builder-steps" class="panel builder-section">
    <div class="panel-header">
      <h3>Method</h3>
    </div>

    <p v-if="!processes.length" class="empty">No steps yet. Add a process, then add steps to it.</p>

    <div v-for="group in processes" :key="group.symbol" class="process-group">
      <div class="process-head">
        <h4>{{ processName(group.symbol) }}</h4>
        <button :disabled="disabled" @click="emit('addOp', group.symbol)">
          <Plus :size="14" /> Step
        </button>
      </div>
      <div class="cards">
        <OperationCard
          v-for="(operation, index) in group.operations"
          :key="operation.symbol"
          :operation="operation"
          :resource-symbols="resourceSymbols"
          :operation-symbols="operationSymbols"
          :recipe-id="recipeId"
          :disabled="disabled"
          :can-move-up="index > 0"
          :can-move-down="index < group.operations.length - 1"
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
        />
      </div>
    </div>

    <div class="add-row">
      <button :disabled="disabled" @click="emit('addProcess')"><Plus :size="14" /> Process</button>
    </div>
  </section>
</template>

<style scoped>
.empty {
  color: #8a938c;
  font-size: 13px;
  margin: 0 0 12px;
}
.process-group {
  margin-bottom: 20px;
}
.process-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 10px;
}
.process-head h4 {
  margin: 0;
  font-size: 13px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #55635b;
}
.process-head button,
.add-row button {
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
