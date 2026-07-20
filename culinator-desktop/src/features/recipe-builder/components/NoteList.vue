<script setup lang="ts">
/* global HTMLElement */
import { ref, watch } from "vue";
import { Plus, X } from "lucide-vue-next";

/**
 * An editable list of free-text notes, committed as a whole array.
 *
 * Notes are a repeatable statement (`note "…";`), so the list is regenerated on
 * every change rather than patched entry by entry. Edits commit on blur/Enter,
 * and the local copy is only re-seeded from the model when focus is elsewhere,
 * so a note doesn't reset mid-sentence.
 */
const props = defineProps<{ notes: string[]; label?: string; disabled?: boolean }>();
const emit = defineEmits<{ commit: [notes: string[]] }>();

const local = ref<string[]>([...props.notes]);
const root = ref<HTMLElement>();

watch(
  () => props.notes,
  (value) => {
    if (!root.value?.contains(document.activeElement)) local.value = [...value];
  },
);

function commit(): void {
  emit("commit", local.value.map((note) => note.trim()).filter(Boolean));
}
function add(): void {
  local.value = [...local.value, ""];
}
function removeAt(index: number): void {
  local.value = local.value.filter((_, i) => i !== index);
  commit();
}
</script>

<template>
  <div ref="root" class="note-list">
    <span v-if="label" class="note-label">{{ label }}</span>
    <div v-for="(note, index) in local" :key="index" class="note-row">
      <input v-model="local[index]" :disabled="disabled" @change="commit" />
      <button class="icon" title="Remove note" :disabled="disabled" @click="removeAt(index)">
        <X :size="14" />
      </button>
    </div>
    <button class="add-note" :disabled="disabled" @click="add"><Plus :size="14" /> Add note</button>
  </div>
</template>

<style scoped>
.note-list {
  display: grid;
  gap: 6px;
}
.note-label {
  font-size: 12px;
  color: #657169;
}
.note-row {
  display: flex;
  gap: 6px;
}
.note-row input {
  flex: 1;
}
.icon {
  width: 34px;
  padding: 0;
  display: grid;
  place-items: center;
}
.add-note {
  justify-self: start;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  padding: 5px 10px;
}
</style>
