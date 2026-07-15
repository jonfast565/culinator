<script setup lang="ts">
import { computed, ref } from "vue";
import { Plus, Trash2 } from "lucide-vue-next";
import type { UiRecipeModel } from "../../recipe-editor/model";

const props = defineProps<{ source: string; model: UiRecipeModel }>();
const emit = defineEmits<{ "update:source": [value: string] }>();
const selected = ref<string | null>(null);
const selectedOperation = computed(() =>
  props.model.operations.find((item) => item.symbol === selected.value),
);

function eventValue(event: unknown): string {
  return (event as { target?: { value?: string } }).target?.value ?? "";
}
function replaceRange(start: number, end: number, text: string): void {
  emit("update:source", props.source.slice(0, start) + text + props.source.slice(end));
}
function renameTitle(event: unknown): void {
  const value = eventValue(event).replaceAll('"', '\\"');
  const regex = /\btitle\s+"[^"]*"\s*;/;
  emit(
    "update:source",
    regex.test(props.source)
      ? props.source.replace(regex, `title "${value}";`)
      : props.source.replace(/(recipe\s+\w+[^{}]*{)/, `$1\n    title "${value}";`),
  );
}
function updateOperation(field: "duration" | "labor", value: string): void {
  const operation = selectedOperation.value;
  if (!operation?.range) return;
  let block = props.source.slice(operation.range.start, operation.range.end);
  const pattern = field === "duration" ? /\bduration\s+[^;]+;/ : /\blabor\s+\w+\s*;/;
  const line = field === "duration" ? `duration ${value || "5 min"};` : `labor ${value};`;
  block = pattern.test(block)
    ? block.replace(pattern, line)
    : block.replace(/{/, `{\n            ${line}`);
  replaceRange(operation.range.start, operation.range.end, block);
}
function deleteOperation(): void {
  const operation = selectedOperation.value;
  if (operation?.range && window.confirm(`Delete ${operation.symbol}?`)) {
    replaceRange(operation.range.start, operation.range.end, "");
    selected.value = null;
  }
}
function addOperation(): void {
  const symbol = `operation_${props.model.operations.length + 1}`;
  const snippet = `\n    process visual_workflow {\n        operation ${symbol} does prepare {\n            duration 5 min;\n            labor active;\n        }\n    }\n`;
  emit("update:source", `${props.source.trimEnd()}\n${snippet}`);
  selected.value = symbol;
}
</script>

<template>
  <section class="panel space-y-4">
    <div>
      <label class="text-xs font-semibold uppercase tracking-wide">Recipe title</label>
      <input class="mt-1 w-full rounded border p-2" :value="model.title" @change="renameTitle" />
    </div>
    <div class="flex items-center justify-between">
      <h3>Visual workflow</h3>
      <button @click="addOperation"><Plus :size="14" /> Operation</button>
    </div>
    <div class="grid gap-2">
      <button
        v-for="operation in model.operations"
        :key="operation.symbol"
        class="card text-left"
        :class="{ 'ring-2 ring-herb': selected === operation.symbol }"
        @click="selected = operation.symbol"
      >
        <strong>{{ operation.symbol }}</strong>
        <small
          >{{ operation.action }} · {{ operation.durationMinutes }} min ·
          {{ operation.labor }}</small
        >
      </button>
    </div>
    <div v-if="selectedOperation" class="card space-y-3">
      <h4>{{ selectedOperation.symbol }}</h4>
      <label>
        Duration
        <input
          class="w-full rounded border p-2"
          :value="`${selectedOperation.durationMinutes} min`"
          @change="updateOperation('duration', eventValue($event))"
        />
      </label>
      <label>
        Labor
        <select
          class="w-full rounded border p-2"
          :value="selectedOperation.labor"
          @change="updateOperation('labor', eventValue($event))"
        >
          <option v-for="mode in ['active', 'passive', 'monitor', 'automated']" :key="mode">
            {{ mode }}
          </option>
        </select>
      </label>
      <p class="text-xs">Depends on: {{ selectedOperation.after.join(", ") || "nothing" }}</p>
      <button class="danger" @click="deleteOperation">
        <Trash2 :size="14" /> Delete operation
      </button>
    </div>
  </section>
</template>
