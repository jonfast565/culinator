<script setup lang="ts">
/* global HTMLElement, KeyboardEvent */
import { computed, ref, watch } from "vue";
import { Plus, X } from "lucide-vue-next";
import type { BuilderEquipment } from "../composables/useRecipeBuilder";

/** A step's tools and vessels, each a role paired with a resource symbol. */
const props = defineProps<{
  bindings: BuilderEquipment[];
  options: string[];
  disabled?: boolean;
  listId: string;
  /** Create a missing tool/vessel and return its symbol. */
  createResource?: (name: string, role: string) => string | undefined;
}>();

const emit = defineEmits<{ commit: [bindings: BuilderEquipment[]] }>();

const ROLES = ["tool", "container", "equipment", "target"];
const local = ref<BuilderEquipment[]>(props.bindings.map((binding) => ({ ...binding })));
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
  local.value = [...local.value, { role: "tool", symbol: "" }];
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
  const binding = local.value[index];
  const name = unknownName(binding?.symbol ?? "");
  if (!name || !props.createResource || !binding) return;
  const symbol = props.createResource(name, binding.role);
  if (!symbol) return;
  local.value = local.value.map((row, i) => (i === index ? { ...row, symbol } : row));
  commit();
}

function onSymbolEnter(index: number, event: KeyboardEvent): void {
  if (!unknownName(local.value[index]?.symbol ?? "")) return;
  event.preventDefault();
  createAt(index);
}

function kindLabel(role: string): string {
  if (role === "container" || role === "target") return "container";
  return "equipment";
}
</script>

<template>
  <div ref="root" class="equipment-editor">
    <span class="editor-label">Tools &amp; vessels</span>
    <datalist :id="listId">
      <option v-for="option in options" :key="option" :value="option" />
    </datalist>
    <div v-for="(binding, index) in local" :key="index" class="equipment-block">
      <div class="equipment-row" :class="{ unknown: !!unknownName(binding.symbol) }">
        <select v-model="binding.role" :disabled="disabled" aria-label="Role" @change="commit">
          <option v-for="role in ROLES" :key="role" :value="role">{{ role }}</option>
        </select>
        <input
          v-model="binding.symbol"
          :list="listId"
          :disabled="disabled"
          placeholder="resource"
          aria-label="Resource"
          @change="commit"
          @keydown.enter="onSymbolEnter(index, $event)"
        />
        <button class="icon" title="Remove" :disabled="disabled" @click="removeAt(index)">
          <X :size="14" />
        </button>
      </div>
      <button
        v-if="createResource && unknownName(binding.symbol)"
        type="button"
        class="create-btn"
        :disabled="disabled"
        @click="createAt(index)"
      >
        <Plus :size="13" /> Add “{{ unknownName(binding.symbol) }}” as
        {{ kindLabel(binding.role) }}
      </button>
    </div>
    <button class="add-row" :disabled="disabled" @click="add"><Plus :size="14" /> Add tool</button>
  </div>
</template>

<style scoped>
.equipment-editor {
  display: grid;
  gap: 6px;
}
.editor-label {
  font-size: 12px;
  color: #657169;
}
.equipment-block {
  display: grid;
  gap: 5px;
}
.equipment-row {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 6px;
}
.equipment-row.unknown input {
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
