<script setup lang="ts">
import { Check, Clock, Play, Trash2 } from "lucide-vue-next";
import type { TryOperation } from "../../../domain/types";
import type { LiveTimerState } from "../../kitchen-mode/composables/useKitchenExecution";
import type { UiOperation } from "../../recipe-editor/model";
import RecipeImage from "./RecipeImage.vue";

const props = defineProps<{
  number: number;
  operation?: UiOperation;
  text: string;
  meta?: string;
  time?: string;
  recipeId?: string;
  editable?: boolean;
  kitchenOperation?: TryOperation;
  kitchenTimer?: LiveTimerState;
}>();

const emit = defineEmits<{
  delete: [];
  "start-timer": [operation: TryOperation];
  "complete-timer": [operation: TryOperation];
}>();

function formatClock(totalSeconds: number): string {
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function timerText(): string {
  if (!props.kitchenTimer) return "";
  if (props.kitchenTimer.overdue) {
    return `+${formatClock(props.kitchenTimer.elapsedSeconds - props.kitchenTimer.durationSeconds)}`;
  }
  return formatClock(props.kitchenTimer.remainingSeconds);
}
</script>

<template>
  <div class="step">
    <span class="step-number">{{ number }}</span>
    <div class="step-body">
      <p class="step-text">{{ text }}</p>
      <small v-if="meta" class="step-meta">{{ meta }}</small>
      <figure v-if="operation?.photo" class="step-photo">
        <RecipeImage :image-ref="operation.photo" :recipe-id="recipeId" />
      </figure>
    </div>
    <div class="step-aside">
      <template v-if="kitchenOperation">
        <button
          v-if="kitchenOperation.status === 'pending'"
          type="button"
          class="step-timer start"
          :aria-label="`Start timer for step ${number}`"
          @click="emit('start-timer', kitchenOperation)"
        >
          <Play :size="12" />
          {{ formatClock(kitchenOperation.durationSeconds) }}
        </button>
        <button
          v-else-if="kitchenOperation.status === 'active'"
          type="button"
          class="step-timer running"
          :class="{ overdue: kitchenTimer?.overdue }"
          :aria-label="`Complete step ${number}`"
          @click="emit('complete-timer', kitchenOperation)"
        >
          <Clock :size="12" />
          <strong>{{ timerText() }}</strong>
          <span>Done</span>
        </button>
        <span v-else-if="kitchenOperation.status === 'completed'" class="step-timer complete">
          <Check :size="13" /> Done
        </span>
        <span v-else class="step-timer skipped">Skipped</span>
      </template>
      <span v-else-if="time" class="step-time"><Clock :size="12" />{{ time }}</span>
      <button
        v-if="editable"
        type="button"
        class="step-delete"
        title="Delete step"
        @click="emit('delete')"
      >
        <Trash2 :size="14" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.step {
  display: grid;
  grid-template-columns: minmax(4.5rem, 6.5rem) 1fr auto;
  gap: 12px;
  align-items: start;
}
.step-number {
  font-family: "Iowan Old Style", "Palatino Linotype", Palatino, Georgia, serif;
  font-size: calc(26px * var(--reading-scale, 1));
  font-weight: 600;
  line-height: 1;
  color: #28643b;
  text-align: right;
  font-variant-numeric: lining-nums;
}
.step-text {
  margin: 0;
  font-size: calc(16px * var(--reading-scale, 1));
  line-height: 1.55;
}
.step-meta {
  display: block;
  margin-top: 4px;
  font-size: calc(12px * var(--reading-scale, 1));
  text-transform: capitalize;
  color: #6d7972;
}
.step-photo {
  margin: 12px 0 2px;
  max-width: 340px;
  aspect-ratio: 4 / 3;
  overflow: hidden;
  border-radius: 4px;
  box-shadow: 0 8px 20px -14px rgba(40, 40, 30, 0.5);
}
.step-aside {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 6px;
  min-width: 2.5rem;
}
.step-time {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  white-space: nowrap;
  padding: 3px 11px;
  border-radius: 999px;
  background: #e8f0e6;
  color: #28643b;
  font-size: calc(12px * var(--reading-scale, 1));
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
.step-time svg {
  opacity: 0.7;
}
.step-timer {
  min-width: 78px;
  min-height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  white-space: nowrap;
  padding: 5px 10px;
  border: 1px solid #b9cbbd;
  border-radius: 8px;
  background: #f1f6ef;
  color: #245c36;
  font-size: calc(12px * var(--reading-scale, 1));
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}
button.step-timer:hover {
  border-color: #28643b;
  background: #e3eee0;
}
.step-timer.running {
  flex-wrap: wrap;
  border-color: #28643b;
  background: #28643b;
  color: white;
}
.step-timer.running span {
  width: 100%;
  padding-top: 3px;
  border-top: 1px solid rgba(255, 255, 255, 0.25);
  font-size: 9px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}
.step-timer.running.overdue {
  border-color: #a94b32;
  background: #a94b32;
}
.step-timer.complete {
  border-color: transparent;
  background: #e6eee3;
}
.step-timer.skipped {
  border-color: transparent;
  background: transparent;
  color: #89918c;
  font-weight: 500;
}
.step-delete {
  display: grid;
  place-items: center;
  width: 30px;
  height: 30px;
  padding: 0;
  border-radius: 7px;
  border: 1px solid #e2d8d8;
  background: #fff;
  color: #a83737;
}
.step-delete:hover {
  background: #fbeceb;
  border-color: #e8b4b4;
}
</style>
