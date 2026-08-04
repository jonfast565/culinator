<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import { Codemirror } from "vue-codemirror";
import { basicSetup } from "codemirror";
import { HighlightStyle, StreamLanguage, syntaxHighlighting } from "@codemirror/language";
import { simpleMode } from "@codemirror/legacy-modes/mode/simple-mode";
import { tags } from "@lezer/highlight";
import { EditorView } from "@codemirror/view";
import type { Diagnostic } from "../../../domain/types";
import type { UiRecipeModel } from "../model";
import { referenceNavigation, setEditorReferenceIndex } from "../referenceNavigation";
import { buildSymbolReferenceIndex } from "../symbolReferences";

const props = defineProps<{
  modelValue: string;
  model?: UiRecipeModel | null;
  diagnostics?: Diagnostic[];
}>();
const emit = defineEmits<{ "update:modelValue": [value: string] }>();

const view = shallowRef<EditorView>();

/**
 * Structural and statement keywords from docs/GRAMMAR.ebnf. Kept as one list so
 * the source editor highlights the same vocabulary the parser recognizes.
 */
const KEYWORDS = [
  "culinator",
  "book",
  "recipe",
  "type",
  "resource",
  "ingredient",
  "material",
  "container",
  "equipment",
  "environment",
  "labor",
  "process",
  "operation",
  "prep",
  "yield",
  "serving",
  "formula",
  "as",
  "measured",
  "by",
  "relative",
  "to",
  "of",
  "total",
  "does",
  "into",
  "after",
  "input",
  "output",
  "produces",
  "target",
  "tool",
  "requires",
  "duration",
  "estimated",
  "up",
  "temperature",
  "heat",
  "until",
  "optional",
  "repeat",
  "note",
  "title",
  "section",
  "description",
  "source",
  "publisher",
  "source_url",
  "attribution",
  "active_time",
  "total_time",
  "image",
  "photo",
  "name",
  "quantity",
  "amount",
  "percentage",
  "reference",
  "state",
  "divided",
  "substitutes",
  "to_taste",
  "size",
  "variant",
  "allergen",
].join("|");

const ATOMS = [
  "mass",
  "volume",
  "count",
  "time",
  "temperature",
  "length",
  "area",
  "energy",
  "ratio",
  "concentration",
  "active",
  "passive",
  "monitor",
  "automated",
  "low",
  "medium_low",
  "medium",
  "medium_high",
  "high",
  "internal_temp",
  "visual",
  "tester",
  "texture",
  "rise",
  "true",
  "false",
  "start_start",
  "finish_finish",
  "start_finish",
  "lag",
].join("|");

/** Time units from docs/GRAMMAR.ebnf — matched with a leading number as one duration token. */
const TIME_UNITS = [
  "seconds?",
  "secs?",
  "s",
  "minutes?",
  "mins?",
  "min",
  "hours?",
  "hrs?",
  "hr",
  "h",
  "days?",
  "weeks?",
  "wks?",
  "wk",
].join("|");

const highlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: "#28643b", fontWeight: "600" },
  { tag: tags.atom, color: "#5d4a8a" },
  { tag: tags.string, color: "#8a5e10" },
  { tag: tags.number, color: "#1a5f8a", fontWeight: "600" },
  { tag: tags.unit, color: "#a14d1a", fontWeight: "600" },
  { tag: tags.comment, color: "#8a938c", fontStyle: "italic" },
]);

const extensions = [
  basicSetup,
  StreamLanguage.define({
    ...simpleMode({
      start: [
        { regex: "//.*$", token: "comment" },
        { regex: '"(?:[^\\\\"]|\\\\.)*"', token: "string" },
        { regex: `\\b(?:${KEYWORDS})\\b`, token: "keyword" },
        { regex: `\\b(?:${ATOMS})\\b`, token: "atom" },
        // `5 min`, `90 seconds`, `1.5 h` — before bare numbers so the unit tags along.
        {
          regex: new RegExp(`\\b\\d+(?:\\.\\d+)?\\s+(?:${TIME_UNITS})\\b`),
          token: "duration",
        },
        { regex: "\\b\\d+(?:\\.\\d+)?%?\\b", token: "number" },
      ],
    }),
    tokenTable: {
      duration: tags.unit,
    },
  }),
  syntaxHighlighting(highlightStyle),
  referenceNavigation(),
  EditorView.theme({
    "&": { backgroundColor: "#fbfbf9", height: "100%" },
    ".cm-content": { caretColor: "#27342d" },
    "&.cm-focused .cm-cursor": { borderLeftColor: "#28643b" },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
      backgroundColor: "rgb(40 100 59 / 16%)",
    },
  }),
];

const value = computed({
  get: () => props.modelValue,
  set: (next) => emit("update:modelValue", next),
});

function refreshReferenceIndex(): void {
  const editor = view.value;
  if (!editor || !props.model) return;
  setEditorReferenceIndex(editor, buildSymbolReferenceIndex(props.modelValue, props.model));
}

function onReady(payload: { view: EditorView }): void {
  view.value = payload.view;
  refreshReferenceIndex();
}

watch(
  () => [props.modelValue, props.model] as const,
  () => refreshReferenceIndex(),
);

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
  <div class="source-editor-shell">
    <Codemirror
      v-model="value"
      class="source-editor"
      :extensions="extensions"
      :indent-with-tab="true"
      @ready="onReady"
    />
    <p class="reference-hint">
      Recipe names are colored by kind. Click to highlight binding and uses; Ctrl/⌘-click jumps to
      the definition.
    </p>
  </div>
</template>

<style scoped>
.source-editor-shell {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
}
.source-editor {
  flex: 1;
  min-height: 0;
}
.reference-hint {
  flex: 0 0 auto;
  margin: 0;
  padding: 6px 12px;
  border-top: 1px solid #e2e6e1;
  background: #f3f5f2;
  color: #6d7972;
  font-size: 11px;
}
</style>
