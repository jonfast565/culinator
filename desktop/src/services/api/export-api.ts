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
export function downloadExport(result: RecipeExportResponse): void {
  const bytes = Uint8Array.from(atob(result.archiveBase64), (char) => char.charCodeAt(0));
  const url = URL.createObjectURL(new Blob([bytes], { type: result.mediaType }));
  const link = document.createElement("a");
  link.href = url;
  link.download = result.fileName;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
