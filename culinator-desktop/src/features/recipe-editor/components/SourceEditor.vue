<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { Codemirror } from "vue-codemirror";
import { StreamLanguage } from "@codemirror/language";
import { simpleMode } from "@codemirror/legacy-modes/mode/simple-mode";
import { EditorView } from "@codemirror/view";
import type { Diagnostic } from "../../../domain/types";

const props = defineProps<{ modelValue: string; diagnostics?: Diagnostic[] }>();
const emit = defineEmits<{ "update:modelValue": [value: string] }>();

const view = shallowRef<EditorView>();

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

function onReady(payload: { view: EditorView }): void {
  view.value = payload.view;
}

function jumpToOffset(offset: number): void {
  const editor = view.value;
  if (!editor) return;
  const pos = Math.max(0, Math.min(offset, props.modelValue.length));
  editor.dispatch({
    selection: { anchor: pos, head: pos },
    effects: EditorView.scrollIntoView(pos, { y: "center" }),
  });
  editor.focus();
}

function diagnosticLine(diagnostic: Diagnostic): number | null {
  if (diagnostic.start == null) return null;
  return props.modelValue.slice(0, diagnostic.start).split("\n").length;
}

defineExpose({ jumpToOffset, diagnosticLine });
</script>

<template>
  <Codemirror
    v-model="value"
    class="source-editor"
    :extensions="extensions"
    :indent-with-tab="true"
    @ready="onReady"
  />
</template>
