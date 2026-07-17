import type {
  ImportDraft,
  PublicImportSettings,
  RecipeImageInput,
  RecipeImportResult,
  StructuredInput,
} from "../../domain/types";
import { hasConfiguredService, serviceRpc } from "../transport/websocket-client";

export const getImportSettings = () => serviceRpc<PublicImportSettings>("imports.settings.get");
export const updateImportSettings = (value: {
  openaiApiKey?: string;
  openaiModel: string;
  useLocalOcr: boolean;
  tesseractCommand: string;
}) => serviceRpc<PublicImportSettings>("imports.settings.update", value);

export const clearStoredApiKey = () =>
  serviceRpc<PublicImportSettings>("imports.settings.update", {
    openaiApiKey: "",
    openaiModel: "gpt-4.1-mini",
    useLocalOcr: true,
    tesseractCommand: "tesseract",
  });

export const translateRecipeImages = (images: RecipeImageInput[], targetLanguage?: string) =>
  serviceRpc<RecipeImportResult>("imports.translate", { images, targetLanguage });

export async function importStructured(input: StructuredInput): Promise<ImportDraft> {
  if (!hasConfiguredService()) throw new Error("Structured import requires the Culinator service");
  return serviceRpc<ImportDraft>("imports.structured", { ...input } as Record<string, unknown>);
}
