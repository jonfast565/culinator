<script setup lang="ts">
import type { IngredientGroup } from "../../recipe-editor/narrative";
import IngredientListRow from "./IngredientListRow.vue";

defineProps<{
  groups: IngredientGroup[];
  selectable?: boolean;
  highlightedSymbol?: string | null;
  bakerBySymbol?: Record<string, number>;
}>();
const emit = defineEmits<{ select: [symbol: string] }>();
</script>

<template>
  <div class="ingredient-groups">
    <div v-for="group in groups" :key="group.label ?? 'base'" class="ingredient-group">
      <h3 v-if="group.label" class="variant-heading">{{ group.label }}</h3>
      <ul class="ingredient-list">
        <IngredientListRow
          v-for="(item, index) in group.items"
          :key="`${item.description}-${index}`"
          :parts="item"
          :selectable="selectable"
          :highlighted="highlightedSymbol === item.symbol"
          :baker-percent="item.symbol ? bakerBySymbol?.[item.symbol] : null"
          @select="emit('select', $event)"
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
  font-family: var(--reading-sans);
  font-size: calc(13px * var(--reading-scale, 1));
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
