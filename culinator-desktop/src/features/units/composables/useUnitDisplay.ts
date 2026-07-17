import { ref, watch } from "vue";
import type { InjectionKey, Ref } from "vue";
import type { UnitSystem } from "../../../domain/types";
import type { UiRecipeModel } from "../../recipe-editor/model";
import { convertRecipeQuantitiesInSource } from "../../recipe-editor/sourcePatch";
import { convertQuantityForDisplay, convertQuantityForSource } from "../quantityConvert";

export interface UnitDisplayContext {
  unitSystem: Ref<UnitSystem>;
  toggleUnitSystem: () => void;
  formatQuantity: (text: string | undefined) => Promise<string>;
  convertRecipeSource: (source: string, model: UiRecipeModel) => Promise<string>;
}

export const UNIT_DISPLAY_KEY: InjectionKey<UnitDisplayContext> = Symbol("unitDisplay");

const STORAGE_KEY = "culinator.unitSystem";

function readStoredSystem(): UnitSystem {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored === "us_customary" || stored === "metric") return stored;
  } catch {
    // ignore
  }
  return "metric";
}

export function useUnitDisplay() {
  const unitSystem = ref<UnitSystem>(readStoredSystem());
  watch(unitSystem, (value) => {
    try {
      window.localStorage.setItem(STORAGE_KEY, value);
    } catch {
      // ignore
    }
  });

  function toggleUnitSystem(): void {
    unitSystem.value = unitSystem.value === "metric" ? "us_customary" : "metric";
  }

  async function formatQuantity(text: string | undefined): Promise<string> {
    const raw = text?.trim();
    if (!raw) return raw ?? "";
    return convertQuantityForDisplay(raw, unitSystem.value);
  }

  async function convertRecipeSource(source: string, model: UiRecipeModel): Promise<string> {
    const ingredients = model.resources.filter((resource) => resource.kind === "ingredient");
    const convert = (text: string) => convertQuantityForSource(text, unitSystem.value);
    return convertRecipeQuantitiesInSource(source, ingredients, model.operations ?? [], convert);
  }

  return { unitSystem, toggleUnitSystem, formatQuantity, convertRecipeSource };
}

export async function formatOperationTemperature(
  text: string | undefined,
  system: UnitSystem,
): Promise<string> {
  const raw = text?.trim();
  if (!raw) return raw ?? "";
  return convertQuantityForDisplay(raw, system);
}
