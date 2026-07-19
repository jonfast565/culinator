<script setup lang="ts">
import type { IngredientDisplayParts } from "../../recipe-editor/narrative";

defineProps<{ parts: IngredientDisplayParts }>();
</script>

<template>
  <li class="ingredient-row">
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
