<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { CheckCircle2, ChevronDown, ChevronRight, TriangleAlert } from "lucide-vue-next";
import type { Diagnostic } from "../../../domain/types";

const props = defineProps<{
  diagnostics: Diagnostic[];
  source: string;
}>();

const emit = defineEmits<{
  select: [diagnostic: Diagnostic];
  "update:expanded": [value: boolean];
}>();

const errorCount = computed(
  () => props.diagnostics.filter((item) => item.severity === "error").length,
);
const warningCount = computed(
  () => props.diagnostics.filter((item) => item.severity === "warning").length,
);
const hasIssues = computed(() => props.diagnostics.length > 0);

/** Collapsed by default when clean; opens automatically when issues appear. */
const expanded = ref(
  typeof window !== "undefined"
    ? window.localStorage.getItem("cg:diagnostics-expanded") === "1" || hasIssues.value
    : hasIssues.value,
);

watch(hasIssues, (next) => {
  if (next) setExpanded(true);
});

watch(
  expanded,
  (next) => {
    emit("update:expanded", next);
  },
  { immediate: true },
);

function setExpanded(value: boolean): void {
  expanded.value = value;
  window.localStorage.setItem("cg:diagnostics-expanded", value ? "1" : "0");
}

function toggle(): void {
  setExpanded(!expanded.value);
}

function lineFor(diagnostic: Diagnostic): number | null {
  if (diagnostic.start == null) return null;
  return props.source.slice(0, diagnostic.start).split("\n").length;
}
</script>

<template>
  <section
    class="diagnostics-pane"
    :class="{ collapsed: !expanded, clean: !hasIssues }"
    aria-label="Recipe diagnostics"
  >
    <header>
      <button type="button" class="toggle" :aria-expanded="expanded" @click="toggle">
        <ChevronDown v-if="expanded" :size="14" />
        <ChevronRight v-else :size="14" />
        <span>Issues</span>
      </button>
      <div class="counts">
        <template v-if="hasIssues">
          <span v-if="errorCount" class="badge error"
            >{{ errorCount }} error{{ errorCount === 1 ? "" : "s" }}</span
          >
          <span v-if="warningCount" class="badge warning"
            >{{ warningCount }} warning{{ warningCount === 1 ? "" : "s" }}</span
          >
          <span v-if="!errorCount && !warningCount" class="badge">{{ diagnostics.length }}</span>
        </template>
        <span v-else class="badge ok"><CheckCircle2 :size="12" /> Clean</span>
      </div>
    </header>
    <div v-if="expanded && !hasIssues" class="clean-body">
      <CheckCircle2 :size="15" />
      No syntax issues
    </div>
    <ul v-else-if="expanded">
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
.diagnostics-pane.collapsed {
  height: 34px;
}
header {
  flex: 0 0 auto;
  min-height: 34px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 0 8px 0 4px;
  border-bottom: 1px solid #dde1dc;
  background: #fff;
  color: #45524b;
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.toggle {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 28px;
  padding: 0 6px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: inherit;
  font: inherit;
  letter-spacing: inherit;
  text-transform: inherit;
  cursor: pointer;
}
.toggle:hover {
  background: #eef1ed;
}
.counts {
  display: flex;
  align-items: center;
  gap: 6px;
}
.badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 7px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.02em;
  text-transform: none;
  background: #eef1ec;
  color: #55635b;
}
.badge.error {
  background: #f8e8e8;
  color: #a83737;
}
.badge.warning {
  background: #fbf3e0;
  color: #8a6d1f;
}
.badge.ok {
  background: #e4efe6;
  color: #28643b;
}
.clean-body {
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
