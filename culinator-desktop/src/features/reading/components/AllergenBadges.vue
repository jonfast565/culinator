<script setup lang="ts">
import { computed } from "vue";
import { formatAllergen } from "../allergens";

const props = defineProps<{ allergens: string[] }>();

const labels = computed(() => props.allergens.map(formatAllergen));
const ariaLabel = computed(() => `Contains allergens: ${labels.value.join(", ")}`);
</script>

<template>
  <div class="allergens" :aria-label="ariaLabel">
    <span class="allergens-label">Contains</span>
    <ul role="list">
      <li v-for="(allergen, index) in allergens" :key="allergen" :title="allergen">
        {{ labels[index] }}
      </li>
    </ul>
  </div>
</template>

<style scoped>
.allergens {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 14px;
}
.allergens-label {
  color: #7a2e1f;
  font-size: calc(11px * var(--reading-scale, 1));
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}
ul {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
}
li {
  padding: 3px 9px;
  border: 1px solid #e8b9ae;
  border-radius: 999px;
  background: #fde8e4;
  color: #7a2e1f;
  font-size: calc(11px * var(--reading-scale, 1));
  font-weight: 600;
}
</style>
