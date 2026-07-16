<script setup lang="ts">
import { computed } from "vue";
import { Codemirror } from "vue-codemirror";
import { StreamLanguage } from "@codemirror/language";
import { simpleMode } from "@codemirror/legacy-modes/mode/simple-mode";
const props = defineProps<{ modelValue: string }>();
const emit = defineEmits<{ "update:modelValue": [value: string] }>();
const extensions = [
  StreamLanguage.define(
    simpleMode({
      start: [
        {
          regex:
            /\b(culinator|book|recipe|ingredient|material|container|equipment|process|operation|yield|serving|formula|measured|by|relative|to|does|after|duration|labor|title|quantity|percentage|reference)\b/,
          token: "keyword",
        },
        { regex: /"(?:[^\\"]|\\.)*"/, token: "string" },
        { regex: /\b\d+(?:\.\d+)?%?\b/, token: "number" },
        { regex: /\/\/.*$/, token: "comment" },
      ],
    }),
  ),
];
const value = computed({
  get: () => props.modelValue,
  set: (next) => emit("update:modelValue", next),
});
</script>
<template>
  <Codemirror
    v-model="value"
    class="source-editor"
    :extensions="extensions"
    :indent-with-tab="true"
  />
</template>
