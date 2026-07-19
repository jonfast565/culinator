<script setup lang="ts">
import type { IngredientGroup } from "../../recipe-editor/narrative";
import IngredientListRow from "./IngredientListRow.vue";

// Items arrive already rendered by the shared narrative generator, so there is
// nothing to format here.
defineProps<{ groups: IngredientGroup[] }>();
</script>

<template>
  <div class="ingredient-groups">
    <div v-for="group in groups" :key="group.label ?? 'base'" class="ingredient-group">
      <h3 v-if="group.label" class="variant-heading">{{ group.label }} finish</h3>
      <ul class="ingredient-list">
        <IngredientListRow
          v-for="(item, index) in group.items"
          :key="`${item.description}-${index}`"
          :parts="item"
        />
      </ul>
    </div>
  </div>
</template>

<style scoped>
.ingredient-groups {
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.variant-heading {
  margin: 0 0 8px;
  font-family: "Iowan Old Style", "Palatino Linotype", Palatino, Georgia, serif;
  font-size: 13px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: #6d7972;
}
.ingredient-list {
  list-style: none;
  margin: 0;
  padding: 0;
}
</style>
