import { ref, watch } from "vue";
import type { InjectionKey, Ref } from "vue";
import type { UnitSystem } from "../../../domain/types";
import type { UiRecipeModel } from "../../recipe-editor/model";
import { convertRecipeQuantitiesInSource } from "../../recipe-editor/sourcePatch";
import {
  collectRecipeTemperatures,
  convertQuantityForDisplay,
  convertQuantityForSource,
  convertTemperatureForDisplay,
  convertTemperatureForSource,
  detectAuthoredTemperatureScale,
  detectAuthoredUnitSystem,
  type TemperatureScale,
} from "../quantityConvert";

export type { TemperatureScale };

export interface UnitDisplayContext {
  unitSystem: Ref<UnitSystem>;
  temperatureScale: Ref<TemperatureScale>;
  toggleUnitSystem: () => void;
  toggleTemperatureScale: () => void;
  setUnitSystem: (system: UnitSystem) => void;
  setTemperatureScale: (scale: TemperatureScale) => void;
  /** Align display with a recipe's authored units when opening it. */
  syncToRecipe: (model: UiRecipeModel) => void;
  formatQuantity: (text: string | undefined) => Promise<string>;
  convertRecipeSource: (source: string, model: UiRecipeModel) => Promise<string>;
}

export const UNIT_DISPLAY_KEY: InjectionKey<UnitDisplayContext> = Symbol("unitDisplay");

const STORAGE_KEY = "culinator.unitSystem";
const TEMPERATURE_SCALE_KEY = "culinator.temperatureScale";

function readStoredSystem(): UnitSystem {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored === "us_customary" || stored === "metric") return stored;
  } catch {
    // ignore
  }
  return "metric";
}

function readStoredTemperatureScale(): TemperatureScale {
  try {
    const stored = window.localStorage.getItem(TEMPERATURE_SCALE_KEY);
    if (stored === "celsius" || stored === "fahrenheit") return stored;
  } catch {
    // ignore
  }
  return "celsius";
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

  const temperatureScale = ref<TemperatureScale>(readStoredTemperatureScale());
  watch(temperatureScale, (value) => {
    try {
      window.localStorage.setItem(TEMPERATURE_SCALE_KEY, value);
    } catch {
      // ignore
    }
  });

  function toggleUnitSystem(): void {
    unitSystem.value = unitSystem.value === "metric" ? "us_customary" : "metric";
  }

  function toggleTemperatureScale(): void {
    temperatureScale.value =
      temperatureScale.value === "celsius" ? "fahrenheit" : "celsius";
  }

  function setUnitSystem(system: UnitSystem): void {
    unitSystem.value = system;
  }

  function setTemperatureScale(scale: TemperatureScale): void {
    temperatureScale.value = scale;
  }

  function syncToRecipe(model: UiRecipeModel): void {
    const quantities: string[] = [];
    for (const resource of model.resources) {
      if (resource.kind === "ingredient" && resource.quantity) {
        quantities.push(resource.quantity);
      }
    }
    const authored = detectAuthoredUnitSystem(quantities);
    if (authored) unitSystem.value = authored;

    const authoredTemperature = detectAuthoredTemperatureScale(collectRecipeTemperatures(model));
    if (authoredTemperature) temperatureScale.value = authoredTemperature;
  }

  async function formatQuantity(text: string | undefined): Promise<string> {
    const raw = text?.trim();
    if (!raw) return raw ?? "";
    return convertQuantityForDisplay(raw, unitSystem.value);
  }

  async function convertRecipeSource(source: string, model: UiRecipeModel): Promise<string> {
    const ingredients = model.resources.filter((resource) => resource.kind === "ingredient");
    const convertIngredient = (text: string) => convertQuantityForSource(text, unitSystem.value);
    const convertTemperature = (text: string) =>
      convertTemperatureForSource(text, temperatureScale.value);
    return convertRecipeQuantitiesInSource(
      source,
      ingredients,
      model.operations ?? [],
      convertIngredient,
      convertTemperature,
    );
  }

  return {
    unitSystem,
    temperatureScale,
    toggleUnitSystem,
    toggleTemperatureScale,
    setUnitSystem,
    setTemperatureScale,
    syncToRecipe,
    formatQuantity,
    convertRecipeSource,
  };
}

export async function formatOperationTemperature(
  text: string | undefined,
  scale: TemperatureScale,
): Promise<string> {
  const raw = text?.trim();
  if (!raw) return raw ?? "";
  return convertTemperatureForDisplay(raw, scale);
}
