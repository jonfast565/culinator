<script setup lang="ts">
import type { IngredientDisplayParts } from "../../recipe-editor/narrative";

defineProps<{
  parts: IngredientDisplayParts;
  selectable?: boolean;
  highlighted?: boolean;
}>();
const emit = defineEmits<{ select: [symbol: string] }>();
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
  border-bottom: 0;
}
.ingredient-qty {
  font-variant-numeric: tabular-nums;
  font-weight: 600;
  font-size: 0.92em;
  color: #28643b;
  text-align: right;
  line-height: 1.35;
}
.ingredient-qty:empty::before {
  content: "\00a0";
}
.ingredient-body {
  display: block;
  min-width: 0;
  line-height: 1.45;
}
.ingredient-name {
  color: #23302a;
}
.ingredient-aside {
  color: #6d7972;
  font-size: 0.92em;
}
.ingredient-aside::before {
  content: ", ";
}
</style>
