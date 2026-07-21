<script setup lang="ts">
/* global PointerEvent, HTMLElement, KeyboardEvent, DOMRect */
import { computed, onBeforeUnmount, onMounted, provide, ref, watch } from "vue";
import {
  Database,
  Pencil,
  Blocks,
  BookOpen,
  ChevronLeft,
  ChefHat,
  FileCode2,
  Save,
} from "lucide-vue-next";
import { useRecipeLibrary } from "../features/library/composables/useRecipeLibrary";
import DiagnosticsPane from "../features/recipe-editor/components/DiagnosticsPane.vue";
import EditDrawer from "../features/recipe-editor/components/EditDrawer.vue";
import type { InspectorTabId } from "../features/recipe-editor/components/InspectorPanel.vue";
import RecipeToolDialog from "../features/recipe-editor/components/RecipeToolDialog.vue";
import { useRecipeEditor } from "../features/recipe-editor/composables/useRecipeEditor";
import RecipePage from "../features/reading/components/RecipePage.vue";
import RecipeBuilderView from "../features/recipe-builder/components/RecipeBuilderView.vue";
import Bookshelf from "../features/bookshelf/components/Bookshelf.vue";
import OpenBook from "../features/bookshelf/components/OpenBook.vue";
import MeasuresView from "../features/units/components/MeasuresView.vue";
import RecipeImportPanel from "../features/import/components/RecipeImportPanel.vue";
import type { ImportAcceptPayload } from "../features/import/components/RecipeImportPanel.vue";
import ConnectionBadge from "../shared/components/ConnectionBadge.vue";
import { useAppDialog } from "../shared/composables/useAppDialog";
import { isSearchShortcut, triggerSearch } from "../shared/composables/useGlobalSearch";
import { useNavigation } from "./useNavigation";
import { openRecipeFile } from "../services/api";
import { onConnectionStatus, type ConnectionStatus } from "../services/transport/websocket-client";
import { UNIT_DISPLAY_KEY, useUnitDisplay } from "../features/units/composables/useUnitDisplay";
import {
  VIEW_SETTINGS_KEY,
  useViewSettings,
} from "../features/reading/composables/useViewSettings";
import AppMenuBar, { type AppMenuAction } from "./components/AppMenuBar.vue";
import type { Diagnostic } from "../domain/types";

const library = useRecipeLibrary();
const editor = useRecipeEditor(library.selectedRecipe);
const nav = useNavigation();
const unitDisplay = useUnitDisplay();
const dialog = useAppDialog();
provide(UNIT_DISPLAY_KEY, unitDisplay);
const viewSettings = useViewSettings();
provide(VIEW_SETTINGS_KEY, viewSettings);
watch(
  viewSettings.textSize,
  (size) => {
    document.documentElement.dataset.textSize = size;
  },
  { immediate: true },
);

const textSizeLabel = computed(() => {
  if (viewSettings.textSize.value === "large") return "A+";
  if (viewSettings.textSize.value === "x-large") return "A++";
  return "A";
});

const connection = ref<ConnectionStatus>("connecting");
const importing = ref(false);
const activeTool = ref<InspectorTabId | null>(null);
const kitchenMode = ref(false);
const pendingDiagnostic = ref<Diagnostic | null>(null);

const stopStatus = onConnectionStatus((status) => {
  connection.value = status;
});

const openBookSummary = computed(
  () => library.books.value.find((book) => book.id === nav.bookId.value) ?? null,
);
const openBookRecipes = computed(() =>
  library.recipes.value.filter((recipe) => (recipe.bookId ?? null) === nav.bookId.value),
);
const activeRecipe = computed(() =>
  ["reading", "editing", "building"].includes(nav.view.value) ? library.selectedRecipe.value : null,
);

const clampSplit = (value: number): number => Math.min(80, Math.max(20, value));
const storedSplit = Number(window.localStorage.getItem("cg:editor-split"));
const editorSplit = ref(clampSplit(storedSplit || 50));
let resizeBounds: DOMRect | null = null;
function startResize(event: PointerEvent): void {
  resizeBounds =
    (event.currentTarget as HTMLElement).parentElement?.getBoundingClientRect() ?? null;
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  window.addEventListener("pointermove", onResize);
  window.addEventListener("pointerup", stopResize);
}
function onResize(event: PointerEvent): void {
  if (!resizeBounds) return;
  editorSplit.value = clampSplit(((resizeBounds.right - event.clientX) / resizeBounds.width) * 100);
}
function stopResize(): void {
  window.removeEventListener("pointermove", onResize);
  window.removeEventListener("pointerup", stopResize);
  window.localStorage.setItem("cg:editor-split", String(editorSplit.value));
  resizeBounds = null;
}

const diagnosticsHeight = ref(
  Math.max(90, Number(window.localStorage.getItem("cg:diagnostics-height")) || 170),
);
let diagnosticsStartY = 0;
let diagnosticsStartHeight = 0;
let diagnosticsMaxHeight = 420;
function startDiagnosticsResize(event: PointerEvent): void {
  const parent = (event.currentTarget as HTMLElement).parentElement;
  diagnosticsStartY = event.clientY;
  diagnosticsStartHeight = diagnosticsHeight.value;
  diagnosticsMaxHeight = Math.max(120, (parent?.clientHeight ?? 600) - 180);
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  window.addEventListener("pointermove", onDiagnosticsResize);
  window.addEventListener("pointerup", stopDiagnosticsResize);
}
function onDiagnosticsResize(event: PointerEvent): void {
  diagnosticsHeight.value = Math.min(
    diagnosticsMaxHeight,
    Math.max(90, diagnosticsStartHeight + diagnosticsStartY - event.clientY),
  );
}
function stopDiagnosticsResize(): void {
  window.removeEventListener("pointermove", onDiagnosticsResize);
  window.removeEventListener("pointerup", stopDiagnosticsResize);
  window.localStorage.setItem("cg:diagnostics-height", String(diagnosticsHeight.value));
}

function onGlobalKeydown(event: KeyboardEvent): void {
  if (!isSearchShortcut(event)) return;
  event.preventDefault();
  if (nav.view.value === "shelf") triggerSearch("shelf");
  else if (nav.view.value === "book") triggerSearch("book");
}

onMounted(() => window.addEventListener("keydown", onGlobalKeydown));
onBeforeUnmount(() => {
  stopStatus();
  stopResize();
  stopDiagnosticsResize();
  window.removeEventListener("keydown", onGlobalKeydown);
});

function openBook(bookId: string | null): void {
  library.selectedBookId.value = bookId;
  nav.openBook(bookId);
}
async function openRecipe(id: string): Promise<void> {
  await library.selectRecipe(id);
  activeTool.value = null;
  kitchenMode.value = false;
  nav.read();
}
async function bulkDelete(ids: string[]): Promise<void> {
  if (!ids.length) return;
  if (!(await dialog.confirm(`Delete ${ids.length} recipe${ids.length === 1 ? "" : "s"}?`))) return;
  await library.deleteRecipes(ids);
}
async function backToBook(): Promise<void> {
  if (nav.view.value === "editing" && editor.dirty.value) {
    const leave = await dialog.confirm("You have unsaved changes. Leave without saving?");
    if (!leave) return;
  }
  nav.openBook(library.selectedRecipe.value?.bookId ?? nav.bookId.value);
}

async function createBook(): Promise<void> {
  const title = await dialog.prompt("Recipe book name", {
    defaultValue: "My Recipe Book",
    title: "New book",
    confirmLabel: "Create",
  });
  if (title) await library.createBook(title);
}
async function renameBook(book: Parameters<typeof library.renameBook>[0]): Promise<void> {
  const title = await dialog.prompt("Rename recipe book", {
    defaultValue: book.title,
    title: "Rename book",
    confirmLabel: "Save",
  });
  if (title) await library.renameBook(book, title);
}
async function deleteBook(book: Parameters<typeof library.deleteBook>[0]): Promise<void> {
  if (
    await dialog.confirm(`Delete “${book.title}”? Recipes will become unfiled.`, {
      title: "Delete book",
      confirmLabel: "Delete",
    })
  ) {
    await library.deleteBook(book);
    nav.shelf();
  }
}
async function newRecipe(): Promise<void> {
  await library.createRecipe();
  activeTool.value = null;
  // A brand-new recipe is a skeleton, which is exactly what the structured
  // builder is for — no source to hand-write first.
  nav.build();
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
    !(await dialog.confirm(`Delete “${library.selectedRecipe.value.title}”?`, {
      title: "Delete recipe",
      confirmLabel: "Delete",
    }))
  )
    return;
  const bookId = library.selectedRecipe.value.bookId ?? null;
  await editor.remove();
  await library.refresh();
  nav.openBook(bookId);
}
async function acceptImport(payload: ImportAcceptPayload): Promise<void> {
  await library.createRecipe();
  if (!library.selectedRecipe.value) return;
  editor.source.value = payload.source;
  await save();
  importing.value = false;
  if (payload.hasDiagnostics) {
    activeTool.value = null;
    nav.edit();
  } else {
    activeTool.value = null;
    nav.read();
  }
}
async function importFromFile(): Promise<void> {
  const file = await openRecipeFile();
  if (!file) return;
  await acceptImport({
    source: file.sourceText,
    title: file.fileName.replace(/\.(cg|txt)$/i, ""),
    hasDiagnostics: false,
  });
}
async function convertRecipeUnits(): Promise<void> {
  if (!library.selectedRecipe.value) return;
  const target = unitDisplay.unitSystem.value === "metric" ? "metric" : "US customary";
  if (
    !(await dialog.confirm(
      `Convert convertible ingredient quantities and step temperatures in this recipe to ${target} units? Count-based measures (cloves, sticks, etc.) will stay unchanged.`,
      { title: "Convert units", confirmLabel: "Convert" },
    ))
  ) {
    return;
  }
  const converted = await unitDisplay.convertRecipeSource(editor.source.value, editor.model.value);
  if (converted === editor.source.value) {
    await dialog.alert("No convertible quantities were found to update.");
    return;
  }
  editor.source.value = converted;
  await save();
}
async function leaveEdit(): Promise<void> {
  if (editor.dirty.value) {
    const leave = await dialog.confirm("You have unsaved changes. Leave without saving?", {
      title: "Unsaved changes",
      confirmLabel: "Leave",
    });
    if (!leave) return;
  }
  nav.read();
}
async function goHome(): Promise<void> {
  if (editor.dirty.value) {
    const leave = await dialog.confirm("You have unsaved changes. Leave without saving?", {
      title: "Unsaved changes",
      confirmLabel: "Leave",
    });
    if (!leave) return;
  }
  activeTool.value = null;
  nav.shelf();
}
function editSource(): void {
  activeTool.value = null;
  pendingDiagnostic.value = null;
  nav.edit();
}
function openDiagnostic(diagnostic: Diagnostic): void {
  pendingDiagnostic.value = { ...diagnostic };
  activeTool.value = null;
  if (nav.view.value !== "editing") nav.edit();
}
function openTool(tool: InspectorTabId): void {
  if (!library.selectedRecipe.value) return;
  activeTool.value = tool;
}
function enterKitchenMode(): void {
  activeTool.value = null;
  kitchenMode.value = true;
}
async function handleMenuAction(action: AppMenuAction): Promise<void> {
  if (action.startsWith("tool:")) {
    openTool(action.slice(5) as InspectorTabId);
    return;
  }
  switch (action) {
    case "home":
      await goHome();
      break;
    case "new-book":
      await createBook();
      break;
    case "new-recipe":
      await newRecipe();
      break;
    case "import-recipe":
      importing.value = true;
      break;
    case "import-file":
      await importFromFile();
      break;
    case "read":
      await leaveEdit();
      break;
    case "edit-source":
      editSource();
      break;
    case "build":
      activeTool.value = null;
      nav.build();
      break;
    case "save":
      await save();
      break;
    case "delete":
      await remove();
      break;
    case "measures":
      activeTool.value = null;
      nav.measures();
      break;
    case "toggle-units":
      unitDisplay.toggleUnitSystem();
      break;
    case "toggle-mise":
      viewSettings.toggleMisePlacement();
      break;
    case "toggle-numbers":
      viewSettings.toggleNumberStyle();
      break;
    case "cycle-text":
      viewSettings.cycleTextSize();
      break;
    case "convert-units":
      await convertRecipeUnits();
      break;
  }
}
function saveStatusText(): string {
  switch (editor.saveStatus.value) {
    case "saving":
      return "Saving…";
    case "saved":
      return "All changes saved";
    case "error":
      return "Auto-save failed";
    default:
      return editor.dirty.value ? "Unsaved changes" : "";
  }
}
</script>

<template>
  <div class="app-root">
    <AppMenuBar
      :view="nav.view.value"
      :has-recipe="Boolean(activeRecipe)"
      :recipe-title="activeRecipe?.title"
      :dirty="editor.dirty.value"
      :saving="editor.saving.value"
      :unit-system="unitDisplay.unitSystem.value"
      :mise-placement="viewSettings.misePlacement.value"
      :number-style="viewSettings.numberStyle.value"
      :text-size-label="textSizeLabel"
      @action="handleMenuAction"
    />
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
            v-if="!kitchenMode"
            class="ghost"
            title="Cook with step timers"
            @click="openTool('kitchen')"
          >
            <ChefHat :size="15" /> Cook
          </button>
          <button
            v-if="!kitchenMode"
            class="ghost"
            title="Build with structured forms"
            @click="nav.build()"
          >
            <Blocks :size="15" /> Build
          </button>
          <button v-if="!kitchenMode" class="primary" @click="nav.build()">
            <Pencil :size="15" /> Edit
          </button>
        </div>
      </header>
      <div class="reading-stage">
        <RecipePage
          :model="editor.model.value"
          :source="editor.source.value"
          :recipe-id="library.selectedRecipe.value?.id"
          :kitchen-mode="kitchenMode"
          @kitchen-finished="kitchenMode = false"
        />
      </div>
    </main>

    <main
      v-else-if="
        library.selectedRecipe.value &&
        (nav.view.value === 'editing' || nav.view.value === 'building')
      "
      class="workspace"
    >
      <header class="reading-bar">
        <button class="ghost" @click="leaveEdit"><ChevronLeft :size="16" /> Done</button>
        <div class="reading-bar-title">
          <h1>
            {{ library.selectedRecipe.value.title
            }}<span v-if="editor.dirty.value" class="dirty" title="Unsaved changes">•</span>
          </h1>
          <small class="save-hint" :class="editor.saveStatus.value">{{ saveStatusText() }}</small>
        </div>
        <div class="reading-bar-actions">
          <button
            class="primary"
            :disabled="!editor.dirty.value || editor.saving.value"
            @click="save"
          >
            <Save :size="15" /> {{ editor.saving.value ? "Saving…" : "Save" }}
          </button>
          <button
            v-if="nav.view.value === 'building'"
            class="ghost"
            title="Edit the raw recipe source"
            @click="editSource"
          >
            <FileCode2 :size="15" /> Source
          </button>
          <button v-else class="ghost" title="Use the structured builder" @click="nav.build()">
            <Blocks :size="15" /> Builder
          </button>
        </div>
      </header>
      <section
        class="edit-layout"
        :style="{
          '--editor-w': editorSplit + '%',
          '--diagnostics-h': diagnosticsHeight + 'px',
        }"
      >
        <div class="reading-stage">
          <RecipePage
            :model="editor.model.value"
            :recipe-id="library.selectedRecipe.value?.id"
            :source="editor.source.value"
          />
        </div>
        <div
          class="pane-resizer"
          role="separator"
          aria-orientation="vertical"
          title="Drag to resize"
          @pointerdown="startResize"
        ></div>
        <div class="editor-stack">
          <RecipeBuilderView
            v-if="nav.view.value === 'building'"
            :source="editor.source.value"
            :model="editor.model.value"
            :recipe-id="library.selectedRecipe.value.id"
            @update:source="editor.source.value = $event"
            @edit-source="editSource"
          />
          <EditDrawer
            v-else
            :source="editor.source.value"
            :validation="editor.validation.value"
            :dirty="editor.dirty.value"
            :saving="editor.saving.value"
            :save-status="editor.saveStatus.value"
            :initial-diagnostic="pendingDiagnostic"
            @update:source="editor.source.value = $event"
            @save="save"
            @close="leaveEdit"
          />
          <div
            class="diagnostics-resizer"
            role="separator"
            aria-orientation="horizontal"
            title="Drag to resize issues"
            @pointerdown="startDiagnosticsResize"
          ></div>
          <DiagnosticsPane
            :diagnostics="editor.validation.value?.diagnostics ?? []"
            :source="editor.source.value"
            @select="openDiagnostic"
          />
        </div>
      </section>
    </main>

    <section v-else class="empty-workspace">
      <h2>Nothing open</h2>
      <button class="primary" @click="nav.shelf()"><BookOpen :size="15" /> Back to shelf</button>
    </section>
    <RecipeToolDialog
      v-if="activeTool && library.selectedRecipe.value"
      :key="activeTool"
      :tool="activeTool"
      :model="editor.model.value"
      :recipe-id="library.selectedRecipe.value.id"
      :source="editor.source.value"
      @close="activeTool = null"
      @update:source="editor.source.value = $event"
      @kitchen-started="enterKitchenMode"
    />
  </div>
  <RecipeImportPanel v-if="importing" @close="importing = false" @accept="acceptImport" />
</template>

<style scoped>
.app-root {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.app-root > :not(.app-menu-bar) {
  flex: 1;
  min-height: 0;
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
.save-hint {
  display: block;
  font-size: 12px;
  color: #6d7972;
}
.save-hint.saved {
  color: #28643b;
}
.save-hint.error {
  color: #a83737;
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
  grid-template-columns:
    minmax(0, calc(100% - var(--editor-w, 50%) - 6px))
    6px minmax(0, var(--editor-w, 50%));
}
.edit-layout .reading-stage {
  padding: clamp(18px, 3vw, 40px) 16px;
}
.editor-stack {
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template-rows: minmax(180px, 1fr) 6px minmax(90px, var(--diagnostics-h, 170px));
  overflow: hidden;
  background: #f7f6f2;
}
.diagnostics-resizer {
  cursor: row-resize;
  background: #d8ddd9;
  transition: background 0.12s ease;
  touch-action: none;
}
.diagnostics-resizer:hover,
.diagnostics-resizer:active {
  background: #28643b;
}
.dirty {
  margin-left: 6px;
  color: #c98a1a;
}
@media (max-width: 900px) {
  .edit-layout {
    grid-template-columns: 1fr;
    grid-template-rows: minmax(280px, 1fr) minmax(360px, 1fr);
    overflow: auto;
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
