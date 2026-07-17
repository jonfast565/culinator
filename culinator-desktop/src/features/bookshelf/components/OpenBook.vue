<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ChevronLeft, Search, Loader2, BookOpen, ListChecks } from "lucide-vue-next";
import { getRecipe } from "../../../services/api";
import { parseUiModel } from "../../recipe-editor/model";
import { buildLeaves, sectionOf, type LoadedRecipe } from "../bookContents";
import type { RecipeBookSummary, RecipeSummary } from "../../../domain/types";
import BookFlip from "./BookFlip.vue";
import BookManage from "./BookManage.vue";
import BookExportPanel from "../../export/components/BookExportPanel.vue";
import RecipeSearchPanel from "../../search/components/RecipeSearchPanel.vue";

const props = defineProps<{
  book: RecipeBookSummary | null;
  recipes: RecipeSummary[];
  books: RecipeBookSummary[];
}>();
const emit = defineEmits<{
  (event: "back"): void;
  (event: "open-recipe", recipeId: string): void;
  (event: "bulk-move", ids: string[], bookId: string | null): void;
  (event: "bulk-delete", ids: string[]): void;
}>();

const mode = ref<"flip" | "manage">("flip");
const useServiceSearch = ref(true);

const bookTitle = computed(() => props.book?.title ?? "Unfiled recipes");
const query = ref("");
const loading = ref(false);
const loaded = ref<LoadedRecipe[]>([]);

// Load each recipe's document so cards and the table of contents can show the
// parsed section, title, and summary. Re-runs whenever the book's recipe set
// changes (add/remove/reorder).
watch(
  () => props.recipes.map((recipe) => recipe.id).join("|"),
  async () => {
    loading.value = true;
    try {
      const documents = await Promise.all(props.recipes.map((recipe) => getRecipe(recipe.id)));
      loaded.value = documents
        .filter((document): document is NonNullable<typeof document> => Boolean(document))
        .map((document) => ({ id: document.id, model: parseUiModel(document.sourceText) }));
    } finally {
      loading.value = false;
    }
  },
  { immediate: true },
);

const filtered = computed<LoadedRecipe[]>(() => {
  const term = query.value.trim().toLowerCase();
  if (!term) return loaded.value;
  return loaded.value.filter((recipe) => {
    const title = (recipe.model.title || "").toLowerCase();
    return title.includes(term) || sectionOf(recipe.model).toLowerCase().includes(term);
  });
});

const leaves = computed(() => buildLeaves(bookTitle.value, filtered.value));

// Remount the flip engine whenever the leaf set changes (StPageFlip owns its
// DOM, so a fresh instance is safer than mutating pages in place).
const flipKey = computed(() => leaves.value.map((leaf) => leaf.key).join("|"));
</script>

<template>
  <div class="open-book">
    <header class="book-bar">
      <button class="ghost" @click="emit('back')"><ChevronLeft :size="16" /> Shelf</button>
      <h1 class="book-name">{{ bookTitle }}</h1>
      <div class="book-tools">
        <div class="mode-toggle" role="tablist">
          <button :class="{ active: mode === 'flip' }" title="Flip through" @click="mode = 'flip'">
            <BookOpen :size="15" />
          </button>
          <button
            :class="{ active: mode === 'manage' }"
            title="Manage recipes"
            @click="mode = 'manage'"
          >
            <ListChecks :size="15" />
          </button>
        </div>
        <label class="book-search">
          <Search :size="15" />
          <input
            v-if="!useServiceSearch"
            v-model="query"
            type="search"
            placeholder="Search this book…"
            aria-label="Search recipes in this book"
          />
          <span v-else class="search-hint">Service search active below</span>
        </label>
        <BookExportPanel v-if="book" :book-id="book.id" :book-title="book.title" />
      </div>
    </header>

    <RecipeSearchPanel
      v-if="useServiceSearch && book"
      class="book-service-search"
      :book-id="book.id"
      placeholder="Search this book…"
      @select="emit('open-recipe', $event)"
    />

    <div v-if="loading" class="book-loading"><Loader2 :size="22" class="spin" /> Opening book…</div>
    <div v-else-if="!loaded.length" class="book-empty">
      <p>This book has no recipes yet.</p>
    </div>
    <BookManage
      v-else-if="mode === 'manage'"
      :recipes="filtered"
      :books="books"
      @open-recipe="emit('open-recipe', $event)"
      @bulk-move="(ids, bookId) => emit('bulk-move', ids, bookId)"
      @bulk-delete="(ids) => emit('bulk-delete', ids)"
    />
    <div v-else-if="!filtered.length" class="book-empty">
      <p>No recipes match “{{ query }}”.</p>
    </div>
    <BookFlip v-else :key="flipKey" :leaves="leaves" @open-recipe="emit('open-recipe', $event)" />
  </div>
</template>

<style scoped>
.open-book {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: radial-gradient(120% 80% at 50% -10%, #efece2 0%, #e7e3d6 55%, #ddd7c6 100%);
}
.book-bar {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 16px;
  padding: 14px 18px;
  border-bottom: 1px solid rgba(60, 50, 30, 0.14);
}
.book-bar .ghost {
  justify-self: start;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 34px;
  padding: 0 12px;
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid #cbd3cd;
  border-radius: 8px;
  color: #23302a;
  font-size: 13px;
}
.book-bar .ghost:hover {
  background: #fff;
}
.book-name {
  margin: 0;
  font-family: "Iowan Old Style", "Palatino Linotype", Palatino, Georgia, serif;
  font-size: 20px;
  font-weight: 600;
  text-align: center;
  color: #23302a;
}
.book-tools {
  justify-self: end;
  display: flex;
  align-items: center;
  gap: 10px;
}
.mode-toggle {
  display: inline-flex;
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid #cbd3cd;
  border-radius: 8px;
  overflow: hidden;
}
.mode-toggle button {
  width: 36px;
  height: 34px;
  display: grid;
  place-items: center;
  background: transparent;
  border: 0;
  color: #55635b;
}
.mode-toggle button.active {
  background: #28643b;
  color: #fff;
}
.book-search {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: 34px;
  padding: 0 12px;
  background: rgba(255, 255, 255, 0.75);
  border: 1px solid #cbd3cd;
  border-radius: 999px;
  color: #6d7972;
  max-width: 260px;
}
.book-search input {
  border: 0;
  background: transparent;
  outline: none;
  font-size: 13px;
  color: #23302a;
  width: 100%;
  padding: 0;
}
.search-hint {
  font-size: 12px;
  color: #6d7972;
}
.book-service-search {
  padding: 0 18px 12px;
}
.book-loading,
.book-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: #6d7972;
}
.spin {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
@media (prefers-reduced-motion: reduce) {
  .spin {
    animation: none;
  }
}
</style>
