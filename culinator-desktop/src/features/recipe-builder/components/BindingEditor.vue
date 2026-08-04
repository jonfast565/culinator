<script setup lang="ts">
/* global HTMLElement, KeyboardEvent */
import { computed, ref, watch } from "vue";
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
  /** Create a missing ingredient and return its symbol. */
  createIngredient?: (name: string) => string | undefined;
}>();

const emit = defineEmits<{
  commit: [bindings: BuilderBinding[]];
}>();

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

const optionSet = computed(() => new Set(props.options));

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

function unknownName(symbol: string): string | null {
  const clean = symbol.trim();
  if (!clean || optionSet.value.has(clean)) return null;
  return clean;
}

function createAt(index: number): void {
  const name = unknownName(local.value[index]?.symbol ?? "");
  if (!name || !props.createIngredient) return;
  const symbol = props.createIngredient(name);
  if (!symbol) return;
  local.value = local.value.map((binding, i) => (i === index ? { ...binding, symbol } : binding));
  commit();
}

function onSymbolChange(): void {
  // Unknown names stay editable so the author can click "Add ingredient".
  commit();
}

function onSymbolEnter(index: number, event: KeyboardEvent): void {
  if (!unknownName(local.value[index]?.symbol ?? "")) return;
  event.preventDefault();
  createAt(index);
}
</script>

<template>
  <div ref="root" class="binding-editor">
    <span class="editor-label">Ingredients used</span>
    <datalist :id="listId">
      <option v-for="option in options" :key="option" :value="option" />
    </datalist>
    <div v-for="(binding, index) in local" :key="index" class="binding-block">
      <div class="binding-row" :class="{ unknown: !!unknownName(binding.symbol) }">
        <input
          v-model="binding.symbol"
          :list="listId"
          :disabled="disabled"
          placeholder="ingredient"
          aria-label="Ingredient"
          :data-focus-priority="index === 0 && !binding.symbol.trim() ? '' : undefined"
          @change="onSymbolChange"
          @keydown.enter="onSymbolEnter(index, $event)"
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
      <button
        v-if="createIngredient && unknownName(binding.symbol)"
        type="button"
        class="create-btn"
        :disabled="disabled"
        @click="createAt(index)"
      >
        <Plus :size="13" /> Add “{{ unknownName(binding.symbol) }}” as ingredient
      </button>
    </div>
    <button class="add-row" :disabled="disabled" @click="add"><Plus :size="14" /> Add input</button>
  </div>
</template>

<style scoped>
.binding-editor {
  display: grid;
  gap: 8px;
}
.editor-label {
  font-size: 12px;
  color: #657169;
}
.binding-block {
  display: grid;
  gap: 5px;
}
.binding-row {
  display: grid;
  grid-template-columns: 1.3fr 1fr auto;
  gap: 6px;
}
.binding-row.unknown input:first-child {
  border-color: #c9a24a;
  background: #fffdf6;
}
.icon {
  width: 34px;
  padding: 0;
  display: grid;
  place-items: center;
}
.create-btn {
  justify-self: start;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 9px;
  border: 1px dashed #8fb897;
  background: #f3faf4;
  color: #28643b;
  font-size: 12px;
  border-radius: 7px;
}
.create-btn:hover:not(:disabled) {
  background: #e4efe6;
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
