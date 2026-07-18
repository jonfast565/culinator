<script setup lang="ts">
/* global PointerEvent, HTMLElement */
import { computed, onBeforeUnmount, provide, ref } from "vue";
import { Trash2, Database, Pencil, BookOpen, ChevronLeft, Scale, Ruler } from "lucide-vue-next";
import { useRecipeLibrary } from "../features/library/composables/useRecipeLibrary";
import EditDrawer from "../features/recipe-editor/components/EditDrawer.vue";
import { useRecipeEditor } from "../features/recipe-editor/composables/useRecipeEditor";
import RecipePage from "../features/reading/components/RecipePage.vue";
import Bookshelf from "../features/bookshelf/components/Bookshelf.vue";
import OpenBook from "../features/bookshelf/components/OpenBook.vue";
import MeasuresView from "../features/units/components/MeasuresView.vue";
import RecipeImportPanel from "../features/import/components/RecipeImportPanel.vue";
import ConnectionBadge from "../shared/components/ConnectionBadge.vue";
import { useNavigation } from "./useNavigation";
import { openRecipeFile } from "../services/api";
import { onConnectionStatus, type ConnectionStatus } from "../services/transport/websocket-client";
import { UNIT_DISPLAY_KEY, useUnitDisplay } from "../features/units/composables/useUnitDisplay";

const library = useRecipeLibrary();
const editor = useRecipeEditor(library.selectedRecipe);
const nav = useNavigation();
const unitDisplay = useUnitDisplay();
provide(UNIT_DISPLAY_KEY, unitDisplay);
const connection = ref<ConnectionStatus>("connecting");
const importing = ref(false);
const stopStatus = onConnectionStatus((status) => {
  connection.value = status;
});
onBeforeUnmount(stopStatus);

// The book currently open, and the recipes that belong to it (null = Unfiled).
const openBookSummary = computed(
  () => library.books.value.find((book) => book.id === nav.bookId.value) ?? null,
);
const openBookRecipes = computed(() =>
  library.recipes.value.filter((recipe) => (recipe.bookId ?? null) === nav.bookId.value),
);

const clampInspector = (width: number): number => Math.max(280, width);
const inspectorWidth = ref(
  clampInspector(Number(window.localStorage.getItem("cg:inspector-width")) || 390),
);
let resizeStartX = 0;
let resizeStartWidth = 0;
function startResize(event: PointerEvent): void {
  resizeStartX = event.clientX;
  resizeStartWidth = inspectorWidth.value;
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  window.addEventListener("pointermove", onResize);
  window.addEventListener("pointerup", stopResize);
}
function onResize(event: PointerEvent): void {
  inspectorWidth.value = clampInspector(resizeStartWidth - (event.clientX - resizeStartX));
}
function stopResize(): void {
  window.removeEventListener("pointermove", onResize);
  window.removeEventListener("pointerup", stopResize);
  window.localStorage.setItem("cg:inspector-width", String(inspectorWidth.value));
}
onBeforeUnmount(stopResize);

function openBook(bookId: string | null): void {
  library.selectedBookId.value = bookId;
  nav.openBook(bookId);
}
async function openRecipe(id: string): Promise<void> {
  await library.selectRecipe(id);
  nav.read();
}
async function bulkDelete(ids: string[]): Promise<void> {
  if (!ids.length) return;
  if (!window.confirm(`Delete ${ids.length} recipe${ids.length === 1 ? "" : "s"}?`)) return;
  await library.deleteRecipes(ids);
}
function backToBook(): void {
  nav.openBook(library.selectedRecipe.value?.bookId ?? nav.bookId.value);
}

async function createBook(): Promise<void> {
  const title = window.prompt("Recipe book name", "My Recipe Book")?.trim();
  if (title) await library.createBook(title);
}
async function renameBook(book: Parameters<typeof library.renameBook>[0]): Promise<void> {
  const title = window.prompt("Rename recipe book", book.title)?.trim();
  if (title) await library.renameBook(book, title);
}
async function deleteBook(book: Parameters<typeof library.deleteBook>[0]): Promise<void> {
  if (window.confirm(`Delete “${book.title}”? Recipes will become unfiled.`)) {
    await library.deleteBook(book);
    nav.shelf();
  }
}
async function newRecipe(): Promise<void> {
  await library.createRecipe();
  nav.edit();
}
async function save(): Promise<void> {
  const saved = await editor.save();
  if (saved) {
    library.selectedRecipe.value = saved;
    await library.refresh(saved.id);
  }
}
async function remove(): Promise<void> {
  if (
    !library.selectedRecipe.value ||
    !window.confirm(`Delete “${library.selectedRecipe.value.title}”?`)
  )
    return;
  const bookId = library.selectedRecipe.value.bookId ?? null;
  await editor.remove();
  await library.refresh();
  nav.openBook(bookId);
}
async function acceptImport(source: string): Promise<void> {
  await library.createRecipe();
  if (!library.selectedRecipe.value) return;
  editor.source.value = source;
  await save();
  importing.value = false;
  nav.read();
}
async function importFromFile(): Promise<void> {
  const file = await openRecipeFile();
  if (!file) return;
  await acceptImport(file.sourceText);
}
function quickIngredient(): void {
  editor.appendSnippet(
    `    ingredient new_ingredient measured by mass {\n        quantity 100 g;\n    }`,
  );
}
function quickOperation(): void {
  editor.appendSnippet(
    `    process preparation {\n        operation new_operation does mix {\n            duration 5 min;\n            labor active;\n        }\n    }`,
  );
}
async function convertRecipeUnits(): Promise<void> {
  if (!library.selectedRecipe.value) return;
  const target = unitDisplay.unitSystem.value === "metric" ? "metric" : "US customary";
  if (
    !window.confirm(
      `Convert convertible ingredient quantities and step temperatures in this recipe to ${target} units? Count-based measures (cloves, sticks, etc.) will stay unchanged.`,
    )
  ) {
    return;
  }
  const converted = await unitDisplay.convertRecipeSource(editor.source.value, editor.model.value);
  if (converted === editor.source.value) {
    window.alert("No convertible quantities were found to update.");
    return;
  }
  editor.source.value = converted;
  await save();
}
</script>

<template>
  <div class="app-root">
    <!-- Home: the bookshelf -->
    <Bookshelf
      v-if="nav.view.value === 'shelf'"
      :books="library.books.value"
      :recipes="library.recipes.value"
      @open-book="openBook"
      @open-recipe="openRecipe"
      @create-book="createBook"
      @create-recipe="newRecipe"
      @import-recipe="importing = true"
      @import-file="importFromFile"
      @rename-book="renameBook"
      @delete-book="deleteBook"
      @open-measures="nav.measures()"
    />

    <MeasuresView v-else-if="nav.view.value === 'measures'" @back="nav.shelf()" />

    <!-- An open book: flip through / search its recipes -->
    <OpenBook
      v-else-if="nav.view.value === 'book'"
      :book="openBookSummary"
      :recipes="openBookRecipes"
      :books="library.books.value"
      @back="nav.shelf()"
      @open-recipe="openRecipe"
      @bulk-move="library.moveRecipes"
      @bulk-delete="bulkDelete"
    />

    <!-- Reading a recipe as a book page -->
    <main
      v-else-if="library.selectedRecipe.value && nav.view.value === 'reading'"
      class="workspace"
    >
      <header class="reading-bar">
        <button class="ghost" @click="backToBook"><ChevronLeft :size="16" /> Book</button>
        <div class="reading-bar-title">
          <h1>{{ library.selectedRecipe.value.title }}</h1>
          <small
            ><Database :size="13" /> SQLite · WebSocket <ConnectionBadge :status="connection"
          /></small>
        </div>
        <div class="reading-bar-actions">
          <button
            class="ghost unit-toggle"
            :title="
              unitDisplay.unitSystem.value === 'metric' ? 'Switch to US units' : 'Switch to metric'
            "
            @click="unitDisplay.toggleUnitSystem()"
          >
            <Scale :size="15" />
            {{ unitDisplay.unitSystem.value === "metric" ? "Metric" : "US" }}
          </button>
          <button
            class="ghost"
            title="Rewrite convertible quantities in the recipe source"
            @click="convertRecipeUnits"
          >
            <Ruler :size="15" />
            Convert units
          </button>
          <button class="danger" title="Delete recipe" @click="remove">
            <Trash2 :size="15" />
          </button>
          <button class="primary" @click="nav.edit()"><Pencil :size="15" /> Edit</button>
        </div>
      </header>
      <div class="reading-stage">
        <RecipePage :model="editor.model.value" :recipe-id="library.selectedRecipe.value?.id" />
      </div>
    </main>

    <!-- Editing: the recipe page as a live preview + edit drawer -->
    <main
      v-else-if="library.selectedRecipe.value && nav.view.value === 'editing'"
      class="workspace"
    >
      <header class="reading-bar">
        <button class="ghost" @click="nav.read()"><ChevronLeft :size="16" /> Done</button>
        <div class="reading-bar-title">
          <h1>
            {{ library.selectedRecipe.value.title
            }}<span v-if="editor.dirty.value" class="dirty" title="Unsaved changes">•</span>
          </h1>
          <small
            ><Database :size="13" /> SQLite · WebSocket <ConnectionBadge :status="connection"
          /></small>
        </div>
        <div class="reading-bar-actions">
          <button
            class="ghost unit-toggle"
            :title="
              unitDisplay.unitSystem.value === 'metric' ? 'Switch to US units' : 'Switch to metric'
            "
            @click="unitDisplay.toggleUnitSystem()"
          >
            <Scale :size="15" />
            {{ unitDisplay.unitSystem.value === "metric" ? "Metric" : "US" }}
          </button>
          <button
            class="ghost"
            title="Rewrite convertible quantities in the recipe source"
            @click="convertRecipeUnits"
          >
            <Ruler :size="15" />
            Convert units
          </button>
        </div>
      </header>
      <section class="edit-layout" :style="{ '--inspector-w': inspectorWidth + 'px' }">
        <div class="reading-stage">
          <RecipePage :model="editor.model.value" :recipe-id="library.selectedRecipe.value?.id" />
        </div>
        <div
          class="pane-resizer"
          role="separator"
          aria-orientation="vertical"
          title="Drag to resize"
          @pointerdown="startResize"
        ></div>
        <EditDrawer
          :source="editor.source.value"
          :model="editor.model.value"
          :validation="editor.validation.value"
          :recipe-id="library.selectedRecipe.value.id"
          :books="library.books.value"
          :current-book-id="library.selectedRecipe.value.bookId ?? null"
          :dirty="editor.dirty.value"
          :saving="editor.saving.value"
          @update:source="editor.source.value = $event"
          @save="save"
          @close="nav.read()"
          @move-book="library.moveSelected($event)"
          @delete="remove"
          @insert-ingredient="quickIngredient"
          @insert-operation="quickOperation"
        />
      </section>
    </main>

    <!-- Fallback: no recipe selected but not on shelf/book -->
    <section v-else class="empty-workspace">
      <h2>Nothing open</h2>
      <button class="primary" @click="nav.shelf()"><BookOpen :size="15" /> Back to shelf</button>
    </section>
  </div>
  <RecipeImportPanel v-if="importing" @close="importing = false" @accept="acceptImport" />
</template>

<style scoped>
.app-root {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.workspace {
  flex: 1;
  min-height: 0;
}
.reading-bar {
  display: flex;
  align-items: center;
  gap: 16px;
  min-height: 72px;
  padding: 12px 18px;
  background: white;
  border-bottom: 1px solid #d8ddd9;
}
.reading-bar .ghost {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 34px;
  padding: 0 12px;
  font-size: 13px;
}
.reading-bar-title {
  min-width: 0;
  flex: 1;
}
.reading-bar-title h1 {
  margin: 0 0 3px;
  font-size: 19px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.reading-bar-title small {
  display: flex;
  align-items: center;
  gap: 5px;
}
.reading-bar-actions {
  display: flex;
  align-items: center;
  gap: 7px;
  flex-shrink: 0;
}
.reading-bar-actions button {
  height: 34px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  font-size: 13px;
}
.reading-bar-actions button.danger {
  width: 34px;
  padding: 0;
  justify-content: center;
}
.reading-stage {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: clamp(24px, 4vw, 56px) 20px;
  background: radial-gradient(120% 80% at 50% -10%, #efece2 0%, #e7e3d6 55%, #e0dbcb 100%);
}
.edit-layout {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(320px, 1fr) 6px var(--inspector-w, 420px);
}
.edit-layout .reading-stage {
  padding: clamp(18px, 3vw, 40px) 16px;
}
.dirty {
  margin-left: 6px;
  color: #c98a1a;
}
@media (max-width: 900px) {
  .edit-layout {
    grid-template-columns: 1fr;
  }
  .edit-layout .pane-resizer {
    display: none;
  }
}
.empty-workspace {
  margin: auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  color: #6e7a73;
}
</style>
