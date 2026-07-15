import type {
  PublicImportSettings,
  RecipeImageInput,
  RecipeImportResult,
} from "../../domain/types";
import { serviceRpc } from "../transport/websocket-client";
export const getImportSettings = () => serviceRpc<PublicImportSettings>("imports.settings.get");
export const updateImportSettings = (value: {
  openaiApiKey?: string;
  openaiModel: string;
  useLocalOcr: boolean;
  tesseractCommand: string;
}) => serviceRpc<PublicImportSettings>("imports.settings.update", value);
export const translateRecipeImages = (images: RecipeImageInput[], targetLanguage?: string) =>
  serviceRpc<RecipeImportResult>("imports.translate", { images, targetLanguage });
