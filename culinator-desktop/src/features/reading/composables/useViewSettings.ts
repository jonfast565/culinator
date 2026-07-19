import { ref, watch } from "vue";
import type { InjectionKey, Ref } from "vue";

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

export interface ViewSettingsContext {
  misePlacement: Ref<MisePlacement>;
  toggleMisePlacement: () => void;
  numberStyle: Ref<NumberStyle>;
  toggleNumberStyle: () => void;
}

export const VIEW_SETTINGS_KEY: InjectionKey<ViewSettingsContext> = Symbol("viewSettings");

const STORAGE_KEY = "culinator.misePlacement";
const NUMBER_STYLE_KEY = "culinator.numberStyle";

function readStoredPlacement(): MisePlacement {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored === "colocated" || stored === "top-matter") return stored;
  } catch {
    // ignore
  }
  return "top-matter";
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

  return { misePlacement, toggleMisePlacement, numberStyle, toggleNumberStyle };
}
