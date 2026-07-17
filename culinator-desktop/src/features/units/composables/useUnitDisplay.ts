import { ref, watch } from "vue";
import type { InjectionKey, Ref } from "vue";
import type { UnitSystem } from "../../../domain/types";
import { convertUnits, formatUnit } from "../../../services/api/units-api";

export interface UnitDisplayContext {
  unitSystem: Ref<UnitSystem>;
  toggleUnitSystem: () => void;
  formatQuantity: (text: string | undefined) => Promise<string>;
}

export const UNIT_DISPLAY_KEY: InjectionKey<UnitDisplayContext> = Symbol("unitDisplay");

const STORAGE_KEY = "culinator.unitSystem";

const METRIC_MASS = new Set(["g", "gram", "grams", "kg", "kilogram", "kilograms", "mg"]);
const US_MASS = new Set(["oz", "ounce", "ounces", "lb", "lbs", "pound", "pounds"]);
const METRIC_VOLUME = new Set(["ml", "milliliter", "milliliters", "l", "liter", "liters", "cl"]);
const US_VOLUME = new Set([
  "cup",
  "cups",
  "tbsp",
  "tablespoon",
  "tablespoons",
  "tsp",
  "teaspoon",
  "teaspoons",
  "floz",
  "fl oz",
  "pt",
  "pint",
  "pints",
  "qt",
  "quart",
  "quarts",
  "gal",
  "gallon",
  "gallons",
]);
const TEMPERATURE = new Set(["c", "f", "celsius", "fahrenheit"]);

function readStoredSystem(): UnitSystem {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored === "us_customary" || stored === "metric") return stored;
  } catch {
    // ignore
  }
  return "metric";
}

function parseNumeric(raw: string): number | null {
  const trimmed = raw.trim();
  if (!trimmed) return null;
  if (trimmed.includes("/")) {
    const [num, den] = trimmed.split("/").map((part) => Number(part.trim()));
    if (Number.isFinite(num) && Number.isFinite(den) && den !== 0) return num / den;
    return null;
  }
  const value = Number(trimmed);
  return Number.isFinite(value) ? value : null;
}

function parseQuantity(text: string): { value: number; unit: string } | null {
  const match = text.trim().match(/^([\d./]+(?:\s+[\d./]+)?)\s*(.*)$/);
  if (!match) return null;
  const value = parseNumeric(match[1].replace(/\s+/g, " ").split(" ")[0] ?? match[1]);
  const unit = match[2].trim().toLowerCase();
  if (value == null || !unit) return null;
  return { value, unit };
}

function targetUnit(unit: string, system: UnitSystem): string | null {
  const normalized = unit.toLowerCase();
  if (METRIC_MASS.has(normalized) || US_MASS.has(normalized)) {
    return system === "metric" ? "g" : "oz";
  }
  if (METRIC_VOLUME.has(normalized) || US_VOLUME.has(normalized)) {
    return system === "metric" ? "ml" : "cup";
  }
  if (TEMPERATURE.has(normalized)) {
    return system === "metric" ? "c" : "f";
  }
  return null;
}

function alreadyInSystem(unit: string, system: UnitSystem): boolean {
  const normalized = unit.toLowerCase();
  if (METRIC_MASS.has(normalized) || METRIC_VOLUME.has(normalized)) {
    return system === "metric";
  }
  if (US_MASS.has(normalized) || US_VOLUME.has(normalized)) {
    return system === "us_customary";
  }
  if (TEMPERATURE.has(normalized)) {
    return (
      (system === "metric" && (normalized === "c" || normalized === "celsius")) ||
      (system === "us_customary" && (normalized === "f" || normalized === "fahrenheit"))
    );
  }
  return true;
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
    const parsed = parseQuantity(raw);
    if (!parsed || alreadyInSystem(parsed.unit, unitSystem.value)) return raw;

    const toUnit = targetUnit(parsed.unit, unitSystem.value);
    if (!toUnit) return raw;

    try {
      const converted = await convertUnits({
        value: parsed.value,
        fromUnit: parsed.unit,
        toUnit,
      });
      if (converted.dimension === "unknown" || converted.dimension === "same") return raw;
      const formatted = await formatUnit({
        value: converted.value,
        unit: converted.unit,
        unitSystem: unitSystem.value,
      });
      return formatted.formatted;
    } catch {
      return raw;
    }
  }

  return { unitSystem, toggleUnitSystem, formatQuantity };
}
