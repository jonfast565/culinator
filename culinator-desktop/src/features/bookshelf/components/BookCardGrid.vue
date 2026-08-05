<script setup lang="ts">
import { computed } from "vue";
import { ArrowRight, Clock } from "lucide-vue-next";
import { sectionOf, summarize, type LoadedRecipe } from "../bookContents";
import { previewIngredientParts, previewSteps } from "../../recipe-editor/narrative";
import RecipeImage from "../../reading/components/RecipeImage.vue";
import IngredientListRow from "../../reading/components/IngredientListRow.vue";

const props = defineProps<{
  recipes: LoadedRecipe[];
}>();
const emit = defineEmits<{ (event: "open-recipe", recipeId: string): void }>();

interface CardGroup {
  section: string;
  cards: {
    id: string;
    title: string;
    summary: string;
    cover?: string;
    ingredients: ReturnType<typeof previewIngredientParts>;
    steps: string[];
    stepCount: number;
  }[];
}

const groups = computed<CardGroup[]>(() => {
  const order: string[] = [];
  const bySection = new Map<string, CardGroup["cards"]>();
  for (const recipe of props.recipes) {
    const section = sectionOf(recipe.model);
    if (!bySection.has(section)) {
      bySection.set(section, []);
      order.push(section);
    }
    const operations = recipe.model.operations ?? [];
    bySection.get(section)!.push({
      id: recipe.id,
      title: recipe.model.title || "Untitled recipe",
      summary: summarize(recipe.model),
      cover: recipe.model.coverImage,
      ingredients: previewIngredientParts(recipe.source, 5),
      steps: previewSteps(recipe.source, 4),
      stepCount: operations.length,
    });
  }
  return order.map((section) => ({ section, cards: bySection.get(section)! }));
});

const showSectionHeaders = computed(() => groups.value.length > 1);
</script>

<template>
  <div class="card-grid-wrap">
    <div class="card-grid-scroll">
      <section v-for="group in groups" :key="group.section" class="card-section">
        <h2 v-if="showSectionHeaders" class="section-heading">{{ group.section }}</h2>
        <div class="card-grid">
          <article
            v-for="card in group.cards"
            :key="card.id"
            class="recipe-card"
            role="button"
            tabindex="0"
            @click="emit('open-recipe', card.id)"
            @keydown.enter="emit('open-recipe', card.id)"
          >
            <figure v-if="card.cover" class="card-cover">
              <RecipeImage :image-ref="card.cover" :recipe-id="card.id" :alt="card.title" />
            </figure>
            <div class="card-body">
              <p v-if="showSectionHeaders" class="card-eyebrow">{{ group.section }}</p>
              <h3 class="card-title">{{ card.title }}</h3>
              <p class="card-summary"><Clock :size="12" /> {{ card.summary }}</p>

              <div class="card-body-grid">
                <section v-if="card.ingredients.length" class="card-block">
                  <h4 class="block-label">Ingredients</h4>
                  <ul class="card-ings">
                    <IngredientListRow
                      v-for="(ingredient, index) in card.ingredients"
                      :key="index"
                      :parts="ingredient"
                    />
                  </ul>
                </section>

                <section v-if="card.steps.length" class="card-block">
                  <h4 class="block-label">Method</h4>
                  <ol class="card-steps">
                    <li v-for="(step, index) in card.steps" :key="index">{{ step }}</li>
                  </ol>
                  <p v-if="card.stepCount > card.steps.length" class="more-steps">
                    + {{ card.stepCount - card.steps.length }} more step{{
                      card.stepCount - card.steps.length === 1 ? "" : "s"
                    }}
                  </p>
                </section>
              </div>

              <span class="open-link">Open full recipe <ArrowRight :size="14" /></span>
            </div>
          </article>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.card-grid-wrap {
  --serif: "Iowan Old Style", "Palatino Linotype", Palatino, "Book Antiqua", Georgia, serif;
  --paper: #fbf9f3;
  --ink: #23302a;
  --muted: #6d7972;
  --herb: #28643b;
  --rule: #ddd9cc;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.card-grid-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 20px clamp(16px, 3vw, 32px) 40px;
}
.card-section + .card-section {
  margin-top: 28px;
}
.section-heading {
  margin: 0 0 14px;
  font-family: var(--serif);
  font-size: 20px;
  font-weight: 600;
  color: var(--herb);
}
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 340px), 1fr));
  gap: 18px;
  max-width: 1320px;
  margin: 0 auto;
}
.recipe-card {
  display: flex;
  flex-direction: column;
  background: var(--paper);
  border: 1px solid #e3ddcd;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 10px 28px -20px rgba(40, 40, 30, 0.45);
  cursor: pointer;
  transition:
    transform 0.16s ease,
    box-shadow 0.16s ease,
    border-color 0.16s ease;
}
.recipe-card:hover,
.recipe-card:focus-visible {
  transform: translateY(-3px);
  border-color: var(--herb);
  box-shadow: 0 16px 36px -18px rgba(40, 100, 59, 0.35);
  outline: none;
}
.card-cover {
  margin: 0;
  aspect-ratio: 16 / 9;
  max-height: 180px;
  overflow: hidden;
  background: #ece8dc;
}
.card-body {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 8px;
  padding: 16px 18px 18px;
  color: var(--ink);
}
.card-eyebrow {
  margin: 0;
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--herb);
  font-weight: 600;
}
.card-title {
  margin: 0;
  font-family: var(--serif);
  font-size: 20px;
  font-weight: 600;
  line-height: 1.12;
}
.card-summary {
  display: flex;
  align-items: center;
  gap: 5px;
  margin: 0;
  font-size: 11px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--muted);
}
.card-body-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.15fr);
  gap: 14px;
  margin-top: 4px;
}
.card-block {
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.block-label {
  margin: 0 0 6px;
  font-size: 10px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--herb);
  font-weight: 700;
}
.card-ings {
  list-style: none;
  margin: 0;
  padding: 0;
}
.card-ings :deep(.ingredient-row) {
  padding: 3px 0;
  border-bottom-color: var(--rule);
  grid-template-columns: minmax(3rem, 4.5rem) 1fr;
  gap: 6px;
}
.card-ings :deep(.ingredient-qty) {
  font-size: 11px;
}
.card-ings :deep(.ingredient-name),
.card-ings :deep(.ingredient-aside) {
  font-size: 12px;
}
.card-steps {
  margin: 0;
  padding: 0 0 0 1.1em;
  color: #3a463f;
  font-size: 12px;
  line-height: 1.5;
}
.card-steps li + li {
  margin-top: 6px;
}
.more-steps {
  margin: 6px 0 0;
  font-size: 11px;
  color: var(--muted);
  font-style: italic;
}
.open-link {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  margin-top: auto;
  padding-top: 8px;
  font-size: 12px;
  font-weight: 600;
  color: var(--herb);
}
.recipe-card:hover .open-link {
  text-decoration: underline;
}

@media (max-width: 560px) {
  .card-body-grid {
    grid-template-columns: 1fr;
  }
}
</style>
