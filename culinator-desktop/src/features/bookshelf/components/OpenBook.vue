<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ChevronLeft, Search, Loader2, BookOpen, ListChecks, X } from "lucide-vue-next";
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
const searchOpen = ref(false);

const bookTitle = computed(() => props.book?.title ?? "Unfiled recipes");
const query = ref("");
const loading = ref(false);
const loaded = ref<LoadedRecipe[]>([]);

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
const flipKey = computed(() => leaves.value.map((leaf) => leaf.key).join("|"));

function handleSearchSelect(recipeId: string): void {
  searchOpen.value = false;
  emit("open-recipe", recipeId);
}
</script>

<template>
  <div class="open-book">
    <header class="book-bar">
      <button type="button" class="ghost" @click="emit('back')">
        <ChevronLeft :size="16" /> Shelf
      </button>
      <h1 class="book-name">{{ bookTitle }}</h1>
      <div class="book-tools">
        <div class="mode-toggle" role="tablist">
          <button
            type="button"
            :class="{ active: mode === 'flip' }"
            title="Flip through"
            @click="mode = 'flip'"
          >
            <BookOpen :size="15" />
          </button>
          <button
            type="button"
            :class="{ active: mode === 'manage' }"
            title="Manage recipes"
            @click="mode = 'manage'"
          >
            <ListChecks :size="15" />
          </button>
        </div>
        <button
          v-if="useServiceSearch && book"
          type="button"
          class="tool-btn"
          :class="{ active: searchOpen }"
          title="Search this book"
          @click="searchOpen = !searchOpen"
        >
          <Search :size="15" />
        </button>
        <label v-else class="book-search">
          <Search :size="15" />
          <input
            v-model="query"
            type="search"
            placeholder="Search…"
            aria-label="Search recipes in this book"
          />
        </label>
        <BookExportPanel v-if="book" :book-id="book.id" :book-title="book.title" />
      </div>
    </header>

    <div v-if="searchOpen && useServiceSearch && book" class="search-popover">
      <div class="search-popover-head">
        <strong>Search this book</strong>
        <button type="button" class="icon-btn" aria-label="Close search" @click="searchOpen = false">
          <X :size="16" />
        </button>
      </div>
      <RecipeSearchPanel
        :book-id="book.id"
        placeholder="Search this book…"
        @select="handleSearchSelect"
      />
    </div>

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
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: radial-gradient(120% 80% at 50% -10%, #efece2 0%, #e7e3d6 55%, #ddd7c6 100%);
}
.book-bar {
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 12px;
  padding: 8px 14px;
  border-bottom: 1px solid rgba(60, 50, 30, 0.14);
  background: rgba(251, 249, 243, 0.72);
  backdrop-filter: blur(6px);
}
.book-bar .ghost {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 32px;
  padding: 0 10px;
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid #cbd3cd;
  border-radius: 8px;
  color: #23302a;
  font-size: 13px;
  cursor: pointer;
}
.book-bar .ghost:hover {
  background: #fff;
}
.book-name {
  margin: 0;
  font-family: "Iowan Old Style", "Palatino Linotype", Palatino, Georgia, serif;
  font-size: 18px;
  font-weight: 600;
  text-align: center;
  color: #23302a;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.book-tools {
  display: flex;
  align-items: center;
  gap: 8px;
}
.mode-toggle {
  display: inline-flex;
  background: rgba(255, 255, 255, 0.6);
  border: 1px solid #cbd3cd;
  border-radius: 8px;
  overflow: hidden;
}
.mode-toggle button {
  width: 34px;
  height: 32px;
  display: grid;
  place-items: center;
  background: transparent;
  border: 0;
  color: #55635b;
  cursor: pointer;
}
.mode-toggle button.active {
  background: #28643b;
  color: #fff;
}
.tool-btn {
  display: grid;
  place-items: center;
  width: 34px;
  height: 32px;
  border-radius: 8px;
  border: 1px solid #cbd3cd;
  background: rgba(255, 255, 255, 0.75);
  color: #55635b;
  cursor: pointer;
}
.tool-btn:hover,
.tool-btn.active {
  background: #fff;
  color: #28643b;
}
.book-search {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: 32px;
  padding: 0 12px;
  background: rgba(255, 255, 255, 0.75);
  border: 1px solid #cbd3cd;
  border-radius: 999px;
  color: #6d7972;
  max-width: 200px;
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
.search-popover {
  position: absolute;
  top: 52px;
  right: 14px;
  z-index: 20;
  width: min(92vw, 420px);
  padding: 14px;
  border: 1px solid #cbd3cd;
  border-radius: 12px;
  background: #fbf9f3;
  box-shadow: 0 16px 40px -18px rgba(20, 30, 25, 0.35);
}
.search-popover-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
  font-size: 14px;
  color: #23302a;
}
.icon-btn {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: #55635b;
  cursor: pointer;
}
.icon-btn:hover {
  background: rgba(40, 100, 59, 0.08);
  color: #28643b;
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
