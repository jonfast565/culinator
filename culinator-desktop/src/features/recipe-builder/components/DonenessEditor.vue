<script setup lang="ts">
/* global HTMLElement */
import { ref, watch } from "vue";
import { Plus, X } from "lucide-vue-next";
import type { BuilderDoneness } from "../composables/useRecipeBuilder";

/**
 * "Cook until…" cues: a kind and a value. A temperature cue carries a bare
 * quantity (`internal_temp 165 fahrenheit`); the rest are free-text phrases the
 * composable quotes.
 */
const props = defineProps<{ cues: BuilderDoneness[]; disabled?: boolean }>();
const emit = defineEmits<{ commit: [cues: BuilderDoneness[]] }>();

const KINDS = ["internal_temp", "visual", "tester", "texture", "rise"];
const local = ref<BuilderDoneness[]>(props.cues.map((cue) => ({ ...cue })));
const root = ref<HTMLElement>();

watch(
  () => props.cues,
  (value) => {
    if (!root.value?.contains(document.activeElement)) {
      local.value = value.map((cue) => ({ ...cue }));
    }
  },
);

function commit(): void {
  emit(
    "commit",
    local.value.filter((cue) => cue.kind.trim() && cue.value.trim()),
  );
}
function add(): void {
  local.value = [...local.value, { kind: "visual", value: "" }];
}
function removeAt(index: number): void {
  local.value = local.value.filter((_, i) => i !== index);
  commit();
}
</script>

<template>
  <div ref="root" class="doneness-editor">
    <span class="editor-label">Cook until</span>
    <div v-for="(cue, index) in local" :key="index" class="doneness-row">
      <select v-model="cue.kind" :disabled="disabled" aria-label="Cue kind" @change="commit">
        <option v-for="kind in KINDS" :key="kind" :value="kind">
          {{ kind.replace("_", " ") }}
        </option>
      </select>
      <input
        v-model="cue.value"
        :disabled="disabled"
        :placeholder="cue.kind === 'internal_temp' ? '165 fahrenheit' : 'golden brown'"
        aria-label="Cue value"
        @change="commit"
      />
      <button class="icon" title="Remove" :disabled="disabled" @click="removeAt(index)">
        <X :size="14" />
      </button>
    </div>
    <button class="add-row" :disabled="disabled" @click="add"><Plus :size="14" /> Add cue</button>
  </div>
</template>

<style scoped>
.doneness-editor {
  display: grid;
  gap: 6px;
}
.editor-label {
  font-size: 12px;
  color: #657169;
}
.doneness-row {
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
