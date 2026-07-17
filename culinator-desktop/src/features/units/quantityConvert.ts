import type { UnitSystem } from "../../domain/types";
import { convertUnits, formatUnit } from "../../services/api/units-api";

const METRIC_MASS = new Set([
  "g",
  "gram",
  "grams",
  "gm",
  "gms",
  "kg",
  "kilogram",
  "kilograms",
  "kilo",
  "kilos",
  "mg",
  "milligram",
  "milligrams",
]);
const US_MASS = new Set(["oz", "ounce", "ounces", "lb", "lbs", "pound", "pounds", "dram", "drams"]);
const METRIC_VOLUME = new Set([
  "ml",
  "milliliter",
  "milliliters",
  "millilitre",
  "millilitres",
  "cc",
  "cl",
  "centiliter",
  "centiliters",
  "dl",
  "deciliter",
  "deciliters",
  "l",
  "liter",
  "liters",
  "litre",
  "litres",
]);
const US_VOLUME = new Set([
  "cup",
  "cups",
  "tbsp",
  "tbsps",
  "tbs",
  "tablespoon",
  "tablespoons",
  "tsp",
  "tsps",
  "teaspoon",
  "teaspoons",
  "floz",
  "fl oz",
  "fl_oz",
  "fluid_ounce",
  "fluid_ounces",
  "pt",
  "pint",
  "pints",
  "qt",
  "quart",
  "quarts",
  "gal",
  "gallon",
  "gallons",
  "dash",
  "dashes",
  "pinch",
  "pinches",
  "smidgen",
  "smidgens",
  "drop",
  "drops",
]);
const TEMPERATURE = new Set(["c", "f", "k", "celsius", "centigrade", "fahrenheit", "kelvin"]);
const TIME = new Set([
  "s",
  "sec",
  "secs",
  "second",
  "seconds",
  "min",
  "mins",
  "minute",
  "minutes",
  "h",
  "hr",
  "hrs",
  "hour",
  "hours",
]);
const COUNT = new Set([
  "count",
  "each",
  "clove",
  "cloves",
  "stick",
  "sticks",
  "slice",
  "slices",
  "piece",
  "pieces",
  "head",
  "heads",
  "bunch",
  "bunches",
  "can",
  "cans",
  "package",
  "packages",
]);

export interface ParsedQuantity {
  value: number;
  unit: string;
  maxValue?: number;
  maxUnit?: string;
}

function normalizeUnit(unit: string): string {
  return unit.trim().toLowerCase().replace(/\.$/, "");
}

export function parseNumeric(raw: string): number | null {
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

/** Parse a quantity expression such as `100 g`, `1/2 cup`, or `0.5 count to 1 count`. */
export function parseQuantity(text: string): ParsedQuantity | null {
  const trimmed = text.trim();
  const range = trimmed.match(
    /^([\d./]+(?:\s+[\d./]+)?)\s+([A-Za-z_]+(?:\s+[A-Za-z_]+)?)\s+to\s+([\d./]+(?:\s+[\d./]+)?)\s+([A-Za-z_]+(?:\s+[A-Za-z_]+)?)$/i,
  );
  if (range) {
    const value = parseNumeric(range[1].replace(/\s+/g, " ").split(" ")[0] ?? range[1]);
    const maxValue = parseNumeric(range[3].replace(/\s+/g, " ").split(" ")[0] ?? range[3]);
    const unit = normalizeUnit(range[2]);
    const maxUnit = normalizeUnit(range[4]);
    if (value == null || maxValue == null || !unit || !maxUnit) return null;
    return { value, unit, maxValue, maxUnit };
  }

  const match = trimmed.match(/^([\d./]+(?:\s+[\d./]+)?)\s*(.*)$/);
  if (!match) return null;
  const value = parseNumeric(match[1].replace(/\s+/g, " ").split(" ")[0] ?? match[1]);
  const unit = normalizeUnit(match[2]);
  if (value == null || !unit) return null;
  return { value, unit };
}

export function quantityDimension(unit: string): string | null {
  const normalized = normalizeUnit(unit);
  if (METRIC_MASS.has(normalized) || US_MASS.has(normalized)) return "mass";
  if (METRIC_VOLUME.has(normalized) || US_VOLUME.has(normalized)) return "volume";
  if (TEMPERATURE.has(normalized)) return "temperature";
  if (TIME.has(normalized)) return "time";
  if (COUNT.has(normalized)) return "count";
  return null;
}

export function isConvertibleUnit(unit: string): boolean {
  const dimension = quantityDimension(unit);
  return dimension != null && dimension !== "count";
}

export function targetUnit(unit: string, system: UnitSystem): string | null {
  const normalized = normalizeUnit(unit);
  const dimension = quantityDimension(normalized);
  if (!dimension || dimension === "count") return null;
  if (dimension === "mass") return system === "metric" ? "g" : "oz";
  if (dimension === "volume") return system === "metric" ? "ml" : "cup";
  if (dimension === "temperature") return system === "metric" ? "c" : "f";
  if (dimension === "time") return normalized;
  return null;
}

export function alreadyInSystem(unit: string, system: UnitSystem): boolean {
  const normalized = normalizeUnit(unit);
  if (METRIC_MASS.has(normalized) || METRIC_VOLUME.has(normalized)) return system === "metric";
  if (US_MASS.has(normalized) || US_VOLUME.has(normalized)) return system === "us_customary";
  if (TEMPERATURE.has(normalized)) {
    return (
      (system === "metric" &&
        (normalized === "c" || normalized === "celsius" || normalized === "centigrade")) ||
      (system === "us_customary" && (normalized === "f" || normalized === "fahrenheit"))
    );
  }
  return true;
}

async function convertSingle(
  value: number,
  unit: string,
  system: UnitSystem,
): Promise<string | null> {
  const toUnit = targetUnit(unit, system);
  if (!toUnit || alreadyInSystem(unit, system)) return null;
  try {
    const converted = await convertUnits({ value, fromUnit: unit, toUnit });
    if (
      converted.dimension === "unknown" ||
      converted.dimension === "same" ||
      converted.dimension === "count"
    ) {
      return null;
    }
    const formatted = await formatUnit({
      value: converted.value,
      unit: converted.unit,
      unitSystem: system,
    });
    return formatted.formatted;
  } catch {
    return null;
  }
}

/** Convert a quantity expression for display; returns the original text when unchanged. */
export async function convertQuantityForDisplay(text: string, system: UnitSystem): Promise<string> {
  const raw = text.trim();
  if (!raw) return raw;
  const parsed = parseQuantity(raw);
  if (!parsed || alreadyInSystem(parsed.unit, system)) return raw;

  if (parsed.maxValue != null && parsed.maxUnit) {
    if (!isConvertibleUnit(parsed.unit) || !isConvertibleUnit(parsed.maxUnit)) return raw;
    const low = await convertSingle(parsed.value, parsed.unit, system);
    const high = await convertSingle(parsed.maxValue, parsed.maxUnit, system);
    if (!low || !high) return raw;
    const lowParts = low.match(/^([\d./]+)\s*(.*)$/);
    const highParts = high.match(/^([\d./]+)\s*(.*)$/);
    if (!lowParts || !highParts) return raw;
    const suffix = highParts[2] || lowParts[2];
    return `${lowParts[1]} to ${highParts[1]}${suffix ? ` ${suffix}` : ""}`.trim();
  }

  const converted = await convertSingle(parsed.value, parsed.unit, system);
  return converted ?? raw;
}

/** Convert a quantity expression for source patching; always targets the requested system. */
export async function convertQuantityForSource(
  text: string,
  system: UnitSystem,
): Promise<string | null> {
  const raw = text.trim();
  if (!raw) return null;
  const parsed = parseQuantity(raw);
  if (!parsed || !isConvertibleUnit(parsed.unit)) return null;

  if (parsed.maxValue != null && parsed.maxUnit) {
    if (!isConvertibleUnit(parsed.maxUnit)) return null;
    const low = await convertSingle(parsed.value, parsed.unit, system);
    const high = await convertSingle(parsed.maxValue, parsed.maxUnit, system);
    if (!low || !high) return null;
    const lowParts = low.match(/^([\d./]+)\s*(.*)$/);
    const highParts = high.match(/^([\d./]+)\s*(.*)$/);
    if (!lowParts || !highParts) return null;
    const unit = (highParts[2] || lowParts[2]).trim().toLowerCase();
    return `${lowParts[1]} ${unit} to ${highParts[1]} ${unit}`;
  }

  const converted = await convertSingle(parsed.value, parsed.unit, system);
  if (!converted) return null;
  const parts = converted.match(/^([\d./]+)\s*(.*)$/);
  if (!parts) return null;
  const unit = parts[2]
    .replace(/°[CFK]/i, "")
    .trim()
    .toLowerCase();
  return `${parts[1]} ${unit}`.trim();
}
