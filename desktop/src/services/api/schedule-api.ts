import type { RecipeSchedule, ScheduledOperation } from "../../domain/types";
import { hasConfiguredService, serviceRpc } from "../transport/websocket-client";
import { parseUiModel } from "../../features/recipe-editor/model";

export async function scheduleRecipe(sourceText: string): Promise<RecipeSchedule> {
  if (hasConfiguredService()) return serviceRpc("recipes.schedule", { sourceText, options: {} });
  const model = parseUiModel(sourceText);
  const completed = new Map<string, ScheduledOperation>();
  const pending = [...model.operations];
  while (pending.length) {
    const index = pending.findIndex((operation) =>
      operation.after.every((item) => completed.has(item)),
    );
    if (index < 0) throw new Error("Dependency graph contains a cycle");
    const operation = pending.splice(index, 1)[0];
    const startSeconds = Math.max(
      0,
      ...operation.after.map((item) => completed.get(item)?.endSeconds ?? 0),
    );
    const durationSeconds = Math.max(1, Math.round(operation.durationMinutes * 60));
    completed.set(operation.symbol, {
      symbol: operation.symbol,
      process: operation.process,
      action: operation.action,
      startSeconds,
      endSeconds: startSeconds + durationSeconds,
      durationSeconds,
      labor: operation.labor,
      dependencies: operation.after,
      resources: [],
    });
  }
  const operations = [...completed.values()].sort((a, b) => a.startSeconds - b.startSeconds);
  const makespanSeconds = Math.max(0, ...operations.map((item) => item.endSeconds));
  return {
    operations,
    makespanSeconds,
    criticalPath: operations
      .filter((item) => item.endSeconds === makespanSeconds)
      .map((item) => item.symbol),
  };
}
