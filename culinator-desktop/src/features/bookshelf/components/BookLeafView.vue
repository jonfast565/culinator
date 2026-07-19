<script setup lang="ts">
import { Clock, ArrowRight } from "lucide-vue-next";
import type { BookLeaf } from "../bookContents";
import RecipeImage from "../../reading/components/RecipeImage.vue";
import IngredientListRow from "../../reading/components/IngredientListRow.vue";

defineProps<{ leaf: BookLeaf }>();
const emit = defineEmits<{
  (event: "open-recipe", recipeId: string): void;
  (event: "flip-to", page: number): void;
}>();
</script>

<template>
  <!-- Cover -->
  <div v-if="leaf.kind === 'cover'" class="leaf cover">
    <p class="cover-kicker">Recipe book</p>
    <h1 class="cover-title">{{ leaf.title }}</h1>
    <p class="cover-sub">{{ leaf.subtitle }}</p>
  </div>

  <!-- Table of contents (front matter) -->
  <div v-else-if="leaf.kind === 'toc'" class="leaf toc">
    <h2 class="leaf-heading">Contents</h2>
    <ul v-if="leaf.entries.length" class="toc-list">
      <li v-for="entry in leaf.entries" :key="entry.recipeId">
        <button
          type="button"
          class="toc-entry"
          :data-flip-to="entry.page"
          @click="emit('flip-to', entry.page)"
        >
          <span class="toc-title">{{ entry.title }}</span>
          <span class="toc-dots" aria-hidden="true"></span>
          <span class="toc-page">{{ entry.page + 1 }}</span>
        </button>
      </li>
    </ul>
    <p v-else class="empty">This book has no recipes yet.</p>
  </div>

  <!-- Section divider -->
  <div v-else-if="leaf.kind === 'section'" class="leaf section">
    <span class="section-rule" aria-hidden="true"></span>
    <h2 class="section-title">{{ leaf.title }}</h2>
    <span class="section-rule" aria-hidden="true"></span>
  </div>

  <!-- Recipe card -->
  <div v-else class="leaf recipe">
    <figure v-if="leaf.cover" class="card-cover">
      <RecipeImage :image-ref="leaf.cover" :recipe-id="leaf.recipeId" :alt="leaf.title" />
    </figure>
    <p class="recipe-eyebrow">{{ leaf.eyebrow }}</p>
    <h2 class="recipe-title">{{ leaf.title }}</h2>
    <p class="recipe-summary"><Clock :size="13" /> {{ leaf.summary }}</p>

    <div class="recipe-body">
      <section v-if="leaf.ingredients.length" class="recipe-block">
        <h3 class="block-label">Ingredients</h3>
        <ul class="recipe-ings">
          <IngredientListRow
            v-for="(ingredient, index) in leaf.ingredients"
            :key="index"
            :parts="ingredient"
          />
        </ul>
      </section>

      <section v-if="leaf.steps.length" class="recipe-block">
        <h3 class="block-label">Method</h3>
        <ol class="recipe-steps">
          <li v-for="(step, index) in leaf.steps" :key="index">{{ step }}</li>
        </ol>
        <p v-if="leaf.stepCount > leaf.steps.length" class="more-steps">
          + {{ leaf.stepCount - leaf.steps.length }} more step{{
            leaf.stepCount - leaf.steps.length === 1 ? "" : "s"
          }}
        </p>
      </section>
    </div>

    <button type="button" class="open-recipe" @click="emit('open-recipe', leaf.recipeId)">
      Open full recipe <ArrowRight :size="15" />
    </button>
  </div>
</template>

<style scoped>
.leaf {
  --serif: "Iowan Old Style", "Palatino Linotype", Palatino, "Book Antiqua", Georgia, serif;
  --paper: #fbf9f3;
  --ink: #23302a;
  --muted: #6d7972;
  --herb: #28643b;
  --rule: #ddd9cc;
  height: 100%;
  padding: clamp(20px, 3.5vw, 36px);
  overflow: hidden;
  color: var(--ink);
  display: flex;
  flex-direction: column;
  box-shadow: inset 0 0 60px -40px rgba(60, 50, 30, 0.6);
}

/* Cover */
.cover {
  justify-content: center;
  align-items: flex-start;
  background:
    linear-gradient(180deg, rgba(40, 100, 59, 0.06), rgba(40, 100, 59, 0.02)), var(--paper);
  border-left: 6px solid var(--herb);
}
.cover-kicker {
  margin: 0 0 14px;
  font-size: 11px;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--herb);
  font-weight: 600;
}
.cover-title {
  margin: 0;
  font-family: var(--serif);
  font-weight: 600;
  font-size: clamp(28px, 4.4vw, 44px);
  line-height: 1.05;
}
.cover-sub {
  margin: 16px 0 0;
  font-size: 13px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--muted);
}

/* Table of contents */
.leaf-heading {
  margin: 0 0 20px;
  font-family: var(--serif);
  font-size: 24px;
  font-weight: 600;
  color: var(--ink);
}
.toc-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow: auto;
}
.toc-entry {
  width: 100%;
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 9px 0;
  background: transparent;
  border: 0;
  border-bottom: 1px solid var(--rule);
  text-align: left;
  color: var(--ink);
  cursor: pointer;
}
.toc-entry:hover {
  color: var(--herb);
}
.toc-title {
  font-family: var(--serif);
  font-size: 16px;
}
.toc-dots {
  flex: 1;
  border-bottom: 1px dotted var(--rule);
  transform: translateY(-3px);
}
.toc-page {
  font-variant-numeric: tabular-nums;
  font-size: 13px;
  color: var(--muted);
}

/* Section divider */
.section {
  align-items: center;
  justify-content: center;
  gap: 20px;
}
.section-rule {
  width: 44px;
  height: 2px;
  background: var(--herb);
}
.section-title {
  margin: 0;
  font-family: var(--serif);
  font-size: clamp(24px, 4vw, 34px);
  font-weight: 600;
  text-align: center;
}

/* Recipe card */
.card-cover {
  margin: 0 0 14px;
  aspect-ratio: 16 / 9;
  max-height: 38%;
  overflow: hidden;
  border-radius: 4px;
  box-shadow: 0 8px 20px -14px rgba(40, 40, 30, 0.5);
}
.recipe-eyebrow {
  margin: 0 0 8px;
  font-size: 11px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--herb);
  font-weight: 600;
}
.recipe-title {
  margin: 0;
  font-family: var(--serif);
  font-weight: 600;
  font-size: clamp(22px, 3.2vw, 30px);
  line-height: 1.08;
}
.recipe-summary {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 10px 0 14px;
  font-size: 12px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--muted);
}
.recipe-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr);
  gap: 16px;
  overflow: hidden;
}
.recipe-block {
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.block-label {
  margin: 0 0 8px;
  font-size: 11px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--herb);
  font-weight: 700;
}
.recipe-ings {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow: auto;
}
.recipe-ings :deep(.ingredient-row) {
  padding: 4px 0;
  border-bottom-color: var(--rule);
  grid-template-columns: minmax(3.5rem, 5rem) 1fr;
  gap: 8px;
}
.recipe-ings :deep(.ingredient-qty) {
  font-size: 12px;
}
.recipe-ings :deep(.ingredient-name),
.recipe-ings :deep(.ingredient-aside) {
  font-size: 13px;
}
.recipe-steps {
  margin: 0;
  padding: 0 0 0 1.1em;
  color: #3a463f;
  font-size: 13px;
  line-height: 1.55;
  overflow: auto;
}
.recipe-steps li + li {
  margin-top: 8px;
}
.more-steps {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--muted);
  font-style: italic;
}
.open-recipe {
  margin-top: 14px;
  align-self: flex-start;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 9px 15px;
  border-radius: 999px;
  border: 1px solid var(--herb);
  background: transparent;
  color: var(--herb);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.open-recipe:hover {
  background: var(--herb);
  color: #fff;
}
.empty {
  color: var(--muted);
  font-style: italic;
}

@media (max-width: 520px) {
  .recipe-body {
    grid-template-columns: 1fr;
  }
}
</style>
