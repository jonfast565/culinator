import type { InitReport } from "../../domain/types";
import { hasConfiguredService, serviceRpc } from "../transport/websocket-client";

export type InitPhase =
  | "connecting"
  | "initializing_catalog"
  | "loading_nutrition"
  | "seeding_samples"
  | "ready"
  | "failed";

export async function fetchInitStatus(): Promise<InitReport> {
  if (!hasConfiguredService()) {
    return {
      catalogReady: true,
      recipesSeeded: true,
      nutritionReady: false,
      nutritionStarter: false,
      recipeCount: 0,
    };
  }
  return serviceRpc<InitReport>("service.status", {} as Record<string, unknown>);
}

export async function runInitialization(): Promise<InitReport> {
  if (!hasConfiguredService()) {
    return fetchInitStatus();
  }
  return serviceRpc<InitReport>("service.initialize", {} as Record<string, unknown>);
}
