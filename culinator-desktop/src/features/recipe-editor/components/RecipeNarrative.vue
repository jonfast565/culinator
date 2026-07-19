<script setup lang="ts">
import { computed, toRef } from "vue";
import { Clock } from "lucide-vue-next";
import type { UiRecipeModel } from "../model";
import { useRecipeNarrative } from "../narrative";
import IngredientGroupList from "../../reading/components/IngredientGroupList.vue";

const props = defineProps<{ model: UiRecipeModel; source: string }>();

const { ingredientGroups, summary, sections } = useRecipeNarrative(toRef(props, "source"));

const hasSteps = computed(() => sections.value.some((section) => section.steps.length > 0));
</script>

<template>
  <section class="panel narrative">
    <header class="narrative-head">
      <h3>{{ model.title || "Untitled recipe" }}</h3>
      <small>{{ summary }}</small>
    </header>

    <div class="narrative-section">
      <h4>Ingredients</h4>
      <IngredientGroupList v-if="ingredientGroups.length" :groups="ingredientGroups" />
      <p v-else class="empty">No ingredients yet.</p>
    </div>

    <div class="narrative-section">
      <h4>Method</h4>
      <div v-if="hasSteps" class="method">
        <template v-for="section in sections" :key="section.process">
          <h5 v-if="section.title" class="method-heading">{{ section.title }}</h5>
          <div v-for="step in section.steps" :key="step.symbol" class="method-step">
            <span class="step-number">{{ step.number }}</span>
            <div class="step-body">
              <p>{{ step.text }}</p>
              <small v-if="step.meta">{{ step.meta }}</small>
            </div>
            <span v-if="step.time" class="step-time"> <Clock :size="12" />{{ step.time }} </span>
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
