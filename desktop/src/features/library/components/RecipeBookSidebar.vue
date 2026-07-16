<script setup lang="ts">
import {
  BookPlus,
  FilePlus2,
  FileUp,
  Library,
  Pencil,
  ScanLine,
  Trash2,
  Utensils,
} from "lucide-vue-next";
import type { RecipeBookSummary, RecipeSummary } from "../../../domain/types";

defineProps<{
  books: RecipeBookSummary[];
  recipes: RecipeSummary[];
  selectedBookId: string | null;
  selectedRecipeId?: string;
}>();
const emit = defineEmits<{
  selectBook: [id: string | null];
  selectRecipe: [id: string];
  createRecipe: [];
  importRecipe: [];
  importFile: [];
  createBook: [];
  renameBook: [book: RecipeBookSummary];
  deleteBook: [book: RecipeBookSummary];
}>();
function recipesFor(bookId: string | null, recipes: RecipeSummary[]): RecipeSummary[] {
  return recipes.filter((recipe) => (recipe.bookId ?? null) === bookId);
}
</script>

<template>
  <aside class="sidebar">
    <header class="brand">
      <span class="brand-mark"><Utensils :size="19" /></span
      ><span><strong>Culinograph</strong><small>Food production studio</small></span>
    </header>
    <div class="sidebar-actions">
      <button class="sidebar-action" @click="emit('createRecipe')">
        <FilePlus2 :size="16" />New recipe
      </button>
      <button class="sidebar-action" @click="emit('importRecipe')">
        <ScanLine :size="16" />Scan
      </button>
      <button class="sidebar-action" @click="emit('importFile')">
        <FileUp :size="16" />Import file
      </button>
    </div>
    <div class="sidebar-heading">
      <span>Recipe books</span
      ><button class="icon-button" title="New book" @click="emit('createBook')">
        <BookPlus :size="16" />
      </button>
    </div>
    <nav class="library-list">
      <section v-for="book in books" :key="book.id" class="book-group">
        <div class="book-row" :class="{ selected: selectedBookId === book.id }">
          <button
            class="book-select"
            @click="emit('selectBook', selectedBookId === book.id ? null : book.id)"
          >
            <Library :size="16" /><span
              ><strong>{{ book.title }}</strong
              ><small>{{ recipesFor(book.id, recipes).length }} recipes</small></span
            >
          </button>
          <button title="Rename" @click="emit('renameBook', book)"><Pencil :size="14" /></button
          ><button title="Delete" @click="emit('deleteBook', book)"><Trash2 :size="14" /></button>
        </div>
        <button
          v-for="recipe in recipesFor(book.id, recipes)"
          :key="recipe.id"
          class="recipe-row"
          :class="{ selected: selectedRecipeId === recipe.id }"
          @click="emit('selectRecipe', recipe.id)"
        >
          {{ recipe.title }}
        </button>
      </section>
      <section class="book-group">
        <div class="book-row">
          <button class="book-select" @click="emit('selectBook', null)">
            <Library :size="16" /><span
              ><strong>Unfiled</strong
              ><small>{{ recipesFor(null, recipes).length }} recipes</small></span
            >
          </button>
        </div>
        <button
          v-for="recipe in recipesFor(null, recipes)"
          :key="recipe.id"
          class="recipe-row"
          :class="{ selected: selectedRecipeId === recipe.id }"
          @click="emit('selectRecipe', recipe.id)"
        >
          {{ recipe.title }}
        </button>
      </section>
    </nav>
  </aside>
</template>
