import type { Diagnostic, ValidationResult } from "../../domain/types";
import { hasConfiguredService, serviceRequest } from "../transport/websocket-client";

export async function validateRecipe(sourceText: string): Promise<ValidationResult> {
  if (hasConfiguredService()) {
    return serviceRequest("/api/v1/validation", {
      method: "POST",
      body: JSON.stringify({ sourceText }),
    });
  }
  const diagnostics: Diagnostic[] = [];
  if (!sourceText.trimStart().startsWith("culinograph "))
    diagnostics.push({ severity: "error", message: "Missing protocol header." });
  if ((sourceText.match(/{/g) ?? []).length !== (sourceText.match(/}/g) ?? []).length)
    diagnostics.push({ severity: "error", message: "Unbalanced braces." });
  const symbol = sourceText.match(/recipe\s+([A-Za-z_][\w]*)/)?.[1] ?? "unknown";
  return {
    valid: diagnostics.length === 0,
    diagnostics,
    outline: {
      title: sourceText.match(/title\s+"([^"]+)"/)?.[1] ?? symbol.replaceAll("_", " "),
      symbol,
      protocolVersion: sourceText.match(/culinograph\s+([^;]+);/)?.[1]?.trim() ?? "0.3",
      typeCount: (sourceText.match(/\btype\s+/g) ?? []).length,
      resourceCount: (
        sourceText.match(
          /\b(?:ingredient|material|container|equipment|environment|labor|resource)\s+/g,
        ) ?? []
      ).length,
      processCount: (sourceText.match(/\bprocess\s+/g) ?? []).length,
      operationCount: (sourceText.match(/\boperation\s+/g) ?? []).length,
      servingCount: (sourceText.match(/\bserving\s+/g) ?? []).length,
    },
  };
}
