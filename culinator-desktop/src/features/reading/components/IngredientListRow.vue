<script setup lang="ts">
import type { IngredientDisplayParts } from "../../recipe-editor/narrative";

defineProps<{
  parts: IngredientDisplayParts;
  selectable?: boolean;
  highlighted?: boolean;
  bakerPercent?: number | null;
}>();
const emit = defineEmits<{ select: [symbol: string] }>();

function formatBaker(value: number): string {
  const rounded = Math.round(value * 10) / 10;
  return `${rounded}%`;
}
</script>

<template>
  <li
    class="ingredient-row"
    :class="{ selectable, highlighted }"
    :data-preview-symbol="parts.symbol"
    @click="selectable && parts.symbol && emit('select', parts.symbol)"
  >
    <span class="ingredient-qty">{{ parts.amount }}</span>
    <span class="ingredient-body">
      <span class="ingredient-name">{{ parts.description }}</span>
      <span v-if="bakerPercent != null" class="baker-pct" title="Baker's percentage">{{
        formatBaker(bakerPercent)
      }}</span>
      <span v-if="parts.aside" class="ingredient-aside">{{ parts.aside }}</span>
    </span>
  </li>
</template>

<style scoped>
.ingredient-row {
  display: grid;
  grid-template-columns: minmax(4.5rem, 6.5rem) 1fr;
  gap: 12px;
  align-items: baseline;
  padding: 8px 0;
  border-bottom: 1px dashed #e2e0d4;
}
.ingredient-row.selectable {
  cursor: pointer;
  border-radius: 6px;
  margin: 0 -6px;
  padding-left: 6px;
  padding-right: 6px;
}
.ingredient-row.selectable:hover {
  background: rgb(40 100 59 / 8%);
}
.ingredient-row.highlighted {
  background: rgb(40 100 59 / 12%);
  box-shadow: inset 3px 0 0 #28643b;
}
.ingredient-row:last-child {
  border-bottom: none;
}
.ingredient-qty {
  font-variant-numeric: tabular-nums;
  font-weight: 600;
  color: #3d4a42;
}
.ingredient-body {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px 10px;
}
.ingredient-name {
  color: #1f2923;
}
.baker-pct {
  font-size: 0.85em;
  font-variant-numeric: tabular-nums;
  color: #8a6a28;
  background: #f7efd8;
  padding: 0 0.35em;
  border-radius: 4px;
}
.ingredient-aside {
  width: 100%;
  font-size: 0.9em;
  color: #6d7972;
  font-style: italic;
}
</style>
