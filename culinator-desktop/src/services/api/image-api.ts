/* global crypto, FileReader, File */
import type {
  RecipeImageAsset,
  RecipeImageData,
  UploadRecipeImageRequest,
} from "../../domain/types";
import { hasConfiguredService, serviceRpc } from "../transport/websocket-client";

// Recipe images are stored server-side (recipe_images table) when a service is
// configured, and in localStorage otherwise so the offline/demo app persists
// uploads across reloads. External-URL images never reach here — the reading
// view renders those straight from the `.cg` handle.

const STORE_KEY = "culinator.demo.images";
const keyOf = (recipeId: string, handle: string): string => `${recipeId}::${handle}`;

function readStore(): Record<string, RecipeImageData> {
  try {
    return JSON.parse(window.localStorage.getItem(STORE_KEY) || "{}") as Record<
      string,
      RecipeImageData
    >;
  } catch {
    return {};
  }
}
function writeStore(store: Record<string, RecipeImageData>): void {
  try {
    window.localStorage.setItem(STORE_KEY, JSON.stringify(store));
  } catch {
    // localStorage quota exceeded — degrade gracefully rather than throw so the
    // rest of the save still succeeds; the image just won't persist offline.
    console.warn("Could not persist image to localStorage (quota?). It will not survive reload.");
  }
}

function randomHandle(role: string): string {
  const hex = Math.random().toString(16).slice(2, 10);
  return `img_${role}_${hex}`;
}

// Read a picked file into the bare base64 payload (no data-URL prefix) that the
// upload API expects. Shared by every image upload control (cover, per-step).
export function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result).split(",")[1] ?? "");
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

export async function listRecipeImages(recipeId: string): Promise<RecipeImageAsset[]> {
  if (hasConfiguredService()) return serviceRpc("images.list", { recipeId });
  return Object.values(readStore())
    .filter((entry) => entry.asset.recipeId === recipeId)
    .map((entry) => entry.asset);
}

export async function getRecipeImage(
  recipeId: string,
  handle: string,
): Promise<RecipeImageData | null> {
  if (hasConfiguredService()) {
    try {
      return await serviceRpc<RecipeImageData>("images.get", { recipeId, handle });
    } catch {
      return null;
    }
  }
  return readStore()[keyOf(recipeId, handle)] ?? null;
}

export async function uploadRecipeImage(
  recipeId: string,
  input: UploadRecipeImageRequest,
): Promise<RecipeImageAsset> {
  const handle = input.handle || randomHandle(input.role);
  if (hasConfiguredService()) return serviceRpc("images.upload", { recipeId, ...input, handle });

  const byteSize = Math.floor((input.dataBase64.length * 3) / 4);
  const asset: RecipeImageAsset = {
    id: typeof crypto !== "undefined" ? crypto.randomUUID() : handle,
    recipeId,
    handle,
    role: input.role,
    operationSymbol: input.operationSymbol ?? null,
    mediaType: input.mediaType,
    fileName: input.fileName ?? null,
    byteSize,
    createdAt: new Date().toISOString(),
  };
  const store = readStore();
  store[keyOf(recipeId, handle)] = { asset, dataBase64: input.dataBase64 };
  writeStore(store);
  return asset;
}

export async function deleteRecipeImage(recipeId: string, handle: string): Promise<void> {
  if (hasConfiguredService()) return serviceRpc("images.delete", { recipeId, handle });
  const store = readStore();
  delete store[keyOf(recipeId, handle)];
  writeStore(store);
}
