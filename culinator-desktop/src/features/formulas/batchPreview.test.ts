import { beforeAll, describe, expect, it } from "vitest";
import { parseUiModel } from "../recipe-editor/model";
import { batchApplyWasm, batchPreviewWasm } from "../../services/wasm/parser";
import { loadParser, seed } from "../recipe-builder/test-support";
import type { BatchRequest } from "../../domain/types";

beforeAll(loadParser);

describe("shared formula batch planner", () => {
  it("previews and applies the same complete source through WASM", () => {
    const sourceText = seed("pizza_dough.cg");
    const model = parseUiModel(sourceText);
    const raw = model.formulas?.[0]?.raw;
    expect(raw).toBeDefined();
    const request: BatchRequest = {
      sourceText,
      formulaSymbol: "dough",
      formula: raw!,
      constraint: { kind: "target_mass", grams: 1256 },
      applyMinimums: true,
    };
    const preview = batchPreviewWasm(request);
    expect(preview.blockers).toEqual([]);
    expect(preview.proposedSource).toContain("input flour 800 g;");
    expect(preview.proposedSource).toContain("quantity 800 g;");
    expect(batchApplyWasm(request, preview.sourceFingerprint).proposedSource).toBe(
      preview.proposedSource,
    );
    expect(() => batchApplyWasm(request, "stale")).toThrow("changed after the batch preview");
  });
});
