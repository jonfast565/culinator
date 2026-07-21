<script setup lang="ts">
import { computed } from "vue";
import type { UiResource } from "../../recipe-editor/model";
import { collectIngredientSubstitutions } from "../substitutions";

const props = defineProps<{ resources: UiResource[] }>();

const substitutions = computed(() => collectIngredientSubstitutions(props.resources));
</script>

<template>
  <details v-if="substitutions.length" class="substitution-assistant">
    <summary>
      <span>Out of an ingredient?</span>
      <small>
        {{ substitutions.length }}
        {{ substitutions.length === 1 ? "ingredient has" : "ingredients have" }} alternatives
      </small>
    </summary>
    <ul role="list">
      <li v-for="item in substitutions" :key="item.symbol">
        <strong>{{ item.ingredient }}</strong>
        <span aria-hidden="true">→</span>
        <span>{{ item.alternatives.join(" or ") }}</span>
      </li>
    </ul>
  </details>
</template>

<style scoped>
.substitution-assistant {
  margin-top: 14px;
  border: 1px solid #cfd8cb;
  border-radius: 5px;
  background: #f4f2e9;
}
summary {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  padding: 9px 12px;
  color: #28643b;
  cursor: pointer;
}
summary::marker {
  color: #28643b;
}
summary span {
  font-size: calc(13px * var(--reading-scale, 1));
  font-weight: 650;
}
summary small {
  color: #6d7972;
  font-size: calc(11px * var(--reading-scale, 1));
  text-align: right;
}
ul {
  display: grid;
  gap: 8px;
  margin: 0;
  padding: 10px 12px 12px;
  border-top: 1px solid #dfe3d8;
  list-style: none;
}
li {
  display: grid;
  grid-template-columns: minmax(7rem, auto) auto 1fr;
  gap: 8px;
  align-items: baseline;
  color: #4d5b53;
  font-size: calc(13px * var(--reading-scale, 1));
}
li strong {
  color: #23302a;
}

@media (max-width: 520px) {
  summary,
  li {
    align-items: flex-start;
  }
  li {
    grid-template-columns: auto auto 1fr;
  }
}
</style>
