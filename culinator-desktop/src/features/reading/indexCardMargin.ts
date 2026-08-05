/** Adjustable page / card margins for index-card view and print. */
export type IndexCardMargin = "tight" | "medium" | "wide";

/** Compact chrome control labels (Tit/Body/Notes style). */
export const INDEX_CARD_MARGIN_LABELS: Record<IndexCardMargin, string> = {
  tight: "T",
  medium: "M",
  wide: "W",
};

/** Menu / tooltip wording. */
export const INDEX_CARD_MARGIN_NAMES: Record<IndexCardMargin, string> = {
  tight: "Tight",
  medium: "Medium",
  wide: "Wide",
};

/**
 * Printer `@page` margin (and matching on-screen inset).
 *
 * Defaults target printability + readability:
 * - Most consumer printers clip inside ~0.25in; keep even Tight near that floor.
 * - Medium (default) adds a readable gutter without starving small card stock.
 * - Full-page letter uses larger insets so body text doesn’t hug the paper edge.
 */
export const INDEX_CARD_MARGIN_IN: Record<
  IndexCardMargin,
  { card: number; full: number }
> = {
  /** Dense pack; still clears typical non-printable regions. */
  tight: { card: 0.2, full: 0.4 },
  /** Default — safe to print and comfortable to read. */
  medium: { card: 0.3, full: 0.6 },
  /** Airy / binder-friendly. */
  wide: { card: 0.45, full: 0.85 },
};

/** Default margin preset for new sessions (printability + readability). */
export const DEFAULT_INDEX_CARD_MARGIN: IndexCardMargin = "medium";

export function isIndexCardMargin(value: string): value is IndexCardMargin {
  return value === "tight" || value === "medium" || value === "wide";
}

export function cycleIndexCardMargin(current: IndexCardMargin): IndexCardMargin {
  return current === "tight" ? "medium" : current === "medium" ? "wide" : "tight";
}

/** CSS length for the injected `@page { margin }` rule. */
export function indexCardPrintMarginCss(format: "full" | string, margin: IndexCardMargin): string {
  const inches =
    format === "full" ? INDEX_CARD_MARGIN_IN[margin].full : INDEX_CARD_MARGIN_IN[margin].card;
  return `${inches}in`;
}
