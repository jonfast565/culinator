<script setup lang="ts">
/* global Event, HTMLInputElement, HTMLSelectElement */
import { computed } from "vue";
import { X } from "lucide-vue-next";

/**
 * Explicit `after` predecessors for a step. Scheduling ignores source order, so
 * this control is how authors declare real dependencies. "Depends on previous"
 * is the common case when steps are written top-to-bottom.
 */
const props = defineProps<{
  predecessors: string[];
  options: string[];
  previousStep?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{ commit: [predecessors: string[]] }>();

const dependsOnPrevious = computed(
  () =>
    !!props.previousStep &&
    props.predecessors.length === 1 &&
    props.predecessors[0] === props.previousStep,
);

const available = computed(() =>
  props.options.filter((symbol) => !props.predecessors.includes(symbol)),
);

function setDependsOnPrevious(on: boolean): void {
  if (!props.previousStep) return;
  if (on) {
    emit("commit", [props.previousStep]);
    return;
  }
  emit(
    "commit",
    props.predecessors.filter((symbol) => symbol !== props.previousStep),
  );
}

function onDependsChange(event: Event): void {
  setDependsOnPrevious((event.target as HTMLInputElement).checked);
}

function addPredecessor(event: Event): void {
  const select = event.target as HTMLSelectElement;
  const symbol = select.value;
  select.value = "";
  if (!symbol || props.predecessors.includes(symbol)) return;
  emit("commit", [...props.predecessors, symbol]);
}

function removePredecessor(symbol: string): void {
  emit(
    "commit",
    props.predecessors.filter((item) => item !== symbol),
  );
}

function label(symbol: string): string {
  return symbol.replace(/_/g, " ");
}
</script>

<template>
  <div class="after-editor">
    <div class="after-head">
      <span class="editor-label">Wait for</span>
      <label v-if="previousStep" class="depends">
        <input
          type="checkbox"
          :checked="dependsOnPrevious"
          :disabled="disabled"
          @change="onDependsChange"
        />
        Depends on previous
      </label>
    </div>
    <div v-if="predecessors.length" class="chips">
      <span v-for="symbol in predecessors" :key="symbol" class="chip">
        {{ label(symbol) }}
        <button
          type="button"
          class="chip-remove"
          title="Remove dependency"
          :disabled="disabled"
          @click="removePredecessor(symbol)"
        >
          <X :size="12" />
        </button>
      </span>
    </div>
    <select
      v-if="available.length"
      class="add-select"
      :disabled="disabled"
      aria-label="Add step dependency"
      @change="addPredecessor"
    >
      <option value="">Add step…</option>
      <option v-for="symbol in available" :key="symbol" :value="symbol">
        {{ label(symbol) }}
      </option>
    </select>
    <p v-else-if="!predecessors.length" class="hint">
      No dependency — this step can start with the recipe.
    </p>
  </div>
</template>

<style scoped>
.after-editor {
  display: grid;
  gap: 6px;
}
.after-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.editor-label {
  font-size: 12px;
  color: #657169;
}
.depends {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin: 0;
  font-size: 12px;
  color: #45524b;
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 6px 3px 8px;
  border-radius: 999px;
  background: #e8efe9;
  color: #28643b;
  font-size: 12px;
  text-transform: capitalize;
}
.chip-remove {
  width: 18px;
  height: 18px;
  padding: 0;
  display: grid;
  place-items: center;
  border: 0;
  background: transparent;
  color: inherit;
  cursor: pointer;
}
.add-select {
  width: fit-content;
  max-width: 100%;
  font-size: 12px;
}
.hint {
  margin: 0;
  font-size: 11px;
  color: #8a938c;
}
</style>
