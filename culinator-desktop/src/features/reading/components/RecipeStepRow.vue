<script setup lang="ts">
import { Clock, Trash2 } from "lucide-vue-next";
import type { UiOperation } from "../../recipe-editor/model";
import RecipeImage from "./RecipeImage.vue";

defineProps<{
  number: number;
  operation?: UiOperation;
  text: string;
  meta?: string;
  time?: string;
  recipeId?: string;
  editable?: boolean;
}>();

const emit = defineEmits<{ delete: [] }>();
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
      <span v-if="time" class="step-time"><Clock :size="12" />{{ time }}</span>
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
  font-size: 26px;
  font-weight: 600;
  line-height: 1;
  color: #28643b;
  text-align: right;
  font-variant-numeric: lining-nums;
}
.step-text {
  margin: 0;
  font-size: 16px;
  line-height: 1.55;
}
.step-meta {
  display: block;
  margin-top: 4px;
  font-size: 12px;
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
  font-size: 12px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
.step-time svg {
  opacity: 0.7;
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
