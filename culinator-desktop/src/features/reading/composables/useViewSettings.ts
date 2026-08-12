import { ref, watch } from "vue";
import type { InjectionKey, Ref } from "vue";
import { type IndexCardFormat, isIndexCardFormat } from "../indexCardFormat";
import {
  type IndexCardMargin,
  DEFAULT_INDEX_CARD_MARGIN,
  cycleIndexCardMargin,
  isIndexCardMargin,
} from "../indexCardMargin";
import { type RecipeTypeScale, cycleRecipeTypeScale, isRecipeTypeScale } from "../recipeTypeScale";
import { isTauri } from "../../../services/platform";

/**
 * How the reading page places ingredients and equipment.
 *
 * - `top-matter` — the traditional recipe-card layout: one ingredient list and
 *   one equipment list above the method.
 * - `colocated` — mise en place: each method section carries only what its own
 *   steps call for, and the top-matter lists are dropped.
 */
export type MisePlacement = "top-matter" | "colocated";

/**
 * How amounts are written: cooking fractions ("1/2 tsp") or plain decimals
 * ("0.5 tsp"). Fractions read like a recipe card; decimals are easier to scale
 * and to match against a kitchen scale's display.
 */
export type NumberStyle = "fractions" | "decimals";

/** Decimal places for plain-decimal amounts (0–3). */
export type DecimalPlaces = 0 | 1 | 2 | 3;

export const DECIMAL_PLACES_OPTIONS: DecimalPlaces[] = [0, 1, 2, 3];

/** User-selected scale for recipe and search-result text. */
export type TextSize = "default" | "large" | "x-large";

/**
 * How an open book presents its recipes.
 *
 * - `book` — page-flip folio with cover, table of contents, and section dividers.
 * - `cards` — a scrollable grid of recipe cards.
 */
export type BookLayout = "book" | "cards";

/**
 * On-screen framing for a single recipe in reading / editor preview.
 *
 * - `full` — the default folio layout (no physical card size).
 * - Physical page-dimension presets (e.g. `3x5`, `6x4`) for view and print.
 */
export type { IndexCardFormat };

export interface ViewSettingsContext {
  misePlacement: Ref<MisePlacement>;
  toggleMisePlacement: () => void;
  setMisePlacement: (placement: MisePlacement) => void;
  numberStyle: Ref<NumberStyle>;
  toggleNumberStyle: () => void;
  setNumberStyle: (style: NumberStyle) => void;
  /** Plain-decimal precision when number style is decimals (also fraction fallbacks). */
  decimalPlaces: Ref<DecimalPlaces>;
  setDecimalPlaces: (places: DecimalPlaces) => void;
  textSize: Ref<TextSize>;
  cycleTextSize: () => void;
  setTextSize: (size: TextSize) => void;
  bookLayout: Ref<BookLayout>;
  toggleBookLayout: () => void;
  setBookLayout: (layout: BookLayout) => void;
  indexCardFormat: Ref<IndexCardFormat>;
  setIndexCardFormat: (format: IndexCardFormat) => void;
  /** Card / print margin preset (tight / medium / wide). */
  indexCardMargin: Ref<IndexCardMargin>;
  cycleIndexCardMargin: () => void;
  setIndexCardMargin: (margin: IndexCardMargin) => void;
  /** Show “Card N of M” on index cards (and in print). */
  showIndexCardPager: Ref<boolean>;
  toggleIndexCardPager: () => void;
  /** Title / header type scale (reading + index cards). */
  recipeHeaderScale: Ref<RecipeTypeScale>;
  cycleRecipeHeaderScale: () => void;
  setRecipeHeaderScale: (scale: RecipeTypeScale) => void;
  /** Ingredients + steps type scale. */
  recipeBodyScale: Ref<RecipeTypeScale>;
  cycleRecipeBodyScale: () => void;
  setRecipeBodyScale: (scale: RecipeTypeScale) => void;
  /** Summary, meta, labels, pager type scale. */
  recipeAnnotationScale: Ref<RecipeTypeScale>;
  cycleRecipeAnnotationScale: () => void;
  setRecipeAnnotationScale: (scale: RecipeTypeScale) => void;
  /** Show the in-app menu bar (off by default in Tauri, which has a system menu). */
  showMenuBar: Ref<boolean>;
  toggleMenuBar: () => void;
  setShowMenuBar: (show: boolean) => void;
}

export type { RecipeTypeScale, IndexCardMargin };

export const VIEW_SETTINGS_KEY: InjectionKey<ViewSettingsContext> = Symbol("viewSettings");

const STORAGE_KEY = "culinator.misePlacement";
const NUMBER_STYLE_KEY = "culinator.numberStyle";
const DECIMAL_PLACES_KEY = "culinator.decimalPlaces";
const TEXT_SIZE_KEY = "culinator.textSize";
const BOOK_LAYOUT_KEY = "culinator.bookLayout";
const INDEX_CARD_FORMAT_KEY = "culinator.indexCardFormat";
const INDEX_CARD_FORMAT_MIGRATED_KEY = "culinator.indexCardFormat.migrated";
const INDEX_CARD_MARGIN_KEY = "culinator.indexCardMargin";
const INDEX_CARD_PAGER_KEY = "culinator.showIndexCardPager";
const HEADER_SCALE_KEY = "culinator.recipeHeaderScale";
const BODY_SCALE_KEY = "culinator.recipeBodyScale";
const ANNOTATION_SCALE_KEY = "culinator.recipeAnnotationScale";
const MENU_BAR_KEY = "culinator.showMenuBar";

function readStoredPlacement(): MisePlacement {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored === "colocated" || stored === "top-matter") return stored;
  } catch {
    // ignore
  }
  return "top-matter";
}

function readStoredDecimalPlaces(): DecimalPlaces {
  try {
    const stored = window.localStorage.getItem(DECIMAL_PLACES_KEY);
    if (stored === "0" || stored === "1" || stored === "2" || stored === "3") {
      return Number(stored) as DecimalPlaces;
    }
  } catch {
    // ignore
  }
  return 2;
}

function isDecimalPlaces(value: unknown): value is DecimalPlaces {
  return value === 0 || value === 1 || value === 2 || value === 3;
}

function readStoredNumberStyle(): NumberStyle {
  try {
    const stored = window.localStorage.getItem(NUMBER_STYLE_KEY);
    if (stored === "fractions" || stored === "decimals") return stored;
  } catch {
    // ignore
  }
  return "fractions";
}

function readStoredTextSize(): TextSize {
  try {
    const stored = window.localStorage.getItem(TEXT_SIZE_KEY);
    if (stored === "default" || stored === "large" || stored === "x-large") return stored;
  } catch {
    // ignore
  }
  return "default";
}

function readStoredBookLayout(): BookLayout {
  try {
    const stored = window.localStorage.getItem(BOOK_LAYOUT_KEY);
    if (stored === "book" || stored === "cards") return stored;
  } catch {
    // ignore
  }
  return "book";
}

function readStoredIndexCardFormat(): IndexCardFormat {
  try {
    const stored = window.localStorage.getItem(INDEX_CARD_FORMAT_KEY);
    if (!stored) return "full";

    const migrated = window.localStorage.getItem(INDEX_CARD_FORMAT_MIGRATED_KEY) === "1";
    if (!migrated) {
      persist(INDEX_CARD_FORMAT_MIGRATED_KEY, "1");
      // Pre-presets "4x6" was landscape 4×6 stock (6×4 page), not portrait 4×6.
      if (stored === "4x6") {
        persist(INDEX_CARD_FORMAT_KEY, "6x4");
        return "6x4";
      }
    }

    if (isIndexCardFormat(stored)) return stored;
  } catch {
    // ignore
  }
  return "full";
}

function readStoredIndexCardMargin(): IndexCardMargin {
  try {
    const stored = window.localStorage.getItem(INDEX_CARD_MARGIN_KEY);
    if (stored && isIndexCardMargin(stored)) return stored;
  } catch {
    // ignore
  }
  return DEFAULT_INDEX_CARD_MARGIN;
}

function readStoredShowIndexCardPager(): boolean {
  try {
    const stored = window.localStorage.getItem(INDEX_CARD_PAGER_KEY);
    if (stored === "false") return false;
    if (stored === "true") return true;
  } catch {
    // ignore
  }
  return true;
}

/**
 * The desktop shell puts the same commands in the system menu, so the in-app
 * menu bar starts hidden there and stays visible in the browser, where it is
 * the only menu the app has.
 */
function readStoredShowMenuBar(): boolean {
  try {
    const stored = window.localStorage.getItem(MENU_BAR_KEY);
    if (stored === "false") return false;
    if (stored === "true") return true;
  } catch {
    // ignore
  }
  return !isTauri();
}

function readStoredTypeScale(key: string): RecipeTypeScale {
  try {
    const stored = window.localStorage.getItem(key);
    if (stored && isRecipeTypeScale(stored)) return stored;
  } catch {
    // ignore
  }
  return "md";
}

function persist(key: string, value: string): void {
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // ignore
  }
}

export function useViewSettings(): ViewSettingsContext {
  const misePlacement = ref<MisePlacement>(readStoredPlacement());
  watch(misePlacement, (value) => {
    try {
      window.localStorage.setItem(STORAGE_KEY, value);
    } catch {
      // ignore
    }
  });

  function toggleMisePlacement(): void {
    misePlacement.value = misePlacement.value === "top-matter" ? "colocated" : "top-matter";
  }

  function setMisePlacement(placement: MisePlacement): void {
    misePlacement.value = placement;
  }

  const numberStyle = ref<NumberStyle>(readStoredNumberStyle());
  watch(numberStyle, (value) => {
    try {
      window.localStorage.setItem(NUMBER_STYLE_KEY, value);
    } catch {
      // ignore
    }
  });

  function toggleNumberStyle(): void {
    numberStyle.value = numberStyle.value === "fractions" ? "decimals" : "fractions";
  }

  function setNumberStyle(style: NumberStyle): void {
    numberStyle.value = style;
  }

  const decimalPlaces = ref<DecimalPlaces>(readStoredDecimalPlaces());
  watch(decimalPlaces, (value) => persist(DECIMAL_PLACES_KEY, String(value)));

  function setDecimalPlaces(places: DecimalPlaces): void {
    if (isDecimalPlaces(places)) decimalPlaces.value = places;
  }

  const textSize = ref<TextSize>(readStoredTextSize());
  watch(textSize, (value) => {
    try {
      window.localStorage.setItem(TEXT_SIZE_KEY, value);
    } catch {
      // ignore
    }
  });

  function cycleTextSize(): void {
    textSize.value =
      textSize.value === "default" ? "large" : textSize.value === "large" ? "x-large" : "default";
  }

  function setTextSize(size: TextSize): void {
    textSize.value = size;
  }

  const bookLayout = ref<BookLayout>(readStoredBookLayout());
  watch(bookLayout, (value) => {
    try {
      window.localStorage.setItem(BOOK_LAYOUT_KEY, value);
    } catch {
      // ignore
    }
  });

  function toggleBookLayout(): void {
    bookLayout.value = bookLayout.value === "book" ? "cards" : "book";
  }

  function setBookLayout(layout: BookLayout): void {
    bookLayout.value = layout;
  }

  const indexCardFormat = ref<IndexCardFormat>(readStoredIndexCardFormat());
  watch(indexCardFormat, (value) => {
    try {
      window.localStorage.setItem(INDEX_CARD_FORMAT_KEY, value);
    } catch {
      // ignore
    }
  });

  function setIndexCardFormat(format: IndexCardFormat): void {
    indexCardFormat.value = format;
  }

  const indexCardMargin = ref<IndexCardMargin>(readStoredIndexCardMargin());
  watch(indexCardMargin, (value) => persist(INDEX_CARD_MARGIN_KEY, value));
  function cycleIndexCardMarginSetting(): void {
    indexCardMargin.value = cycleIndexCardMargin(indexCardMargin.value);
  }

  function setIndexCardMarginSetting(margin: IndexCardMargin): void {
    if (isIndexCardMargin(margin)) indexCardMargin.value = margin;
  }

  const showIndexCardPager = ref(readStoredShowIndexCardPager());
  watch(showIndexCardPager, (value) => persist(INDEX_CARD_PAGER_KEY, String(value)));
  function toggleIndexCardPager(): void {
    showIndexCardPager.value = !showIndexCardPager.value;
  }

  const recipeHeaderScale = ref<RecipeTypeScale>(readStoredTypeScale(HEADER_SCALE_KEY));
  watch(recipeHeaderScale, (value) => persist(HEADER_SCALE_KEY, value));
  function cycleRecipeHeaderScale(): void {
    recipeHeaderScale.value = cycleRecipeTypeScale(recipeHeaderScale.value);
  }

  function setRecipeHeaderScale(scale: RecipeTypeScale): void {
    if (isRecipeTypeScale(scale)) recipeHeaderScale.value = scale;
  }

  const recipeBodyScale = ref<RecipeTypeScale>(readStoredTypeScale(BODY_SCALE_KEY));
  watch(recipeBodyScale, (value) => persist(BODY_SCALE_KEY, value));
  function cycleRecipeBodyScale(): void {
    recipeBodyScale.value = cycleRecipeTypeScale(recipeBodyScale.value);
  }

  function setRecipeBodyScale(scale: RecipeTypeScale): void {
    if (isRecipeTypeScale(scale)) recipeBodyScale.value = scale;
  }

  const recipeAnnotationScale = ref<RecipeTypeScale>(readStoredTypeScale(ANNOTATION_SCALE_KEY));
  watch(recipeAnnotationScale, (value) => persist(ANNOTATION_SCALE_KEY, value));
  function cycleRecipeAnnotationScale(): void {
    recipeAnnotationScale.value = cycleRecipeTypeScale(recipeAnnotationScale.value);
  }

  function setRecipeAnnotationScale(scale: RecipeTypeScale): void {
    if (isRecipeTypeScale(scale)) recipeAnnotationScale.value = scale;
  }

  const showMenuBar = ref(readStoredShowMenuBar());
  watch(showMenuBar, (value) => persist(MENU_BAR_KEY, String(value)));
  function toggleMenuBar(): void {
    showMenuBar.value = !showMenuBar.value;
  }

  function setShowMenuBar(show: boolean): void {
    showMenuBar.value = show;
  }

  return {
    misePlacement,
    toggleMisePlacement,
    setMisePlacement,
    numberStyle,
    toggleNumberStyle,
    setNumberStyle,
    decimalPlaces,
    setDecimalPlaces,
    textSize,
    cycleTextSize,
    setTextSize,
    bookLayout,
    toggleBookLayout,
    setBookLayout,
    indexCardFormat,
    setIndexCardFormat,
    indexCardMargin,
    cycleIndexCardMargin: cycleIndexCardMarginSetting,
    setIndexCardMargin: setIndexCardMarginSetting,
    showIndexCardPager,
    toggleIndexCardPager,
    recipeHeaderScale,
    cycleRecipeHeaderScale,
    setRecipeHeaderScale,
    recipeBodyScale,
    cycleRecipeBodyScale,
    setRecipeBodyScale,
    recipeAnnotationScale,
    cycleRecipeAnnotationScale,
    setRecipeAnnotationScale,
    showMenuBar,
    toggleMenuBar,
    setShowMenuBar,
  };
}
