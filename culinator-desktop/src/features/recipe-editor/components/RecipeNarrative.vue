<script setup lang="ts">
import { computed, toRef } from "vue";
import { Clock } from "lucide-vue-next";
import type { UiRecipeModel } from "../model";
import { useRecipeNarrative, formatIngredientDescription } from "../narrative";

const props = defineProps<{ model: UiRecipeModel }>();

const { ingredientGroups, operations, rows, summary, describe, stepTime, stepMeta } =
  useRecipeNarrative(toRef(props, "model"));

const hasSteps = computed(() => operations.value.length > 0);
</script>

<template>
  <section class="panel narrative">
    <header class="narrative-head">
      <h3>{{ model.title || "Untitled recipe" }}</h3>
      <small>{{ summary }}</small>
    </header>

    <div class="narrative-section">
      <h4>Ingredients</h4>
      <div v-if="ingredientGroups.length" class="ingredient-groups">
        <div v-for="group in ingredientGroups" :key="group.label ?? 'base'">
          <h5 v-if="group.label" class="variant-heading">{{ group.label }} finish</h5>
          <ul class="ingredient-list">
            <li v-for="ingredient in group.items" :key="ingredient.symbol">
              {{ formatIngredientDescription(ingredient) }}
            </li>
          </ul>
        </div>
      </div>
      <p v-else class="empty">No ingredients yet.</p>
    </div>

    <div class="narrative-section">
      <h4>Method</h4>
      <div v-if="hasSteps" class="method">
        <template v-for="row in rows" :key="row.key">
          <h5 v-if="row.kind === 'heading'" class="method-heading">{{ row.label }}</h5>
          <div v-else class="method-step">
            <span class="step-number">{{ row.number }}</span>
            <div class="step-body">
              <p>{{ describe(row.operation!) }}</p>
              <small v-if="stepMeta(row.operation!)">{{ stepMeta(row.operation!) }}</small>
            </div>
            <span v-if="stepTime(row.operation!)" class="step-time">
              <Clock :size="12" />{{ stepTime(row.operation!) }}
            </span>
          </div>
        </template>
      </div>
      <p v-else class="empty">No steps yet.</p>
    </div>

    <footer v-if="model.attribution || model.source" class="narrative-credit">
      <p v-if="model.attribution">{{ model.attribution }}</p>
      <p v-else>Recipe from {{ model.source }}.</p>
      <a v-if="model.sourceUrl" :href="model.sourceUrl" target="_blank" rel="noopener noreferrer">
        {{ model.sourceUrl }}
      </a>
    </footer>
  </section>
</template>

<style scoped>
.variant-heading {
  margin: 12px 0 6px;
  font-size: 11px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--muted, #6d7972);
}
</style>
