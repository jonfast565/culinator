<script setup lang="ts">
import type { SectionMise } from "../../recipe-editor/narrative";
import IngredientListRow from "./IngredientListRow.vue";

// A divided ingredient shows this section's own amount, not the recipe total —
// that split is done by the shared narrative generator.
defineProps<{ mise: SectionMise }>();
</script>

<template>
  <aside v-if="mise.ingredients.length || mise.equipment.length" class="mise">
    <div v-if="mise.ingredients.length" class="mise-group">
      <h4 class="mise-label">You'll need</h4>
      <ul class="mise-list">
        <IngredientListRow
          v-for="(item, index) in mise.ingredients"
          :key="`${item.description}-${item.amount}-${index}`"
          :parts="item"
        />
      </ul>
    </div>
    <div v-if="mise.equipment.length" class="mise-group">
      <h4 class="mise-label">Equipment</h4>
      <ul class="equipment-list">
        <li v-for="item in mise.equipment" :key="item">{{ item }}</li>
      </ul>
    </div>
  </aside>
</template>

<style scoped>
.mise {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 0;
  padding: 12px 16px;
  border-left: 2px solid #cfd8cb;
  background: #f4f2e9;
  border-radius: 3px;
}
.mise-group {
  min-width: 0;
}
.mise-label {
  margin: 0 0 6px;
  font-size: calc(10px * var(--reading-scale, 1));
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: #6d7972;
}
.mise-list {
  list-style: none;
  margin: 0;
  padding: 0;
}
.mise-list :deep(.ingredient-row) {
  padding: 5px 0;
  border-bottom-color: #e0ddd0;
  grid-template-columns: minmax(4.5rem, 6.5rem) 1fr;
  gap: 12px;
}
.mise-list :deep(.ingredient-qty) {
  font-size: 0.88em;
}
.mise-list :deep(.ingredient-name) {
  font-size: calc(14px * var(--reading-scale, 1));
}
.equipment-list {
  list-style: none;
  margin: 0;
  padding: 0;
  font-size: calc(14px * var(--reading-scale, 1));
  line-height: 1.5;
}
.equipment-list li {
  padding: 2px 0;
}
.equipment-list li::before {
  content: "·";
  margin-right: 8px;
  color: #28643b;
  font-weight: 700;
}
</style>
