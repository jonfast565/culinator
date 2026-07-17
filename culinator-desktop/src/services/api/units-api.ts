import type {
  UnitConvertRequest,
  UnitConvertResponse,
  UnitFormatRequest,
  UnitFormatResponse,
} from "../../domain/types";
import { hasConfiguredService, serviceRpc } from "../transport/websocket-client";

function convertTemperature(value: number, from: string, to: string): number | null {
  const f = from.toLowerCase();
  const t = to.toLowerCase();
  if (f === t) return value;
  if (f === "c" && t === "f") return (value * 9) / 5 + 32;
  if (f === "f" && t === "c") return ((value - 32) * 5) / 9;
  return null;
}

function localConvert(request: UnitConvertRequest): UnitConvertResponse {
  const from = request.fromUnit.toLowerCase();
  const to = request.toUnit.toLowerCase();
  if (from === to) {
    return { value: request.value, unit: request.toUnit, dimension: "same" };
  }
  const mass: Record<string, number> = {
    g: 1,
    kg: 1000,
    oz: 28.349523125,
    lb: 453.59237,
  };
  if (from in mass && to in mass) {
    return {
      value: (request.value * mass[from]) / mass[to],
      unit: request.toUnit,
      dimension: "mass",
    };
  }
  const temp = convertTemperature(request.value, from, to);
  if (temp != null) {
    return { value: temp, unit: request.toUnit, dimension: "temperature" };
  }
  return { value: request.value, unit: request.fromUnit, dimension: "unknown" };
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
  const rounded =
    request.unitSystem === "us_customary"
      ? `${request.value.toFixed(2)} ${request.unit}`
      : `${request.value.toFixed(1)} ${request.unit}`;
  return { formatted: rounded };
}
