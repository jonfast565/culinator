<script setup lang="ts">
/* global HTMLInputElement, HTMLTextAreaElement */
import { ref, watch } from "vue";

/**
 * A labelled text field that commits on change, not on every keystroke, and
 * refuses to overwrite itself while focused.
 *
 * Every builder edit rewrites the source, which reparses and pushes a fresh
 * value back down. Binding straight to that value would reset the input
 * mid-keystroke and bounce the caret to the end. So the field keeps a local
 * copy, emits `commit` on blur/Enter (`@change`), and only accepts a pushed
 * value back when the element is not the active one — which is exactly when a
 * change came from elsewhere (an undo, a source-editor edit) rather than from
 * this field.
 */
const props = defineProps<{
  label: string;
  modelValue: string;
  placeholder?: string;
  type?: string;
  multiline?: boolean;
  disabled?: boolean;
  /** id of a `<datalist>` to offer autocomplete suggestions. */
  list?: string;
}>();

const emit = defineEmits<{ commit: [value: string] }>();

const local = ref(props.modelValue);
const field = ref<HTMLInputElement | HTMLTextAreaElement>();

watch(
  () => props.modelValue,
  (value) => {
    if (document.activeElement !== field.value && value !== local.value) local.value = value;
  },
);

function commit(): void {
  if (local.value !== props.modelValue) emit("commit", local.value);
}
</script>

<template>
  <label class="builder-field">
    <span>{{ label }}</span>
    <textarea
      v-if="multiline"
      ref="field"
      v-model="local"
      :placeholder="placeholder"
      :disabled="disabled"
      rows="2"
      @change="commit"
    />
    <input
      v-else
      ref="field"
      v-model="local"
      :type="type ?? 'text'"
      :placeholder="placeholder"
      :disabled="disabled"
      :list="list"
      @change="commit"
    />
  </label>
</template>

<style scoped>
.builder-field {
  display: grid;
  gap: 5px;
  margin: 0;
  font-size: 12px;
  color: #657169;
}
.builder-field textarea {
  width: 100%;
  resize: vertical;
  font: inherit;
}
.builder-field input {
  width: 100%;
}
</style>
