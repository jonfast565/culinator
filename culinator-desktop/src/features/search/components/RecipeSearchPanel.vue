<script setup lang="ts">
/* global HTMLInputElement */
import { ref, watch } from "vue";
import { Loader2, Search, SlidersHorizontal, X } from "lucide-vue-next";
import type { SearchHit, SearchQuery } from "../../../domain/types";
import { searchRecipes } from "../../../services/api/search-api";

const props = defineProps<{
  bookId?: string | null;
  placeholder?: string;
}>();
const emit = defineEmits<{ select: [recipeId: string] }>();

const text = ref("");
const excludeAllergens = ref("");
const maxActive = ref<number | null>(null);
const hits = ref<SearchHit[]>([]);
const loading = ref(false);
const filtersOpen = ref(false);
const searchInput = ref<HTMLInputElement>();
let timer = 0;

function buildQuery(): SearchQuery {
  return {
    text: text.value.trim() || null,
    bookId: props.bookId ?? null,
    excludeAllergens: excludeAllergens.value
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean),
    maxActiveMinutes: maxActive.value,
    hydration: null,
    limit: 30,
  };
}

async function runSearch(): Promise<void> {
  loading.value = true;
  try {
    hits.value = await searchRecipes(buildQuery());
  } finally {
    loading.value = false;
  }
}

watch([text, excludeAllergens, maxActive, () => props.bookId], () => {
  window.clearTimeout(timer);
  timer = window.setTimeout(() => void runSearch(), 250);
});

function clearFilters(): void {
  excludeAllergens.value = "";
  maxActive.value = null;
}

function focus(): void {
  searchInput.value?.focus();
}

defineExpose({ focus });
</script>

<template>
  <div class="search-panel">
    <label class="search-input">
      <Search :size="15" />
      <input
        ref="searchInput"
        v-model="text"
        type="search"
        :placeholder="placeholder ?? 'Search recipes…'"
        aria-label="Search recipes"
      />
      <Loader2 v-if="loading" :size="15" class="spin" />
    </label>
    <div class="filter-bar">
      <button type="button" class="filter-toggle" @click="filtersOpen = !filtersOpen">
        <SlidersHorizontal :size="14" />
        Filters
        <span v-if="excludeAllergens || maxActive" class="filter-badge">On</span>
      </button>
      <button
        v-if="filtersOpen && (excludeAllergens || maxActive)"
        type="button"
        class="chip-clear"
        @click="clearFilters"
      >
        <X :size="13" /> Clear filters
      </button>
    </div>
    <div v-if="filtersOpen" class="chips">
      <label class="chip"
        >Exclude allergens<input v-model="excludeAllergens" placeholder="milk, egg"
      /></label>
      <label class="chip"
        >Max active min<input v-model.number="maxActive" type="number" min="1" placeholder="60"
      /></label>
    </div>
    <ul v-if="hits.length" class="results">
      <li v-for="hit in hits" :key="hit.recipeId">
        <button @click="emit('select', hit.recipeId)">
          <strong>{{ hit.title }}</strong>
          <span>{{ hit.snippet.replace(/<\/?mark>/g, "") }}</span>
        </button>
      </li>
    </ul>
    <p v-else-if="text.trim() && !loading" class="empty">No matches.</p>
  </div>
</template>

<style scoped>
.search-panel {
  display: grid;
  gap: 10px;
}
.search-input {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 12px;
  border: 1px solid #cbd3cd;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.85);
}
.search-input input {
  flex: 1;
  border: 0;
  background: transparent;
  outline: none;
  font-size: 13px;
}
.filter-bar {
  display: flex;
  align-items: center;
  gap: 8px;
}
.filter-toggle {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid #cbd3cd;
  background: rgba(255, 255, 255, 0.7);
  font-size: 12px;
  color: #4a5a52;
}
.filter-badge {
  padding: 0 6px;
  border-radius: 999px;
  background: #28643b;
  color: #fff;
  font-size: 10px;
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid #cbd3cd;
  font-size: 12px;
}
.chip input {
  width: 88px;
  border: 0;
  background: transparent;
  outline: none;
  font-size: 12px;
}
.chip-clear {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 999px;
  border: 1px solid #cbd3cd;
  background: transparent;
  font-size: 12px;
}
.results {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 280px;
  overflow: auto;
  border: 1px solid #cbd3cd;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.92);
}
.results li + li {
  border-top: 1px solid #e5ebe7;
}
.results button {
  width: 100%;
  text-align: left;
  padding: 10px 12px;
  border: 0;
  background: transparent;
  display: grid;
  gap: 4px;
}
.results strong {
  font-size: calc(14px * var(--reading-scale, 1));
}
.results span {
  font-size: calc(12px * var(--reading-scale, 1));
  color: #6d7972;
}
.results :deep(mark) {
  background: #fff3bf;
  padding: 0 2px;
}
.empty {
  margin: 0;
  font-size: calc(13px * var(--reading-scale, 1));
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
</style>
