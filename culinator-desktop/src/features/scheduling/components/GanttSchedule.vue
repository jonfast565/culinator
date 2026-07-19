<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { RecipeSchedule } from "../../../domain/types";
import { scheduleRecipe } from "../../../services/api";

const props = defineProps<{ source: string }>();
const schedule = ref<RecipeSchedule | null>(null);
const error = ref("");
const zoom = ref(1);
let timer: number | undefined;

watch(
  () => props.source,
  () => {
    window.clearTimeout(timer);
    timer = window.setTimeout(async () => {
      try {
        schedule.value = await scheduleRecipe(props.source);
        error.value = "";
      } catch (e) {
        error.value = e instanceof Error ? e.message : String(e);
      }
    }, 250);
  },
  { immediate: true },
);

const width = computed(() =>
  Math.max(640, ((schedule.value?.makespanSeconds ?? 0) / 60) * 14 * zoom.value),
);

const activeLaborSeconds = computed(() => {
  if (!schedule.value) return 0;
  return schedule.value.operations
    .filter((item) => item.labor === "active")
    .reduce((total, item) => total + item.durationSeconds, 0);
});

const firstStep = computed(() => schedule.value?.operations[0] ?? null);

function minutes(seconds: number): number {
  return Math.round(seconds / 60);
}

function formatAction(item: RecipeSchedule["operations"][number]): string {
  const label = item.action.replaceAll("_", " ");
  return `${label} · ${minutes(item.durationSeconds)} min`;
}
</script>

<template>
  <section class="panel space-y-3">
    <div class="flex items-center justify-between">
      <div>
        <h3>Production schedule</h3>
        <p v-if="schedule" class="schedule-summary">
          About <strong>{{ minutes(schedule.makespanSeconds) }} min</strong> total ·
          <strong>{{ minutes(activeLaborSeconds) }} min</strong> hands-on
          <template v-if="firstStep">
            · start with <strong>{{ firstStep.action.replaceAll("_", " ") }}</strong>
          </template>
        </p>
      </div>
      <input v-model.number="zoom" type="range" min="0.5" max="3" step="0.25" aria-label="Zoom" />
    </div>
    <p v-if="error" class="diagnostic error">{{ error }}</p>
    <div v-else class="overflow-x-auto">
      <div class="gantt" :style="{ width: `${width}px` }">
        <div v-for="item in schedule?.operations" :key="item.symbol" class="gantt-row">
          <div class="gantt-label">
            <strong>{{ formatAction(item) }}</strong>
            <small>{{ item.process }}</small>
          </div>
          <div class="gantt-track">
            <div
              class="gantt-bar"
              :class="{ critical: schedule?.criticalPath.includes(item.symbol) }"
              :style="{
                left: `${(item.startSeconds / Math.max(1, schedule?.makespanSeconds ?? 1)) * 100}%`,
                width: `${(item.durationSeconds / Math.max(1, schedule?.makespanSeconds ?? 1)) * 100}%`,
              }"
              :title="`${minutes(item.startSeconds)}–${minutes(item.endSeconds)} min`"
            >
              <span>{{ minutes(item.durationSeconds) }}m</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.schedule-summary {
  margin: 4px 0 0;
  font-size: 13px;
  color: #4a5a52;
  line-height: 1.4;
}
.gantt-row {
  display: grid;
  grid-template-columns: 12rem 1fr;
  min-height: 42px;
  border-bottom: 1px solid #e5e7eb;
}
.gantt-label {
  padding: 0.4rem;
  display: flex;
  flex-direction: column;
}
.gantt-label strong {
  font-size: 13px;
  font-weight: 600;
  text-transform: capitalize;
}
.gantt-track {
  position: relative;
  background: repeating-linear-gradient(90deg, #fafafa 0, #fafafa 69px, #e5e7eb 70px);
  margin: 0.3rem;
}
.gantt-bar {
  position: absolute;
  top: 0.25rem;
  bottom: 0.25rem;
  min-width: 3px;
  border-radius: 0.35rem;
  background: #6b8e65;
  color: white;
  padding: 0.2rem 0.35rem;
  overflow: hidden;
}
.gantt-bar.critical {
  background: #9f3a38;
}
</style>
