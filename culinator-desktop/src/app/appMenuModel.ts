import type { AppView } from "./useNavigation";
import type { InspectorTabId } from "../features/recipe-editor/components/InspectorPanel.vue";
import type { IndexCardFormat } from "../features/reading/indexCardFormat";
import {
  INDEX_CARD_PICKER_OPTIONS,
  indexCardFormatLabel,
  indexCardMenuAction,
} from "../features/reading/indexCardFormat";
import { isMacPlatform } from "../services/platform";

/**
 * The application menu, described once and rendered twice: as the in-app menu
 * bar (`AppMenuBar.vue`) and — inside the Tauri shell — as the native system
 * menu (`useNativeMenu.ts`). Keeping the model here is what lets the in-app bar
 * be hidden without losing any commands.
 */

export type AppMenuAction =
  | "home"
  | "new-book"
  | "new-recipe"
  | "import-recipe"
  | "import-file"
  | "read"
  | "edit-source"
  | "build"
  | "save"
  | "delete"
  | "measures"
  | "ingredient-match"
  | "toggle-units"
  | "toggle-temperature"
  | "toggle-mise"
  | "toggle-numbers"
  | "cycle-text"
  | "toggle-book-layout"
  | "print-recipe-cards"
  | "print-book"
  | "print-index-card"
  | `set-index-card:${IndexCardFormat}`
  | "cycle-recipe-header-type"
  | "cycle-recipe-body-type"
  | "cycle-recipe-annotation-type"
  | "cycle-index-card-margin"
  | "toggle-index-card-pager"
  | "toggle-menu-bar"
  | "open-settings"
  | "convert-units"
  | `tool:${InspectorTabId}`;

export interface AppMenuItem {
  label: string;
  action: AppMenuAction;
  /** Secondary line in the in-app menu (dropped by the native menu). */
  hint?: string;
  /**
   * Tauri accelerator syntax (`CmdOrCtrl+S`). Drives both the native key
   * equivalent and the hint drawn in the in-app menu, so a key is described in
   * exactly one place. Only ever set on the item the key currently runs.
   */
  accelerator?: string;
  disabled?: boolean;
  /** Draw a separator above this item. */
  divider?: boolean;
}

export interface AppMenuSection {
  label: string;
  items: AppMenuItem[];
  /** Whole section is unavailable — recipe-only menus without an open recipe. */
  disabled?: boolean;
}

/** Everything the menu labels and enablement depend on. */
export interface AppMenuState {
  view: AppView;
  hasRecipe: boolean;
  dirty: boolean;
  saving: boolean;
  unitSystem: "metric" | "us_customary";
  temperatureScale: "celsius" | "fahrenheit";
  misePlacement: "top-matter" | "colocated";
  numberStyle: "fractions" | "decimals";
  textSizeLabel: string;
  bookLayout: "book" | "cards";
  indexCardFormat: IndexCardFormat;
  recipeHeaderTypeLabel: string;
  recipeBodyTypeLabel: string;
  recipeAnnotationTypeLabel: string;
  indexCardMarginLabel: string;
  showIndexCardPager: boolean;
  showMenuBar: boolean;
  onBookView: boolean;
  onRecipeView: boolean;
}

/** Toggles the in-app menu bar; the native menu keeps the commands reachable. */
export const MENU_BAR_ACCELERATOR = "CmdOrCtrl+Shift+M";

export function buildAppMenus(state: AppMenuState): AppMenuSection[] {
  const hasRecipe = state.hasRecipe;
  // ⌘E flips between the two editors, so the accelerator belongs to whichever
  // one it would switch *to* — never to both, which a native menu cannot do.
  const sourceIsNext = state.view === "building";
  return [
    {
      label: "File",
      items: [
        { label: "New recipe", action: "new-recipe", hint: "Create a blank recipe" },
        { label: "New book", action: "new-book" },
        { label: "Import recipe…", action: "import-recipe", divider: true },
        { label: "Open recipe file…", action: "import-file" },
        { label: "Recipe shelf", action: "home", divider: true },
      ],
    },
    {
      label: "Recipe",
      disabled: !hasRecipe,
      items: [
        { label: "Read recipe", action: "read", disabled: state.view === "reading" },
        {
          label: "Edit recipe",
          action: "build",
          hint: "Structured builder",
          accelerator: sourceIsNext ? undefined : "CmdOrCtrl+E",
        },
        {
          label: "Edit source",
          action: "edit-source",
          hint: "Raw DSL",
          accelerator: sourceIsNext ? "CmdOrCtrl+E" : undefined,
        },
        {
          label: state.saving ? "Saving…" : "Save changes",
          action: "save",
          accelerator: "CmdOrCtrl+S",
          disabled: !state.dirty || state.saving,
          divider: true,
        },
        { label: "Delete recipe…", action: "delete", divider: true },
      ],
    },
    {
      label: "View",
      items: [
        {
          label: state.unitSystem === "metric" ? "Use US units" : "Use metric units",
          action: "toggle-units",
          disabled: !hasRecipe,
        },
        {
          label: state.temperatureScale === "celsius" ? "Show Fahrenheit" : "Show Celsius",
          action: "toggle-temperature",
          disabled: !hasRecipe,
        },
        {
          label:
            state.misePlacement === "colocated"
              ? "Use one ingredient list"
              : "Place mise beside steps",
          action: "toggle-mise",
          disabled: !hasRecipe,
        },
        {
          label: state.numberStyle === "fractions" ? "Show decimal amounts" : "Show fractions",
          action: "toggle-numbers",
          disabled: !hasRecipe,
        },
        {
          label: `Text size: ${state.textSizeLabel}`,
          action: "cycle-text",
          disabled: !hasRecipe,
        },
        {
          label: state.bookLayout === "book" ? "Use recipe cards" : "Use page-flip book",
          action: "toggle-book-layout",
          disabled: !state.onBookView,
        },
        {
          label: "Print recipe cards…",
          action: "print-recipe-cards",
          disabled: !state.onBookView || state.bookLayout !== "cards",
          accelerator: state.bookLayout === "cards" ? "CmdOrCtrl+P" : undefined,
        },
        {
          label: "Print book…",
          action: "print-book",
          disabled: !state.onBookView,
          accelerator: state.bookLayout !== "cards" ? "CmdOrCtrl+P" : undefined,
        },
        ...INDEX_CARD_PICKER_OPTIONS.map((option, index) => ({
          label: option.id === "full" ? "Recipe card: Full page" : `Recipe card: ${option.label}`,
          action: indexCardMenuAction(option.id),
          disabled: !state.onRecipeView,
          divider: index === 0,
        })),
        {
          label:
            state.indexCardFormat === "full"
              ? "Print recipe…"
              : `Print ${indexCardFormatLabel(state.indexCardFormat)}…`,
          action: "print-index-card",
          disabled: !state.onRecipeView,
          accelerator: "CmdOrCtrl+P",
        },
        {
          label: `Title type: ${state.recipeHeaderTypeLabel}`,
          action: "cycle-recipe-header-type",
          disabled: !state.onRecipeView,
          divider: true,
        },
        {
          label: `Body type: ${state.recipeBodyTypeLabel}`,
          action: "cycle-recipe-body-type",
          disabled: !state.onRecipeView,
        },
        {
          label: `Notes type: ${state.recipeAnnotationTypeLabel}`,
          action: "cycle-recipe-annotation-type",
          disabled: !state.onRecipeView,
        },
        {
          label: `Card margins: ${state.indexCardMarginLabel}`,
          action: "cycle-index-card-margin",
          disabled: !state.onRecipeView,
        },
        {
          label: state.showIndexCardPager ? "Hide card numbers" : "Show card numbers",
          action: "toggle-index-card-pager",
          disabled: !state.onRecipeView,
        },
        {
          label: state.showMenuBar ? "Hide menu bar" : "Show menu bar",
          action: "toggle-menu-bar",
          accelerator: MENU_BAR_ACCELERATOR,
          divider: true,
        },
        { label: "Settings…", action: "open-settings" },
        { label: "Convert recipe units…", action: "convert-units", disabled: !hasRecipe },
        { label: "Measures & conversions", action: "measures", divider: true },
      ],
    },
    {
      label: "Preview",
      disabled: !hasRecipe,
      items: [
        { label: "Narrative", action: "tool:narrative" },
        { label: "Recipe outline", action: "tool:outline" },
        { label: "Ingredients", action: "tool:ingredients" },
      ],
    },
    {
      label: "Author",
      disabled: !hasRecipe,
      items: [
        { label: "Workflow graph", action: "tool:author" },
        { label: "Jump to source issues", action: "edit-source", hint: "Open source + Issues" },
      ],
    },
    {
      label: "Plan",
      disabled: !hasRecipe,
      items: [
        { label: "Timeline", action: "tool:timeline" },
        { label: "Formula editor", action: "tool:formula" },
      ],
    },
    {
      label: "Produce",
      disabled: !hasRecipe,
      items: [
        { label: "Cook mode", action: "tool:kitchen" },
        { label: "Food safety", action: "tool:haccp" },
        { label: "Nutrition", action: "tool:nutrition" },
        { label: "Ingredient matcher", action: "ingredient-match" },
      ],
    },
    {
      label: "Share",
      disabled: !hasRecipe,
      items: [{ label: "Export recipe", action: "tool:export" }],
    },
  ];
}

/** Modifier symbols, in the order macOS always draws them (⌃⌥⇧⌘). */
const MAC_MODIFIERS: [modifier: string, symbol: string][] = [
  ["Control", "⌃"],
  ["Alt", "⌥"],
  ["Shift", "⇧"],
  ["CmdOrCtrl", "⌘"],
];

/** Render an accelerator the way the host platform writes it: ⇧⌘M / Ctrl+Shift+M. */
export function formatAccelerator(accelerator: string, mac: boolean = isMacPlatform()): string {
  const parts = accelerator.split("+");
  const key = parts[parts.length - 1] ?? "";
  const modifiers = new Set(parts.slice(0, -1));
  if (!mac) {
    return [...modifiers]
      .map((part) => (part === "CmdOrCtrl" ? "Ctrl" : part))
      .concat(key)
      .join("+");
  }
  return (
    MAC_MODIFIERS.filter(([modifier]) => modifiers.has(modifier))
      .map(([, symbol]) => symbol)
      .join("") + key
  );
}
