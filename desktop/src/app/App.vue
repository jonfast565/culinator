<script setup lang="ts">
/* global PointerEvent, HTMLElement */
import { onBeforeUnmount, ref } from "vue";
import { Save, Trash2, Plus, Route, Database } from "lucide-vue-next";
import RecipeBookSidebar from "../features/library/components/RecipeBookSidebar.vue";
import { useRecipeLibrary } from "../features/library/composables/useRecipeLibrary";
import SourceEditor from "../features/recipe-editor/components/SourceEditor.vue";
import InspectorPanel from "../features/recipe-editor/components/InspectorPanel.vue";
import { useRecipeEditor } from "../features/recipe-editor/composables/useRecipeEditor";
import RecipeImportPanel from "../features/import/components/RecipeImportPanel.vue";
import ConnectionBadge from "../shared/components/ConnectionBadge.vue";
import { openRecipeFile } from "../services/api";
import { onConnectionStatus, type ConnectionStatus } from "../services/transport/websocket-client";

const library = useRecipeLibrary();
const editor = useRecipeEditor(library.selectedRecipe);
const connection = ref<ConnectionStatus>("connecting");
const importing = ref(false);
const stopStatus = onConnectionStatus((status) => {
  connection.value = status;
});
onBeforeUnmount(stopStatus);

const clampInspector = (width: number): number => Math.min(760, Math.max(280, width));
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

async function createBook(): Promise<void> {
  const title = window.prompt("Recipe book name", "My Recipe Book")?.trim();
  if (title) await library.createBook(title);
}
async function renameBook(book: Parameters<typeof library.renameBook>[0]): Promise<void> {
  const title = window.prompt("Rename recipe book", book.title)?.trim();
  if (title) await library.renameBook(book, title);
}
async function deleteBook(book: Parameters<typeof library.deleteBook>[0]): Promise<void> {
  if (window.confirm(`Delete “${book.title}”? Recipes will become unfiled.`))
    await library.deleteBook(book);
}
async function selectRecipe(id: string): Promise<void> {
  if (editor.dirty.value && !window.confirm("Discard unsaved changes?")) return;
  await library.selectRecipe(id);
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
  await editor.remove();
  await library.refresh();
}
async function acceptImport(source: string): Promise<void> {
  await library.createRecipe();
  if (!library.selectedRecipe.value) return;
  editor.source.value = source;
  await save();
  importing.value = false;
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
</script>

<template>
  <div class="app-shell">
    <RecipeBookSidebar
      :books="library.books.value"
      :recipes="library.recipes.value"
      :selected-book-id="library.selectedBookId.value"
      :selected-recipe-id="library.selectedRecipe.value?.id"
      @select-book="library.selectedBookId.value = $event"
      @select-recipe="selectRecipe"
      @create-recipe="library.createRecipe"
      @import-recipe="importing = true"
      @import-file="importFromFile"
      @create-book="createBook"
      @rename-book="renameBook"
      @delete-book="deleteBook"
    />
    <main class="workspace">
      <header class="toolbar">
        <div>
          <h1>{{ library.selectedRecipe.value?.title ?? "No recipe selected" }}</h1>
          <small
            ><Database :size="13" /> SQLite · WebSocket <ConnectionBadge :status="connection"
          /></small>
        </div>
        <div class="toolbar-actions">
          <select
            v-if="library.selectedRecipe.value"
            :value="library.selectedRecipe.value.bookId ?? ''"
            @change="library.moveSelected(($event.target as HTMLSelectElement).value || null)"
          >
            <option value="">Unfiled</option>
            <option v-for="book in library.books.value" :key="book.id" :value="book.id">
              {{ book.title }}
            </option></select
          ><button @click="quickIngredient"><Plus :size="15" /> Ingredient</button
          ><button @click="quickOperation"><Route :size="15" /> Operation</button
          ><button class="danger" :disabled="!library.selectedRecipe.value" @click="remove">
            <Trash2 :size="15" /></button
          ><button
            class="primary"
            :disabled="!editor.dirty.value || editor.saving.value"
            @click="save"
          >
            <Save :size="15" /> {{ editor.saving.value ? "Saving…" : "Save" }}
          </button>
        </div>
      </header>
      <section
        v-if="library.selectedRecipe.value"
        class="editor-layout"
        :style="{ '--inspector-w': inspectorWidth + 'px' }"
      >
        <SourceEditor v-model="editor.source.value" />
        <div
          class="pane-resizer"
          role="separator"
          aria-orientation="vertical"
          title="Drag to resize"
          @pointerdown="startResize"
        ></div>
        <InspectorPanel
          :model="editor.model.value"
          :validation="editor.validation.value"
          :recipe-id="library.selectedRecipe.value.id"
          :source="editor.source.value"
          @update:source="editor.source.value = $event"
        />
      </section>
      <section v-else class="empty-workspace">
        <h2>Create or select a recipe</h2>
        <p>Recipes are organized inside recipe books and stored through the local service.</p>
      </section>
    </main>
  </div>
  <RecipeImportPanel v-if="importing" @close="importing = false" @accept="acceptImport" />
</template>
