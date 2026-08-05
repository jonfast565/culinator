/** Physical index-card sizes for on-screen framing and print. */
export type IndexCardFormat =
  | "full"
  | "2.5x4"
  | "4x2.5"
  | "3x5"
  | "5x3"
  | "4x6"
  | "6x4"
  | "5x8"
  | "8x5";

/** CSS px per inch — the browser's fixed conversion for `in` lengths. */
export const CSS_PX_PER_INCH = 96;

/** Typography tier keyed to card area — shared by portrait/landscape pairs. */
export type IndexCardTypeTier = "mini" | "standard" | "medium" | "large";

export interface IndexCardSpec {
  id: Exclude<IndexCardFormat, "full">;
  /** Menu / picker label */
  label: string;
  /** Compact toolbar label */
  shortLabel: string;
  /** Page width in inches. */
  widthIn: number;
  /** Page height in inches. */
  heightIn: number;
  /** CSS `@page size` value. */
  pageSize: string;
  typeTier: IndexCardTypeTier;
  orientation: "portrait" | "landscape";
}

const CARD_FORMATS: Exclude<IndexCardFormat, "full">[] = [
  "2.5x4",
  "4x2.5",
  "3x5",
  "5x3",
  "4x6",
  "6x4",
  "5x8",
  "8x5",
];

function spec(
  id: Exclude<IndexCardFormat, "full">,
  widthIn: number,
  heightIn: number,
  stock: string,
  orientation: "portrait" | "landscape",
  typeTier: IndexCardTypeTier,
): IndexCardSpec {
  const orient = orientation === "portrait" ? "portrait" : "landscape";
  return {
    id,
    label: `${stock} (${orient})`,
    shortLabel: stock,
    widthIn,
    heightIn,
    pageSize: `${widthIn}in ${heightIn}in`,
    typeTier,
    orientation,
  };
}

export const INDEX_CARD_SPECS: Record<Exclude<IndexCardFormat, "full">, IndexCardSpec> = {
  "2.5x4": spec("2.5x4", 2.5, 4, "2.5×4 in", "portrait", "mini"),
  "4x2.5": spec("4x2.5", 4, 2.5, "2.5×4 in", "landscape", "mini"),
  "3x5": spec("3x5", 3, 5, "3×5 in", "portrait", "standard"),
  "5x3": spec("5x3", 5, 3, "3×5 in", "landscape", "standard"),
  "4x6": spec("4x6", 4, 6, "4×6 in", "portrait", "medium"),
  "6x4": spec("6x4", 6, 4, "4×6 in", "landscape", "medium"),
  "5x8": spec("5x8", 5, 8, "5×8 in", "portrait", "large"),
  "8x5": spec("8x5", 8, 5, "5×8 in", "landscape", "large"),
};

export const INDEX_CARD_PICKER_OPTIONS: { id: IndexCardFormat; label: string }[] = [
  { id: "full", label: "Full page (letter)" },
  ...CARD_FORMATS.map((id) => ({ id, label: INDEX_CARD_SPECS[id].label })),
];

/** Base body font size (px) before type-scale multipliers, by card tier. */
export const INDEX_CARD_BODY_BASE_PX: Record<IndexCardTypeTier, number> = {
  mini: 9,
  standard: 9.5,
  medium: 10.5,
  large: 11,
};

/** Base title font size (px) before type-scale multipliers, by card tier. */
export const INDEX_CARD_HEADER_BASE_PX: Record<IndexCardTypeTier, number> = {
  mini: 12,
  standard: 12.5,
  medium: 14,
  large: 16,
};

export function isIndexCardFormat(value: string): value is IndexCardFormat {
  return value === "full" || CARD_FORMATS.includes(value as Exclude<IndexCardFormat, "full">);
}

export function indexCardSpec(format: IndexCardFormat): IndexCardSpec | null {
  if (format === "full") return null;
  return INDEX_CARD_SPECS[format];
}

/** View-menu action id for a card format preset. */
export function indexCardMenuAction(format: IndexCardFormat): `set-index-card:${IndexCardFormat}` {
  return `set-index-card:${format}`;
}

export function parseIndexCardMenuAction(action: string): IndexCardFormat | null {
  if (!action.startsWith("set-index-card:")) return null;
  const id = action.slice("set-index-card:".length);
  return isIndexCardFormat(id) ? id : null;
}

/** Human label for print buttons and menu hints. */
export function indexCardFormatLabel(format: IndexCardFormat): string {
  if (format === "full") return "Full page";
  return INDEX_CARD_SPECS[format].label;
}
