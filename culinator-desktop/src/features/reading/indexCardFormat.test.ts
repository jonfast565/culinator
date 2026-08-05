import { describe, expect, it } from "vitest";
import {
  INDEX_CARD_PICKER_OPTIONS,
  INDEX_CARD_SPECS,
  indexCardFormatLabel,
  indexCardMenuAction,
  indexCardSpec,
  isIndexCardFormat,
  parseIndexCardMenuAction,
} from "./indexCardFormat";

describe("indexCardFormat", () => {
  it("lists full page plus portrait and landscape presets for each stock size", () => {
    expect(INDEX_CARD_PICKER_OPTIONS[0]).toEqual({ id: "full", label: "Full page (letter)" });
    expect(INDEX_CARD_PICKER_OPTIONS).toHaveLength(9);
    expect(INDEX_CARD_PICKER_OPTIONS.map((option) => option.id)).toEqual([
      "full",
      "2.5x4",
      "4x2.5",
      "3x5",
      "5x3",
      "4x6",
      "6x4",
      "5x8",
      "8x5",
    ]);
  });

  it("uses page dimensions as ids with orientation in the label", () => {
    expect(INDEX_CARD_SPECS["6x4"].label).toBe("4×6 in (landscape)");
    expect(INDEX_CARD_SPECS["4x6"].label).toBe("4×6 in (portrait)");
    expect(INDEX_CARD_SPECS["6x4"].widthIn).toBe(6);
    expect(INDEX_CARD_SPECS["6x4"].heightIn).toBe(4);
    expect(INDEX_CARD_SPECS["4x6"].widthIn).toBe(4);
    expect(INDEX_CARD_SPECS["4x6"].heightIn).toBe(6);
  });

  it("shares typography tiers within portrait/landscape pairs", () => {
    expect(INDEX_CARD_SPECS["3x5"].typeTier).toBe(INDEX_CARD_SPECS["5x3"].typeTier);
    expect(INDEX_CARD_SPECS["4x6"].typeTier).toBe(INDEX_CARD_SPECS["6x4"].typeTier);
  });

  it("validates format ids", () => {
    expect(isIndexCardFormat("full")).toBe(true);
    expect(isIndexCardFormat("6x4")).toBe(true);
    expect(isIndexCardFormat("8x10")).toBe(false);
  });

  it("parses view-menu actions", () => {
    expect(parseIndexCardMenuAction(indexCardMenuAction("5x3"))).toBe("5x3");
    expect(parseIndexCardMenuAction("set-index-card:full")).toBe("full");
    expect(parseIndexCardMenuAction("toggle-mise")).toBeNull();
  });

  it("formats labels for print chrome", () => {
    expect(indexCardFormatLabel("full")).toBe("Full page");
    expect(indexCardFormatLabel("2.5x4")).toBe("2.5×4 in (portrait)");
    expect(indexCardSpec("full")).toBeNull();
    expect(indexCardSpec("3x5")?.pageSize).toBe("3in 5in");
  });
});
