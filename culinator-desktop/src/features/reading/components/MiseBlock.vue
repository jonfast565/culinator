<script setup lang="ts">
import { computed } from "vue";
import type { SectionMise } from "../../recipe-editor/narrative";
import { indexCardBlockKey } from "../measureIndexCard";
import {
  MISE_EQUIPMENT_LABEL,
  MISE_INGREDIENTS_LABEL,
  indexCardMiseLabel,
} from "../paginateIndexCards";
import IngredientListRow from "./IngredientListRow.vue";

// A divided ingredient shows this section's own amount, not the recipe total —
// that split is done by the shared narrative generator.
const props = defineProps<{
  mise: SectionMise;
  selectable?: boolean;
  highlightedSymbol?: string | null;
  /** A card holds only the tail of this list; the head was on an earlier card. */
  ingredientsContinued?: boolean;
  equipmentContinued?: boolean;
  /** Offscreen measurement card only — tags each part for `measureIndexCard`. */
  measured?: boolean;
}>();
const emit = defineEmits<{ select: [symbol: string] }>();

const ingredientsLabel = computed(() =>
  indexCardMiseLabel(MISE_INGREDIENTS_LABEL, props.ingredientsContinued),
);
const equipmentLabel = computed(() =>
  indexCardMiseLabel(MISE_EQUIPMENT_LABEL, props.equipmentContinued),
);

function measureKey(key: string): string | undefined {
  return props.measured ? key : undefined;
}
</script>

<template>
  <aside
    v-if="mise.ingredients.length || mise.equipment.length"
    class="mise"
    :data-measure-role="measured ? 'mise' : undefined"
  >
    <div v-if="mise.ingredients.length" class="mise-group">
      <h4
        class="mise-label"
        :data-measure-key="measureKey(indexCardBlockKey.miseLabel(ingredientsLabel))"
      >
        {{ ingredientsLabel }}
      </h4>
      <ul class="mise-list">
        <IngredientListRow
          v-for="(item, index) in mise.ingredients"
          :key="`${item.description}-${item.amount}-${index}`"
          :parts="item"
          :selectable="selectable"
          :highlighted="highlightedSymbol === item.symbol"
          :data-measure-key="measureKey(indexCardBlockKey.miseIngredientRow(item))"
          @select="emit('select', $event)"
        />
      </ul>
    </div>
    <div v-if="mise.equipment.length" class="mise-group">
      <h4
        class="mise-label"
        :data-measure-key="measureKey(indexCardBlockKey.miseLabel(equipmentLabel))"
      >
        {{ equipmentLabel }}
      </h4>
      <ul class="equipment-list">
        <li
          v-for="item in mise.equipment"
          :key="item"
          :data-measure-key="measureKey(indexCardBlockKey.miseEquipmentRow(item))"
        >
          {{ item }}
        </li>
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
  font-family: var(--reading-sans);
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
