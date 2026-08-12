import type {
  UnitConvertRequest,
  UnitConvertResponse,
  UnitFormatRequest,
  UnitFormatResponse,
} from "../../domain/types";
import { hasConfiguredService, serviceRpc } from "../transport/websocket-client";

function normalizeUnit(unit: string): string {
  return unit.trim().toLowerCase().replace(/\.$/, "");
}

function convertTemperature(value: number, from: string, to: string): number | null {
  const f = normalizeUnit(from);
  const t = normalizeUnit(to);
  if (f === t) return value;
  if (f === "c" && t === "f") return (value * 9) / 5 + 32;
  if (f === "f" && t === "c") return ((value - 32) * 5) / 9;
  if (f === "c" && t === "k") return value + 273.15;
  if (f === "k" && t === "c") return value - 273.15;
  if (f === "f" && t === "k") return ((value - 32) * 5) / 9 + 273.15;
  if (f === "k" && t === "f") return ((value - 273.15) * 9) / 5 + 32;
  return null;
}

const MASS: Record<string, number> = {
  g: 1,
  gram: 1,
  grams: 1,
  kg: 1000,
  kilogram: 1000,
  kilograms: 1000,
  mg: 0.001,
  oz: 28.349523125,
  ounce: 28.349523125,
  ounces: 28.349523125,
  lb: 453.59237,
  lbs: 453.59237,
  pound: 453.59237,
  pounds: 453.59237,
};

const VOLUME: Record<string, number> = {
  ml: 1,
  milliliter: 1,
  milliliters: 1,
  l: 1000,
  liter: 1000,
  liters: 1000,
  tsp: 4.92892159375,
  teaspoon: 4.92892159375,
  teaspoons: 4.92892159375,
  tbsp: 14.78676478125,
  tablespoon: 14.78676478125,
  tablespoons: 14.78676478125,
  cup: 236.5882365,
  cups: 236.5882365,
  floz: 29.5735295625,
  pt: 473.176473,
  pint: 473.176473,
  pints: 473.176473,
  qt: 946.352946,
  quart: 946.352946,
  quarts: 946.352946,
  gal: 3785.411784,
  gallon: 3785.411784,
  gallons: 3785.411784,
  pinch: 0.3080575433945,
  pinches: 0.3080575433945,
  dash: 0.6161150867891,
  dashes: 0.6161150867891,
  drop: 0.0492892159375,
  drops: 0.0492892159375,
};

const LENGTH: Record<string, number> = {
  mm: 1,
  cm: 10,
  m: 1000,
  in: 25.4,
  inch: 25.4,
  inches: 25.4,
  ft: 304.8,
  foot: 304.8,
  feet: 304.8,
};

const TIME: Record<string, number> = {
  s: 1,
  sec: 1,
  secs: 1,
  second: 1,
  seconds: 1,
  min: 60,
  mins: 60,
  minute: 60,
  minutes: 60,
  h: 3600,
  hr: 3600,
  hrs: 3600,
  hour: 3600,
  hours: 3600,
};

function convertWithinTable(
  value: number,
  from: string,
  to: string,
  table: Record<string, number>,
  dimension: string,
): UnitConvertResponse | null {
  const fromKey = normalizeUnit(from);
  const toKey = normalizeUnit(to);
  const fromFactor = table[fromKey];
  const toFactor = table[toKey];
  if (fromFactor == null || toFactor == null) return null;
  return {
    value: (value * fromFactor) / toFactor,
    unit: to,
    dimension,
  };
}

function localConvert(request: UnitConvertRequest): UnitConvertResponse {
  const from = normalizeUnit(request.fromUnit);
  const to = normalizeUnit(request.toUnit);
  if (from === to) {
    return { value: request.value, unit: request.toUnit, dimension: "same" };
  }

  const mass = convertWithinTable(request.value, from, to, MASS, "mass");
  if (mass) return mass;
  const volume = convertWithinTable(request.value, from, to, VOLUME, "volume");
  if (volume) return volume;
  const length = convertWithinTable(request.value, from, to, LENGTH, "length");
  if (length) return length;
  const time = convertWithinTable(request.value, from, to, TIME, "time");
  if (time) return time;

  const temp = convertTemperature(request.value, from, to);
  if (temp != null) {
    return { value: temp, unit: request.toUnit, dimension: "temperature" };
  }

  // Offline volume ↔ mass for a few liquids the formula seeder needs when the
  // service isn't up. The full table lives in Rust (`IngredientDensity`).
  const cross = localVolumeMass(request);
  if (cross) return cross;

  return { value: request.value, unit: request.fromUnit, dimension: "unknown" };
}

/** Densities (g/ml) mirrored from culinator-core builtins for offline convert. */
const LOCAL_DENSITY: Record<string, number> = {
  water: 1,
  ice: 1,
  milk: 1.03,
  buttermilk: 1.03,
  oil: 0.92,
  "olive oil": 0.91,
  "vegetable oil": 0.92,
  flour: 0.59,
  "wheat flour": 0.59,
  butter: 0.911,
  honey: 1.42,
  salt: 1.22,
  sugar: 0.85,
  yeast: 0.8,
};

function localDensity(hint: string | undefined): number | null {
  if (!hint) return null;
  const key = hint.trim().toLowerCase().replaceAll("_", " ");
  if (LOCAL_DENSITY[key] != null) return LOCAL_DENSITY[key];
  const tokens = key.split(/\s+/).filter(Boolean);
  for (let start = 1; start < tokens.length; start++) {
    const slice = tokens.slice(start).join(" ");
    if (LOCAL_DENSITY[slice] != null) return LOCAL_DENSITY[slice];
  }
  for (let i = tokens.length - 1; i >= 0; i--) {
    const token = tokens[i];
    if (LOCAL_DENSITY[token] != null) return LOCAL_DENSITY[token];
  }
  return null;
}

function localVolumeMass(request: UnitConvertRequest): UnitConvertResponse | null {
  const from = normalizeUnit(request.fromUnit);
  const to = normalizeUnit(request.toUnit);
  const fromMass = MASS[from];
  const fromVol = VOLUME[from];
  const toMass = MASS[to];
  const toVol = VOLUME[to];
  const density = localDensity(request.ingredient);
  if (density == null) return null;

  if (fromVol != null && toMass != null) {
    const ml = request.value * fromVol;
    return { value: (ml * density) / toMass, unit: request.toUnit, dimension: "mass" };
  }
  if (fromMass != null && toVol != null) {
    const grams = request.value * fromMass;
    return { value: grams / density / toVol, unit: request.toUnit, dimension: "volume" };
  }
  return null;
}

function formatNumber(value: number): string {
  if (Math.abs(value - Math.round(value)) < 0.001) return String(Math.round(value));
  const rounded = Math.round(value * 100) / 100;
  return String(rounded).replace(/\.?0+$/, "");
}

function localFormat(request: UnitFormatRequest): UnitFormatResponse {
  const unit = normalizeUnit(request.unit);
  if (unit === "c" || unit === "celsius") return { formatted: `${formatNumber(request.value)} °C` };
  if (unit === "f" || unit === "fahrenheit")
    return { formatted: `${formatNumber(request.value)} °F` };
  if (unit === "k" || unit === "kelvin") return { formatted: `${formatNumber(request.value)} K` };
  return { formatted: `${formatNumber(request.value)} ${request.unit}` };
}

export async function convertUnits(request: UnitConvertRequest): Promise<UnitConvertResponse> {
  if (hasConfiguredService())
    return serviceRpc<UnitConvertResponse>("units.convert", {
      ...request,
    } as Record<string, unknown>);
  return localConvert(request);
}

export async function formatUnit(request: UnitFormatRequest): Promise<UnitFormatResponse> {
  if (hasConfiguredService())
    return serviceRpc<UnitFormatResponse>("units.format", { ...request } as Record<
      string,
      unknown
    >);
  return localFormat(request);
}
