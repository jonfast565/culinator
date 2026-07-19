<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import {
  Plus,
  ScanLine,
  FileUp,
  Pencil,
  Trash2,
  UtensilsCrossed,
  Files,
  Ruler,
} from "lucide-vue-next";
import type { RecipeBookSummary, RecipeSummary } from "../../../domain/types";
import RecipeSearchPanel from "../../search/components/RecipeSearchPanel.vue";
import { registerSearchHandler } from "../../../shared/composables/useGlobalSearch";

const props = defineProps<{
  books: RecipeBookSummary[];
  recipes: RecipeSummary[];
}>();
const emit = defineEmits<{
  (event: "open-book", bookId: string | null): void;
  (event: "open-recipe", recipeId: string): void;
  (event: "create-book"): void;
  (event: "create-recipe"): void;
  (event: "import-recipe"): void;
  (event: "import-file"): void;
  (event: "rename-book", book: RecipeBookSummary): void;
  (event: "delete-book", book: RecipeBookSummary): void;
  (event: "open-measures"): void;
}>();

const showSearch = ref(window.localStorage.getItem("cg:shelf-search") === "1");
const searchPanel = ref<InstanceType<typeof RecipeSearchPanel>>();

const unfiledCount = computed(() => props.recipes.filter((recipe) => !recipe.bookId).length);

const CLOTHS = ["#2f4b3a", "#8a5a44", "#43364f", "#2d3b52", "#5a2f3a", "#2f5551", "#7a5a1f"];
function clothFor(id: string): string {
  let hash = 0;
  for (let index = 0; index < id.length; index += 1) hash = (hash * 31 + id.charCodeAt(index)) | 0;
  return CLOTHS[Math.abs(hash) % CLOTHS.length];
}

function toggleSearch(): void {
  showSearch.value = !showSearch.value;
  window.localStorage.setItem("cg:shelf-search", showSearch.value ? "1" : "0");
  if (showSearch.value) window.requestAnimationFrame(() => searchPanel.value?.focus());
}

let unregisterSearch = () => {};
onMounted(() => {
  unregisterSearch = registerSearchHandler("shelf", () => {
    if (!showSearch.value) {
      showSearch.value = true;
      window.localStorage.setItem("cg:shelf-search", "1");
    }
    window.requestAnimationFrame(() => searchPanel.value?.focus());
  });
});
onBeforeUnmount(unregisterSearch);
</script>

<template>
  <div class="shelf-view">
    <header class="shelf-head">
      <div class="brand">
        <span class="brand-mark"><UtensilsCrossed :size="20" /></span>
        <span>
          <strong>Culinator</strong>
          <small>Your recipe library</small>
        </span>
      </div>
      <div class="shelf-actions">
        <button @click="emit('open-measures')"><Ruler :size="15" /> Measures</button>
        <button :class="{ active: showSearch }" @click="toggleSearch">Search library</button>
        <button @click="emit('create-recipe')"><Plus :size="15" /> New recipe</button>
        <button @click="emit('import-recipe')"><ScanLine :size="15" /> Scan</button>
        <button @click="emit('import-file')"><FileUp :size="15" /> Import</button>
        <button class="primary" @click="emit('create-book')"><Plus :size="15" /> New book</button>
      </div>
    </header>

    <div v-if="showSearch" class="shelf-search">
      <RecipeSearchPanel
        ref="searchPanel"
        placeholder="Search all recipes… (⌘K)"
        @select="emit('open-recipe', $event)"
      />
    </div>

    <div class="shelf">
      <div class="books">
        <div
          v-for="book in books"
          :key="book.id"
          class="book"
          :style="{ '--cloth': clothFor(book.id) }"
          role="button"
          tabindex="0"
          @click="emit('open-book', book.id)"
          @keydown.enter="emit('open-book', book.id)"
        >
          <div class="book-actions">
            <button title="Rename book" @click.stop="emit('rename-book', book)">
              <Pencil :size="13" />
            </button>
            <button title="Delete book" @click.stop="emit('delete-book', book)">
              <Trash2 :size="13" />
            </button>
          </div>
          <h2 class="book-title">{{ book.title }}</h2>
          <p class="book-count">
            {{ book.recipeCount }} recipe{{ book.recipeCount === 1 ? "" : "s" }}
          </p>
        </div>

        <div
          v-if="unfiledCount"
          class="book unfiled"
          role="button"
          tabindex="0"
          @click="emit('open-book', null)"
          @keydown.enter="emit('open-book', null)"
        >
          <span class="unfiled-mark"><Files :size="26" /></span>
          <h2 class="book-title">Unfiled</h2>
          <p class="book-count">{{ unfiledCount }} recipe{{ unfiledCount === 1 ? "" : "s" }}</p>
        </div>

        <button
          v-if="!books.length && !unfiledCount"
          class="empty-book"
          @click="emit('create-book')"
        >
          <Plus :size="22" />
          <span>Create your first book</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.shelf-view {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: #201a12;
}
.shelf-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  flex-wrap: wrap;
  padding: 16px 22px;
  background: #18120b;
  border-bottom: 1px solid #2e2416;
}
.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  color: #f4ecdd;
}
.brand-mark {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: #d9f0df;
  color: #1f5130;
}
.brand strong {
  display: block;
  font-size: 17px;
}
.brand small {
  color: #b09b7d;
}
.shelf-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.shelf-actions button {
  height: 36px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 13px;
  font-size: 13px;
  background: #2c2213;
  color: #f0e6d4;
  border: 1px solid #3d3020;
  border-radius: 8px;
}
.shelf-actions button:hover {
  background: #392c19;
}
.shelf-actions button.active {
  background: #3d3020;
  border-color: #5a4a32;
}
.shelf-actions button.primary {
  background: #28643b;
  border-color: #28643b;
  color: #fff;
}
.shelf-search {
  padding: 0 22px 16px;
  background: #18120b;
  border-bottom: 1px solid #2e2416;
}

.shelf {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 40px 28px 60px;
  background:
    repeating-linear-gradient(
      90deg,
      rgba(0, 0, 0, 0.05) 0px,
      rgba(0, 0, 0, 0.05) 1px,
      transparent 1px,
      transparent 140px
    ),
    linear-gradient(180deg, #4a3623 0%, #3c2c1c 100%);
}
.books {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(168px, 1fr));
  gap: 34px 26px;
  max-width: 1100px;
  margin: 0 auto;
}
.book {
  position: relative;
  aspect-ratio: 3 / 4;
  padding: 20px 18px 18px 24px;
  border-radius: 4px 7px 7px 4px;
  color: #f6eede;
  background:
    linear-gradient(
      90deg,
      rgba(0, 0, 0, 0.32) 0 8px,
      rgba(255, 255, 255, 0.08) 8px 11px,
      transparent 11px
    ),
    linear-gradient(160deg, color-mix(in srgb, var(--cloth) 88%, white 12%), var(--cloth));
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.12) inset,
    14px 18px 26px -14px rgba(0, 0, 0, 0.7);
  cursor: pointer;
  display: flex;
  flex-direction: column;
  transition:
    transform 0.18s ease,
    box-shadow 0.18s ease;
}
.book:hover,
.book:focus-visible {
  transform: translateY(-10px) rotate(-1deg);
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.12) inset,
    20px 26px 34px -14px rgba(0, 0, 0, 0.75);
  outline: none;
}
.book-title {
  margin: 4px 0 0;
  font-family: "Iowan Old Style", "Palatino Linotype", Palatino, Georgia, serif;
  font-size: 20px;
  font-weight: 600;
  line-height: 1.15;
}
.book-count {
  margin-top: auto;
  font-size: 12px;
  letter-spacing: 0.04em;
  color: rgba(255, 255, 255, 0.72);
}
.book-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s ease;
}
.book:hover .book-actions,
.book:focus-within .book-actions {
  opacity: 1;
}
.book-actions button {
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  padding: 0;
  border-radius: 6px;
  border: 0;
  background: rgba(0, 0, 0, 0.28);
  color: #f6eede;
}
.book-actions button:hover {
  background: rgba(0, 0, 0, 0.5);
}
.unfiled {
  --cloth: #4a4a44;
  align-items: flex-start;
}
.unfiled-mark {
  color: rgba(255, 255, 255, 0.85);
  margin-bottom: 8px;
}
.empty-book {
  aspect-ratio: 3 / 4;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  border: 2px dashed rgba(255, 255, 255, 0.3);
  border-radius: 6px;
  background: transparent;
  color: rgba(255, 255, 255, 0.75);
  font-size: 14px;
}
.empty-book:hover {
  border-color: rgba(255, 255, 255, 0.55);
  color: #fff;
}
</style>
