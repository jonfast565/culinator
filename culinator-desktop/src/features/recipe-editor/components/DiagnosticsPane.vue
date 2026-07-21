<script setup lang="ts">
import { CheckCircle2, TriangleAlert } from "lucide-vue-next";
import type { Diagnostic } from "../../../domain/types";

const props = defineProps<{
  diagnostics: Diagnostic[];
  source: string;
}>();

const emit = defineEmits<{
  select: [diagnostic: Diagnostic];
}>();

function lineFor(diagnostic: Diagnostic): number | null {
  if (diagnostic.start == null) return null;
  return props.source.slice(0, diagnostic.start).split("\n").length;
}
</script>

<template>
  <section class="diagnostics-pane" aria-label="Recipe diagnostics">
    <header>
      <span>Issues</span>
      <small>{{ diagnostics.length || "None" }}</small>
    </header>
    <div v-if="!diagnostics.length" class="clean">
      <CheckCircle2 :size="15" />
      No syntax issues
    </div>
    <ul v-else>
      <li
        v-for="(item, index) in diagnostics"
        :key="`${item.message}-${index}`"
        :class="item.severity"
      >
        <button type="button" @click="emit('select', item)">
          <TriangleAlert :size="14" />
          <strong>{{ item.severity }}</strong>
          <span>{{ item.message }}</span>
          <small v-if="lineFor(item)">Line {{ lineFor(item) }}</small>
        </button>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.diagnostics-pane {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: #f7f6f2;
}
header {
  flex: 0 0 auto;
  height: 34px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  border-bottom: 1px solid #dde1dc;
  background: #fff;
  color: #45524b;
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
header small {
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0;
  text-transform: none;
}
.clean {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 12px;
  color: #4f765b;
  font-size: 12px;
}
ul {
  min-height: 0;
  overflow: auto;
  list-style: none;
  margin: 0;
  padding: 0;
}
li button {
  width: 100%;
  display: grid;
  grid-template-columns: 16px auto minmax(0, 1fr) auto;
  align-items: start;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid #e7eae6;
  background: transparent;
  color: #59635d;
  font-size: 12px;
  text-align: left;
}
li button:hover {
  background: #eef1ed;
}
li.error button {
  color: #a83737;
}
li.warning button {
  color: #8a6d1f;
}
li strong {
  font-size: 10px;
  line-height: 17px;
  text-transform: uppercase;
}
li span {
  line-height: 17px;
}
li small {
  line-height: 17px;
  white-space: nowrap;
}
</style>
