<script setup lang="ts">
/* global PointerEvent, HTMLElement, KeyboardEvent, DOMRect, EventTarget */
import { computed, nextTick, onBeforeUnmount, onMounted, provide, ref, watch } from "vue";
import {
  Database,
  Pencil,
  Blocks,
  BookOpen,
  ChevronLeft,
  ChefHat,
  FileCode2,
  Layers,
  Hash,
  PanelTopOpen,
  Ruler,
  Save,
  Thermometer,
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
import IngredientMatchView from "../features/nutrition/components/IngredientMatchView.vue";
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
import AppMenuBar from "./components/AppMenuBar.vue";
import {
  type AppMenuAction,
  MENU_BAR_ACCELERATOR,
  buildAppMenus,
  formatAccelerator,
} from "./appMenuModel";
import { useNativeMenu } from "./useNativeMenu";
import { isMacPlatform } from "../services/platform";
import AppSettingsDialog from "./components/AppSettingsDialog.vue";
import { printBook } from "../features/bookshelf/printBook";
import { printRecipeCards } from "../features/bookshelf/printRecipeCards";
import { INDEX_CARD_MARGIN_NAMES } from "../features/reading/indexCardMargin";
import { parseIndexCardMenuAction } from "../features/reading/indexCardFormat";
import { printIndexCard, printableDocumentTitle } from "../features/reading/printIndexCard";
import { RECIPE_TYPE_SCALE_LABELS } from "../features/reading/recipeTypeScale";
import IndexCardStage from "../features/reading/components/IndexCardStage.vue";
import IndexCardControls from "../features/reading/components/IndexCardControls.vue";
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
// When the open recipe changes, match unit and temperature display to that
// recipe's authored quantities so a leftover preference does not convert away
// from how the recipe was written.
watch(
  () => library.selectedRecipe.value?.id,
  (id) => {
    if (!id) return;
    unitDisplay.syncToRecipe(editor.model.value);
  },
);

const textSizeLabel = computed(() => {
  if (viewSettings.textSize.value === "large") return "A+";
  if (viewSettings.textSize.value === "x-large") return "A++";
  return "A";
});
const recipeHeaderTypeLabel = computed(
  () => RECIPE_TYPE_SCALE_LABELS[viewSettings.recipeHeaderScale.value],
);
const recipeBodyTypeLabel = computed(
  () => RECIPE_TYPE_SCALE_LABELS[viewSettings.recipeBodyScale.value],
);
const recipeAnnotationTypeLabel = computed(
  () => RECIPE_TYPE_SCALE_LABELS[viewSettings.recipeAnnotationScale.value],
);
const indexCardMarginLabel = computed(
  () => INDEX_CARD_MARGIN_NAMES[viewSettings.indexCardMargin.value],
);

const connection = ref<ConnectionStatus>("connecting");
const importing = ref(false);
const settingsOpen = ref(false);
const activeTool = ref<InspectorTabId | null>(null);
const kitchenMode = ref(false);
const ingredientMatchSymbol = ref<string | null>(null);
const pendingDiagnostic = ref<Diagnostic | null>(null);
const recipeBuilder = ref<{ focusSymbol: (symbol: string) => void } | null>(null);
/** Symbol highlighted in the live preview (click sync with the builder). */
const highlightedSymbol = ref<string | null>(null);

function symbolAtOffset(offset: number | null | undefined): string | null {
  if (offset == null) return null;
  const model = editor.model.value;
  for (const operation of model.operations) {
    if (operation.range && offset >= operation.range.start && offset < operation.range.end) {
      return operation.symbol;
    }
  }
  for (const resource of model.resources) {
    if (resource.range && offset >= resource.range.start && offset < resource.range.end) {
      return resource.symbol;
    }
  }
  return null;
}

function focusBuilderSymbol(symbol: string): void {
  highlightedSymbol.value = symbol;
  void nextTick(() => recipeBuilder.value?.focusSymbol(symbol));
}

const stopStatus = onConnectionStatus((status) => {
  connection.value = status;
});

const liveRecipeTitle = computed(
  () => editor.model.value.title || library.selectedRecipe.value?.title || "Untitled recipe",
);

const recipeBookTitle = computed(() => {
  const bookId = library.selectedRecipe.value?.bookId ?? null;
  if (!bookId) return null;
  return library.books.value.find((book) => book.id === bookId)?.title ?? null;
});

/** Save-as-PDF / print job name: includes the recipe book when available. */
const printRecipeDocumentTitle = computed(() =>
  printableDocumentTitle(liveRecipeTitle.value, recipeBookTitle.value),
);

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
const diagnosticsExpanded = ref(window.localStorage.getItem("cg:diagnostics-expanded") !== "0");
const diagnosticsRow = computed(() =>
  diagnosticsExpanded.value ? `minmax(90px, ${diagnosticsHeight.value}px)` : "34px",
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

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return (
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    tag === "SELECT" ||
    target.isContentEditable ||
    Boolean(target.closest(".cm-editor, [contenteditable='true']"))
  );
}

function onGlobalKeydown(event: KeyboardEvent): void {
  if (isSearchShortcut(event)) {
    event.preventDefault();
    if (nav.view.value === "shelf") triggerSearch("shelf");
    else if (nav.view.value === "book") triggerSearch("book");
    return;
  }

  // Inside the desktop shell these are the system menu's accelerators: the menu
  // runs the command, so handling them here as well would run it twice.
  const mod = (event.metaKey || event.ctrlKey) && !nativeMenu.active;

  if (mod && event.shiftKey && event.key.toLowerCase() === "m") {
    event.preventDefault();
    viewSettings.toggleMenuBar();
    return;
  }

  if (mod && event.key.toLowerCase() === "s" && library.selectedRecipe.value) {
    event.preventDefault();
    if (!saveBlocked.value) void save();
    return;
  }

  if (mod && event.key.toLowerCase() === "e" && library.selectedRecipe.value) {
    event.preventDefault();
    if (nav.view.value === "reading" || nav.view.value === "editing") nav.build();
    else if (nav.view.value === "building") editSource();
    return;
  }

  if (mod && event.key.toLowerCase() === "p" && nav.view.value === "book") {
    event.preventDefault();
    const bookTitle = openBookSummary.value?.title ?? "Book";
    if (viewSettings.bookLayout.value === "cards") {
      void printRecipeCards(bookTitle);
    } else if (nav.bookId.value) {
      void printBook(nav.bookId.value, bookTitle);
    }
    return;
  }

  if (
    mod &&
    event.key.toLowerCase() === "p" &&
    library.selectedRecipe.value &&
    ["reading", "editing", "building"].includes(nav.view.value)
  ) {
    event.preventDefault();
    void printIndexCard({
      format: viewSettings.indexCardFormat.value,
      margin: viewSettings.indexCardMargin.value,
      title: printRecipeDocumentTitle.value,
    });
    return;
  }

  if (event.key === "Escape") {
    if (activeTool.value) {
      event.preventDefault();
      activeTool.value = null;
      return;
    }
    if (
      (nav.view.value === "editing" || nav.view.value === "building") &&
      !isEditableTarget(event.target)
    ) {
      event.preventDefault();
      void leaveEdit();
    }
  }
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
  const massVolume = unitDisplay.unitSystem.value === "metric" ? "metric" : "US customary";
  const tempScale = unitDisplay.temperatureScale.value === "celsius" ? "Celsius" : "Fahrenheit";
  if (
    !(await dialog.confirm(
      `Convert convertible ingredient quantities to ${massVolume} units and step temperatures to ${tempScale}? Count-based measures (cloves, sticks, etc.) will stay unchanged.`,
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
  const symbol = symbolAtOffset(diagnostic.start);
  // Prefer the structured builder when the span maps to a declaration.
  if (symbol) {
    if (nav.view.value !== "building") nav.build();
    focusBuilderSymbol(symbol);
    return;
  }
  if (nav.view.value !== "editing") nav.edit();
}

function onPreviewSelect(symbol: string): void {
  if (nav.view.value !== "building") nav.build();
  focusBuilderSymbol(symbol);
}

function jumpToFirstError(): void {
  const first =
    editor.validation.value?.diagnostics.find((item) => item.severity === "error") ??
    editor.validation.value?.diagnostics[0];
  if (first) openDiagnostic(first);
}

function modKey(): string {
  return isMacPlatform() ? "⌘" : "Ctrl+";
}
function openTool(tool: InspectorTabId): void {
  if (!library.selectedRecipe.value) return;
  activeTool.value = tool;
}
function openIngredientMatcher(symbol?: string): void {
  if (!library.selectedRecipe.value) return;
  activeTool.value = null;
  kitchenMode.value = false;
  ingredientMatchSymbol.value = symbol ?? null;
  nav.ingredientMatch();
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
  const indexCardFormat = parseIndexCardMenuAction(action);
  if (indexCardFormat) {
    viewSettings.setIndexCardFormat(indexCardFormat);
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
    case "ingredient-match":
      openIngredientMatcher();
      break;
    case "toggle-units":
      unitDisplay.toggleUnitSystem();
      break;
    case "toggle-temperature":
      unitDisplay.toggleTemperatureScale();
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
    case "toggle-book-layout":
      viewSettings.toggleBookLayout();
      break;
    case "print-recipe-cards":
      if (nav.view.value === "book" && viewSettings.bookLayout.value === "cards") {
        void printRecipeCards(openBookSummary.value?.title ?? "Book");
      }
      break;
    case "print-book":
      if (nav.view.value === "book" && nav.bookId.value) {
        void printBook(nav.bookId.value, openBookSummary.value?.title ?? "Book");
      }
      break;
    case "print-index-card":
      if (library.selectedRecipe.value) {
        void printIndexCard({
          format: viewSettings.indexCardFormat.value,
          margin: viewSettings.indexCardMargin.value,
          title: printRecipeDocumentTitle.value,
        });
      }
      break;
    case "cycle-recipe-header-type":
      viewSettings.cycleRecipeHeaderScale();
      break;
    case "cycle-recipe-body-type":
      viewSettings.cycleRecipeBodyScale();
      break;
    case "cycle-recipe-annotation-type":
      viewSettings.cycleRecipeAnnotationScale();
      break;
    case "cycle-index-card-margin":
      viewSettings.cycleIndexCardMargin();
      break;
    case "toggle-index-card-pager":
      viewSettings.toggleIndexCardPager();
      break;
    case "toggle-menu-bar":
      viewSettings.toggleMenuBar();
      break;
    case "open-settings":
      settingsOpen.value = true;
      break;
    case "convert-units":
      await convertRecipeUnits();
      break;
  }
}
function saveStatusText(): string {
  const blocked =
    editor.dirty.value &&
    (editor.validation.value?.diagnostics.some((item) => item.severity === "error") ?? false);
  if (blocked) return "Fix issues to save";
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

const saveBlocked = computed(
  () =>
    editor.dirty.value &&
    (editor.validation.value?.diagnostics.some((item) => item.severity === "error") ?? false),
);

const appMenus = computed(() =>
  buildAppMenus({
    view: nav.view.value,
    hasRecipe: Boolean(activeRecipe.value),
    dirty: editor.dirty.value,
    saving: editor.saving.value,
    unitSystem: unitDisplay.unitSystem.value,
    temperatureScale: unitDisplay.temperatureScale.value,
    misePlacement: viewSettings.misePlacement.value,
    numberStyle: viewSettings.numberStyle.value,
    textSizeLabel: textSizeLabel.value,
    bookLayout: viewSettings.bookLayout.value,
    indexCardFormat: viewSettings.indexCardFormat.value,
    recipeHeaderTypeLabel: recipeHeaderTypeLabel.value,
    recipeBodyTypeLabel: recipeBodyTypeLabel.value,
    recipeAnnotationTypeLabel: recipeAnnotationTypeLabel.value,
    indexCardMarginLabel: indexCardMarginLabel.value,
    showIndexCardPager: viewSettings.showIndexCardPager.value,
    showMenuBar: viewSettings.showMenuBar.value,
    onBookView: nav.view.value === "book",
    onRecipeView:
      Boolean(library.selectedRecipe.value) &&
      ["reading", "editing", "building"].includes(nav.view.value),
  }),
);

// In the desktop shell the same menu is mirrored onto the system menu bar,
// which is why the in-app bar can default to hidden there.
const nativeMenu = useNativeMenu(appMenus, (action) => void handleMenuAction(action));
const menuBarHint = computed(() => formatAccelerator(MENU_BAR_ACCELERATOR));
</script>

<template>
  <div class="app-root">
    <AppMenuBar
      v-if="viewSettings.showMenuBar.value"
      :menus="appMenus"
      :view="nav.view.value"
      :recipe-title="activeRecipe ? liveRecipeTitle : undefined"
      :dirty="editor.dirty.value"
      @action="handleMenuAction"
    />
    <!--
      With the bar hidden the browser has no other way back to it; the desktop
      shell does — its system menu carries the same “Show menu bar” item.
    -->
    <button
      v-else-if="!nativeMenu.active"
      class="menu-bar-reveal"
      :title="`Show menu bar (${menuBarHint})`"
      aria-label="Show menu bar"
      @click="viewSettings.toggleMenuBar()"
    >
      <PanelTopOpen :size="12" />
    </button>
    <AppSettingsDialog :open="settingsOpen" @close="settingsOpen = false" />
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

    <IngredientMatchView
      v-else-if="nav.view.value === 'ingredient-match' && library.selectedRecipe.value"
      :key="library.selectedRecipe.value.id"
      :recipe-id="library.selectedRecipe.value.id"
      :resources="editor.model.value.resources"
      :recipe-title="liveRecipeTitle"
      :initial-symbol="ingredientMatchSymbol"
      @back="nav.backFromIngredientMatch()"
    />

    <OpenBook
      v-else-if="nav.view.value === 'book'"
      :book="openBookSummary"
      :recipes="openBookRecipes"
      :books="library.books.value"
      @back="nav.shelf()"
      @open-recipe="openRecipe"
      @create-recipe="newRecipe"
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
          <h1>{{ liveRecipeTitle }}</h1>
          <small
            ><Database :size="13" /> SQLite · WebSocket <ConnectionBadge :status="connection"
          /></small>
        </div>
        <div class="reading-bar-actions">
          <IndexCardControls v-if="!kitchenMode" compact :recipe-title="printRecipeDocumentTitle" />
          <button
            v-if="!kitchenMode"
            class="ghost compact"
            :title="
              unitDisplay.unitSystem.value === 'metric'
                ? 'Showing metric — switch to US customary'
                : 'Showing US customary — switch to metric'
            "
            @click="unitDisplay.toggleUnitSystem()"
          >
            <Ruler :size="14" />
            {{ unitDisplay.unitSystem.value === "metric" ? "Metric" : "US" }}
          </button>
          <button
            v-if="!kitchenMode"
            class="ghost compact"
            :title="
              unitDisplay.temperatureScale.value === 'celsius'
                ? 'Showing Celsius — switch to Fahrenheit'
                : 'Showing Fahrenheit — switch to Celsius'
            "
            @click="unitDisplay.toggleTemperatureScale()"
          >
            <Thermometer :size="14" />
            {{ unitDisplay.temperatureScale.value === "celsius" ? "°C" : "°F" }}
          </button>
          <button
            v-if="!kitchenMode && viewSettings.indexCardFormat.value !== 'full'"
            class="ghost compact"
            :title="
              viewSettings.misePlacement.value === 'colocated'
                ? 'Mise with each section — switch to top list'
                : 'Mise at top — switch to per-section'
            "
            @click="viewSettings.toggleMisePlacement()"
          >
            <Layers :size="14" />
            {{ viewSettings.misePlacement.value === "colocated" ? "By section" : "Top mise" }}
          </button>
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
            title="Edit the raw recipe source"
            @click="editSource"
          >
            <FileCode2 :size="15" /> Source
          </button>
          <button
            v-if="!kitchenMode"
            class="primary"
            :title="'Edit recipe (' + modKey() + 'E)'"
            @click="nav.build()"
          >
            <Pencil :size="15" /> Edit
          </button>
        </div>
      </header>
      <div class="reading-stage">
        <IndexCardStage>
          <RecipePage
            :model="editor.model.value"
            :source="editor.source.value"
            :recipe-id="library.selectedRecipe.value?.id"
            :kitchen-mode="kitchenMode"
            @kitchen-finished="kitchenMode = false"
          />
        </IndexCardStage>
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
            {{ liveRecipeTitle
            }}<span v-if="editor.dirty.value" class="dirty" title="Unsaved changes">•</span>
          </h1>
          <small class="save-hint" :class="[editor.saveStatus.value, { blocked: saveBlocked }]">{{
            saveStatusText()
          }}</small>
        </div>
        <div class="reading-bar-actions">
          <div class="view-toggles" role="group" aria-label="Preview display">
            <IndexCardControls compact :recipe-title="printRecipeDocumentTitle" />
            <button
              class="ghost compact"
              :title="
                unitDisplay.unitSystem.value === 'metric'
                  ? 'Showing metric — switch to US customary'
                  : 'Showing US customary — switch to metric'
              "
              @click="unitDisplay.toggleUnitSystem()"
            >
              <Ruler :size="14" />
              {{ unitDisplay.unitSystem.value === "metric" ? "Metric" : "US" }}
            </button>
            <button
              class="ghost compact"
              :title="
                unitDisplay.temperatureScale.value === 'celsius'
                  ? 'Showing Celsius — switch to Fahrenheit'
                  : 'Showing Fahrenheit — switch to Celsius'
              "
              @click="unitDisplay.toggleTemperatureScale()"
            >
              <Thermometer :size="14" />
              {{ unitDisplay.temperatureScale.value === "celsius" ? "°C" : "°F" }}
            </button>
            <button
              class="ghost compact"
              :title="
                viewSettings.misePlacement.value === 'colocated'
                  ? 'Mise with each section — switch to top list'
                  : 'Mise at top — switch to per-section'
              "
              @click="viewSettings.toggleMisePlacement()"
            >
              <Layers :size="14" />
              {{ viewSettings.misePlacement.value === "colocated" ? "By section" : "Top mise" }}
            </button>
            <button
              class="ghost compact"
              :title="
                viewSettings.numberStyle.value === 'fractions'
                  ? 'Fractions — switch to decimals'
                  : 'Decimals — switch to fractions'
              "
              @click="viewSettings.toggleNumberStyle()"
            >
              <Hash :size="14" />
              {{ viewSettings.numberStyle.value === "fractions" ? "½" : "0.5" }}
            </button>
          </div>
          <button
            class="primary"
            :disabled="!editor.dirty.value || editor.saving.value || saveBlocked"
            :title="modKey() + 'S'"
            @click="save"
          >
            <Save :size="15" /> {{ editor.saving.value ? "Saving…" : "Save" }}
          </button>
          <button
            v-if="nav.view.value === 'building'"
            class="ghost"
            :title="'Edit the raw recipe source (' + modKey() + 'E)'"
            @click="editSource"
          >
            <FileCode2 :size="15" /> Source
          </button>
          <button
            v-else
            class="ghost"
            :title="'Use the structured builder (' + modKey() + 'E)'"
            @click="nav.build()"
          >
            <Blocks :size="15" /> Builder
          </button>
        </div>
      </header>
      <p v-if="saveBlocked" class="save-blocked-banner">
        Autosave is paused while there are syntax errors.
        <button type="button" class="link" @click="jumpToFirstError">Jump to first issue</button>
      </p>
      <section
        class="edit-layout"
        :style="{
          '--editor-w': editorSplit + '%',
          '--diagnostics-h': diagnosticsHeight + 'px',
          '--diagnostics-row': diagnosticsRow,
        }"
      >
        <div class="reading-stage">
          <IndexCardStage>
            <RecipePage
              :model="editor.model.value"
              :recipe-id="library.selectedRecipe.value?.id"
              :source="editor.source.value"
              :editable="true"
              :highlighted-symbol="highlightedSymbol"
              @update:source="editor.source.value = $event"
              @select-symbol="onPreviewSelect"
            />
          </IndexCardStage>
        </div>
        <div
          class="pane-resizer"
          role="separator"
          aria-orientation="vertical"
          title="Drag to resize"
          @pointerdown="startResize"
        ></div>
        <div class="editor-stack" :class="{ 'diagnostics-collapsed': !diagnosticsExpanded }">
          <RecipeBuilderView
            v-if="nav.view.value === 'building'"
            ref="recipeBuilder"
            :source="editor.source.value"
            :model="editor.model.value"
            :recipe-id="library.selectedRecipe.value.id"
            @update:source="editor.source.value = $event"
            @edit-source="editSource"
            @focus-symbol="highlightedSymbol = $event"
            @open-formula-tool="openTool('formula')"
          />
          <EditDrawer
            v-else
            :source="editor.source.value"
            :model="editor.model.value"
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
            v-if="diagnosticsExpanded"
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
            @update:expanded="diagnosticsExpanded = $event"
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
      @open-ingredient-matcher="openIngredientMatcher"
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
.app-root > :not(.app-menu-bar):not(.menu-bar-reveal) {
  flex: 1;
  min-height: 0;
}
/* A pull-tab at the top edge: the only way back to a hidden menu bar. */
.menu-bar-reveal {
  position: fixed;
  top: 0;
  left: 50%;
  z-index: 40;
  transform: translateX(-50%);
  width: 46px;
  height: 15px;
  display: grid;
  place-items: center;
  padding: 0;
  border: 0;
  border-radius: 0 0 8px 8px;
  background: #17251e;
  color: #cfe0d3;
  opacity: 0.4;
  cursor: pointer;
  transition:
    opacity 0.12s ease,
    height 0.12s ease;
}
.menu-bar-reveal:hover,
.menu-bar-reveal:focus-visible {
  opacity: 1;
  height: 22px;
}
@media (prefers-reduced-motion: reduce) {
  .menu-bar-reveal {
    transition: none;
  }
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
  animation: saved-pulse 0.8s ease;
}
@keyframes saved-pulse {
  0% {
    opacity: 0.35;
  }
  40% {
    opacity: 1;
  }
  100% {
    opacity: 1;
  }
}
@media (prefers-reduced-motion: reduce) {
  .save-hint.saved {
    animation: none;
  }
}
.save-hint.error,
.save-hint.blocked {
  color: #a83737;
}
.save-blocked-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  padding: 8px 16px;
  background: #fbf1f1;
  color: #8a3a3a;
  font-size: 13px;
  border-bottom: 1px solid #efd5d5;
}
.save-blocked-banner .link {
  padding: 0;
  border: 0;
  background: transparent;
  color: #28643b;
  text-decoration: underline;
  font: inherit;
  cursor: pointer;
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
.view-toggles {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-right: 4px;
  padding: 2px;
  border-radius: 8px;
  background: #eceee9;
}
.reading-bar-actions .view-toggles button.compact {
  height: 28px;
  padding: 0 8px;
  font-size: 12px;
  border: 0;
  background: transparent;
  color: #45524b;
}
.reading-bar-actions .view-toggles button.compact:hover {
  background: #e4efe6;
  color: #28643b;
}
@media (max-width: 900px) {
  .view-toggles {
    display: none;
  }
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
  grid-template-rows: minmax(180px, 1fr) 6px var(--diagnostics-row, minmax(90px, 170px));
  overflow: hidden;
  background: #f7f6f2;
}
.editor-stack.diagnostics-collapsed {
  grid-template-rows: minmax(180px, 1fr) 34px;
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
