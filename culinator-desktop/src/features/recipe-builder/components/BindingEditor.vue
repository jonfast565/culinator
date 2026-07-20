<script setup lang="ts">
/* global HTMLElement */
import { ref, watch } from "vue";
import { Plus, X } from "lucide-vue-next";
import type { BuilderBinding } from "../composables/useRecipeBuilder";

/**
 * A step's input bindings: each a resource with an optional per-step amount.
 * The composable turns the list back into DSL — one `input [a, b];` for the
 * unquantified ones plus an `input x 400 g;` per quantified one — so the editor
 * only has to present the rows.
 */
const props = defineProps<{
  bindings: BuilderBinding[];
  options: string[];
  disabled?: boolean;
  listId: string;
}>();

const emit = defineEmits<{ commit: [bindings: BuilderBinding[]] }>();

const local = ref<BuilderBinding[]>(props.bindings.map((binding) => ({ ...binding })));
const root = ref<HTMLElement>();

watch(
  () => props.bindings,
  (value) => {
    if (!root.value?.contains(document.activeElement)) {
      local.value = value.map((binding) => ({ ...binding }));
    }
  },
);

function commit(): void {
  emit(
    "commit",
    local.value.filter((binding) => binding.symbol.trim()),
  );
}
function add(): void {
  local.value = [...local.value, { symbol: "", quantity: "" }];
}
function removeAt(index: number): void {
  local.value = local.value.filter((_, i) => i !== index);
  commit();
}
</script>

<template>
  <div ref="root" class="binding-editor">
    <span class="editor-label">Ingredients used</span>
    <datalist :id="listId">
      <option v-for="option in options" :key="option" :value="option" />
    </datalist>
    <div v-for="(binding, index) in local" :key="index" class="binding-row">
      <input
        v-model="binding.symbol"
        :list="listId"
        :disabled="disabled"
        placeholder="ingredient"
        aria-label="Ingredient"
        @change="commit"
      />
      <input
        v-model="binding.quantity"
        :disabled="disabled"
        placeholder="amount (optional)"
        aria-label="Per-step amount"
        @change="commit"
      />
      <button class="icon" title="Remove" :disabled="disabled" @click="removeAt(index)">
        <X :size="14" />
      </button>
    </div>
    <button class="add-row" :disabled="disabled" @click="add"><Plus :size="14" /> Add input</button>
  </div>
</template>

<style scoped>
.binding-editor {
  display: grid;
  gap: 6px;
}
.editor-label {
  font-size: 12px;
  color: #657169;
}
.binding-row {
  display: grid;
  grid-template-columns: 1.3fr 1fr auto;
  gap: 6px;
}
.icon {
  width: 34px;
  padding: 0;
  display: grid;
  place-items: center;
}
.add-row {
  justify-self: start;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  padding: 5px 10px;
}
</style>
