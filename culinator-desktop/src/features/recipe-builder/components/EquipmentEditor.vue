<script setup lang="ts">
/* global HTMLElement */
import { ref, watch } from "vue";
import { Plus, X } from "lucide-vue-next";
import type { BuilderEquipment } from "../composables/useRecipeBuilder";

/** A step's tools and vessels, each a role paired with a resource symbol. */
const props = defineProps<{
  bindings: BuilderEquipment[];
  options: string[];
  disabled?: boolean;
  listId: string;
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
</script>

<template>
  <div ref="root" class="equipment-editor">
    <span class="editor-label">Tools &amp; vessels</span>
    <datalist :id="listId">
      <option v-for="option in options" :key="option" :value="option" />
    </datalist>
    <div v-for="(binding, index) in local" :key="index" class="equipment-row">
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
      />
      <button class="icon" title="Remove" :disabled="disabled" @click="removeAt(index)">
        <X :size="14" />
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
.equipment-row {
  display: grid;
  grid-template-columns: auto 1fr auto;
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
