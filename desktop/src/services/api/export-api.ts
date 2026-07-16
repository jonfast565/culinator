import type { RecipeExportOptions, RecipeExportResponse } from "../../domain/types";
import { hasConfiguredService, serviceRpc } from "../transport/websocket-client";
export async function exportRecipe(
  recipeId: string,
  options: RecipeExportOptions,
): Promise<RecipeExportResponse> {
  if (!hasConfiguredService())
    throw new Error("Recipe export requires the local Culinograph service");
  return serviceRpc<RecipeExportResponse>("recipes.export", { id: recipeId, options });
}

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function fileExtension(fileName: string): string | undefined {
  const dot = fileName.lastIndexOf(".");
  return dot > 0 ? fileName.slice(dot + 1).toLowerCase() : undefined;
}

/**
 * Persist an export bundle to disk. Inside the Tauri desktop shell this opens a
 * native "Save As" dialog and writes the chosen path; a bare `<a download>` is
 * silently ignored by the WKWebView, so it is only used as a browser fallback.
 * Returns `false` when the user cancels the save dialog.
 */
export async function downloadExport(result: RecipeExportResponse): Promise<boolean> {
  if (isTauri()) return saveExportViaTauri(result);
  saveExportViaBrowser(result);
  return true;
}

async function saveExportViaTauri(result: RecipeExportResponse): Promise<boolean> {
  const { save } = await import("@tauri-apps/plugin-dialog");
  const { invoke } = await import("@tauri-apps/api/core");
  const extension = fileExtension(result.fileName);
  const path = await save({
    defaultPath: result.fileName,
    filters: extension
      ? [{ name: `${extension.toUpperCase()} file`, extensions: [extension] }]
      : undefined,
  });
  if (!path) return false;
  await invoke("save_export", { path, contentsBase64: result.archiveBase64 });
  return true;
}

function saveExportViaBrowser(result: RecipeExportResponse): void {
  const bytes = Uint8Array.from(atob(result.archiveBase64), (char) => char.charCodeAt(0));
  const url = URL.createObjectURL(new Blob([bytes], { type: result.mediaType }));
  const link = document.createElement("a");
  link.href = url;
  link.download = result.fileName;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
