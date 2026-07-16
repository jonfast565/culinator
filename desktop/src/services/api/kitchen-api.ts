import type { RecipeTryDocument, RecipeTrySummary } from "../../domain/types";
import { hasConfiguredService, serviceRequest, serviceRpc } from "../transport/websocket-client";

const browserTries = new Map<string, RecipeTryDocument[]>();

export async function listRecipeTries(recipeId: string): Promise<RecipeTrySummary[]> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/recipes/${encodeURIComponent(recipeId)}/tries`);
  return (browserTries.get(recipeId) ?? []).map((item) => ({
    id: item.id,
    recipeId: item.recipeId,
    title: item.title,
    status: item.status,
    startedAt: item.startedAt,
    completedAt: item.completedAt,
    operationCount: item.operations.length,
    observationCount: item.observations.length,
  }));
}

export async function getRecipeTry(tryId: string): Promise<RecipeTryDocument> {
  if (hasConfiguredService()) return serviceRequest(`/api/v1/tries/${encodeURIComponent(tryId)}`);
  for (const tries of browserTries.values()) {
    const found = tries.find((item) => item.id === tryId);
    if (found) return structuredClone(found);
  }
  throw new Error("Recipe try not found");
}

export async function startRecipeTry(
  recipeId: string,
  input: { title?: string; notes?: string },
): Promise<RecipeTryDocument> {
  if (hasConfiguredService()) return serviceRpc("tries.start", { recipeId, ...input });
  const tryDoc: RecipeTryDocument = {
    id: crypto.randomUUID(),
    recipeId,
    title: input.title ?? "Kitchen try",
    status: "active",
    scaleFactor: 1,
    startedAt: new Date().toISOString(),
    notes: input.notes,
    findings: "",
    operations: [],
    observations: [],
  };
  const tries = browserTries.get(recipeId) ?? [];
  tries.push(tryDoc);
  browserTries.set(recipeId, tries);
  return structuredClone(tryDoc);
}

export async function updateRecipeTry(
  tryId: string,
  input: {
    status?: RecipeTryDocument["status"];
    title?: string;
    notes?: string;
    findings?: string;
  },
): Promise<RecipeTryDocument> {
  if (hasConfiguredService()) return serviceRpc("tries.update", { tryId, ...input });
  for (const tries of browserTries.values()) {
    const item = tries.find((candidate) => candidate.id === tryId);
    if (!item) continue;
    Object.assign(item, input);
    if (input.status === "completed" || input.status === "abandoned") {
      item.completedAt = new Date().toISOString();
    }
    return structuredClone(item);
  }
  throw new Error("Recipe try not found");
}

export async function updateTryOperation(
  tryId: string,
  operationId: string,
  input: { status?: RecipeTryDocument["operations"][number]["status"]; notes?: string },
): Promise<RecipeTryDocument> {
  if (hasConfiguredService())
    return serviceRpc("tries.updateOperation", { tryId, operationId, ...input });
  const item = await getRecipeTry(tryId);
  const operation = item.operations.find((candidate) => candidate.operationId === operationId);
  if (!operation) throw new Error("Operation not found");
  Object.assign(operation, input);
  if (input.status === "active") operation.actualStart = new Date().toISOString();
  if (input.status === "completed" || input.status === "skipped") {
    operation.actualEnd = new Date().toISOString();
  }
  return item;
}

export async function addTryObservation(
  tryId: string,
  input: {
    operationSymbol?: string;
    propertyPath: string;
    value: string | number | boolean | null;
    unit?: string;
    notes?: string;
  },
): Promise<RecipeTryDocument> {
  if (hasConfiguredService()) return serviceRpc("tries.observe", { tryId, ...input });
  const item = await getRecipeTry(tryId);
  item.observations.unshift({
    id: crypto.randomUUID(),
    operationSymbol: input.operationSymbol,
    observedAt: new Date().toISOString(),
    propertyPath: input.propertyPath,
    value: input.value,
    unit: input.unit,
    notes: input.notes,
  });
  return item;
}

export async function deleteRecipeTry(tryId: string): Promise<void> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/tries/${encodeURIComponent(tryId)}`, { method: "DELETE" });
  for (const [recipeId, tries] of browserTries.entries()) {
    const next = tries.filter((item) => item.id !== tryId);
    if (next.length !== tries.length) {
      browserTries.set(recipeId, next);
      return;
    }
  }
}
