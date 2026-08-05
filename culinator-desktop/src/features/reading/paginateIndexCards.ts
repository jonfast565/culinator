import type { IndexCardFormat, IndexCardTypeTier } from "./indexCardFormat";
import {
  INDEX_CARD_BODY_BASE_PX,
  INDEX_CARD_HEADER_BASE_PX,
  INDEX_CARD_SPECS,
} from "./indexCardFormat";
import type { IndexCardMargin } from "./indexCardMargin";
import { DEFAULT_INDEX_CARD_MARGIN, INDEX_CARD_MARGIN_IN } from "./indexCardMargin";
import type { IndexCardMeasurements } from "./measureIndexCard";
import { indexCardBlockKey } from "./measureIndexCard";
import type { RecipeTypeScale } from "./recipeTypeScale";
import { RECIPE_TYPE_SCALE_FACTORS } from "./recipeTypeScale";
import type {
  IngredientDisplayParts,
  IngredientGroup,
  MethodSection,
  NarrativeStep,
  SectionMise,
} from "../recipe-editor/narrative";

export type IndexCardMiseLayout = "top-matter" | "colocated";

export interface IndexCardMethodBlock {
  title?: string;
  note?: string;
  /** Present in colocated (mise-by-section) layout. */
  mise?: SectionMise;
  steps: NarrativeStep[];
}

export interface IndexCardPage {
  /** 0-based index within the deck. */
  index: number;
  /** Total cards in the deck. */
  total: number;
  /** Full title block (first card only). */
  showTitle: boolean;
  /** Short “Title (continued)” line instead of the full header. */
  continuation: boolean;
  ingredientGroups: IngredientGroup[];
  equipment: string[];
  methodBlocks: IndexCardMethodBlock[];
}

/** CSS px per inch — matches browser default for `in` lengths. */
const DPI = 96;

/** Space reserved for the “Card N of M” footer inside the leaf. */
const PAGER_RESERVE_PX = 14;

/**
 * Ingredient height estimates include pessimistic wrap/chrome; only a sliver
 * of slack is needed at pack time (see PAGE_HEIGHT_SLACK in tests).
 */
const INGREDIENT_HEIGHT_SLACK_PX = 2;

/** Method steps — same conservative headroom so prose is not clipped. */
const METHOD_PACK_SAFETY = 0.88;

/** Matches index-card-view.css ingredient / section-label typography. */
const INGREDIENT_FONT_PX = 10;
const SECTION_LABEL_FONT_PX = 9.5;

/** Method step grid — index-card-view.css `.step` / `.step-text`. */
const STEP_NUMBER_COLUMN_PX = 1.05 * 16;
const STEP_GRID_GAP_PX = 4;
const STEP_LINE_HEIGHT = 1.2;
const STEP_GAP_PX = 2;
const PROCESS_HEADING_CHROME_PX = 4;
const STEP_TIME_CHROME_PX = 18;

/** Cover art on the title card, when the recipe has one. */
const COVER_ESTIMATE_PX = 36;

/** Leaf title font multipliers — index-card-view.css uses tier-specific sizes. */
const TITLE_FONT_PX: Record<IndexCardTypeTier, number> = {
  mini: 12,
  standard: 13.5,
  medium: 15,
  large: 17,
};

/** Ingredient grid: `minmax(0, 3rem)` qty column + gap (px at 16px root). */
const QTY_COLUMN_PX = 48;
const INGREDIENT_GRID_GAP_PX = 4;

/** Line-height + row padding/border in index-card-view.css. */
const INGREDIENT_LINE_HEIGHT = 1.35;
/** 1px padding top/bottom + 1px dashed border — matches `.ingredient-row`. */
const INGREDIENT_ROW_CHROME_PX = 3;

/** Vertical gap between leaf sections (matches index-card-view.css). */
const SECTION_GAP_PX = 8;
/** `.leaf-section:first-of-type { margin-top: 6px }` after the title block. */
const FIRST_SECTION_GAP_PX = 6;

/** Base body font size (px) before type-scale multipliers, by card tier. */
const BODY_BASE_PX = INDEX_CARD_BODY_BASE_PX;

/** Base title font size (px) before type-scale multipliers, by card tier. */
const HEADER_BASE_PX = INDEX_CARD_HEADER_BASE_PX;

interface PackMetrics {
  width: number;
  height: number;
  body: number;
  header: number;
  /** Leaf title font size (tier-specific × header scale). */
  title: number;
  note: number;
  /** Ingredient row font size (10 × body scale) — smaller than leaf body base. */
  ingredientBody: number;
  /** Section-label font size (9.5 × header scale). */
  sectionLabel: number;
  charsPerLine: number;
}

/**
 * Usable content box after margins — shared by packer and tests.
 *
 * With `measure`, the box comes from the probe card's own content box (so the
 * frame border and the real pager height are accounted for); without it, from
 * the nominal card geometry.
 */
export function cardContentBox(
  format: Exclude<IndexCardFormat, "full">,
  margin: IndexCardMargin = DEFAULT_INDEX_CARD_MARGIN,
  scales: {
    header?: RecipeTypeScale;
    body?: RecipeTypeScale;
    annotation?: RecipeTypeScale;
  } = {},
  showPager = true,
  measure?: IndexCardMeasurements,
): PackMetrics {
  const spec = INDEX_CARD_SPECS[format];
  const marginIn = INDEX_CARD_MARGIN_IN[margin].card;
  const pagerReserve = showPager ? (measure?.pagerReserve ?? PAGER_RESERVE_PX) : 0;
  const width = measure
    ? Math.max(40, measure.contentWidth)
    : Math.max(40, (spec.widthIn - 2 * marginIn) * DPI);
  const height = measure
    ? Math.max(40, measure.contentHeight - pagerReserve)
    : Math.max(40, (spec.heightIn - 2 * marginIn) * DPI - pagerReserve);
  const bodyScale = RECIPE_TYPE_SCALE_FACTORS[scales.body ?? "md"];
  const headerScale = RECIPE_TYPE_SCALE_FACTORS[scales.header ?? "md"];
  const body = BODY_BASE_PX[spec.typeTier] * bodyScale;
  const header = HEADER_BASE_PX[spec.typeTier] * headerScale;
  const title = TITLE_FONT_PX[spec.typeTier] * headerScale;
  const note = 7 * RECIPE_TYPE_SCALE_FACTORS[scales.annotation ?? "md"];
  const ingredientBody = INGREDIENT_FONT_PX * bodyScale;
  const sectionLabel = SECTION_LABEL_FONT_PX * headerScale;
  // Slightly pessimistic glyph width so wrapping is over-estimated, not under.
  const charsPerLine = Math.max(12, Math.floor(width / (body * 0.55)));
  return { width, height, body, header, title, note, ingredientBody, sectionLabel, charsPerLine };
}

function linesFor(text: string, charsPerLine: number): number {
  if (!text) return 1;
  const words = text.split(/\s+/);
  let lines = 1;
  let col = 0;
  for (const word of words) {
    const need = word.length + (col > 0 ? 1 : 0);
    if (col + need > charsPerLine && col > 0) {
      lines += 1;
      col = word.length;
    } else {
      col += need;
    }
  }
  return lines;
}

/**
 * Vertical cost of every block a card can hold, in CSS px.
 *
 * `heuristicHeights` estimates them from font metrics — the only option where
 * there is no layout engine (vitest, first paint). `measuredHeights` reads the
 * probe card instead, which is why it needs no safety multipliers.
 */
interface HeightModel {
  metrics: PackMetrics;
  /** Fraction of the remaining room the method may claim. 1 when measured. */
  methodSafety: number;
  /** Px held back when packing ingredients. 0 when measured. */
  ingredientSlack: number;
  /** Charged between method children after the first. */
  stepsGap: number;
  /** Cover art plus the full header, when the card shows the title. */
  titleBlock(): number;
  continuationHeader(): number;
  sectionTopGap(first: boolean): number;
  sectionLabel(text: string): number;
  groupLabel(text: string): number;
  ingredientRow(item: IngredientDisplayParts): number;
  equipmentList(items: string[]): number;
  processHeading(text: string): number;
  miseBlock(mise: SectionMise): number;
  sectionNote(text: string): number;
  stepRow(step: NarrativeStep): number;
}

function ingredientDescChars(m: PackMetrics): number {
  const descWidth = Math.max(40, m.width - QTY_COLUMN_PX - INGREDIENT_GRID_GAP_PX);
  // Slightly pessimistic glyph width — cards use `overflow-wrap: anywhere`.
  return Math.max(8, Math.floor(descWidth / (m.ingredientBody * 0.52)));
}

function qtyLines(m: PackMetrics, amount: string): number {
  const qtyChars = Math.max(4, Math.floor(QTY_COLUMN_PX / (m.ingredientBody * 0.55)));
  return linesFor(amount || "1", qtyChars);
}

function estimateIngredientRow(m: PackMetrics, item: IngredientDisplayParts): number {
  const text = item.aside ? `${item.description} ${item.aside}` : item.description;
  const descLines = linesFor(text, ingredientDescChars(m));
  const lines = Math.max(qtyLines(m, item.amount), descLines);
  const lineH = m.ingredientBody * INGREDIENT_LINE_HEIGHT;
  return Math.max(lineH + INGREDIENT_ROW_CHROME_PX, lines * lineH + INGREDIENT_ROW_CHROME_PX);
}

function estimateMiseBlock(m: PackMetrics, mise: SectionMise): number {
  if (!mise.ingredients.length && !mise.equipment.length) return 0;
  let h = 14; // padding + border
  if (mise.ingredients.length) {
    h += m.note * 1.2 + 4;
    for (const item of mise.ingredients) {
      h += estimateIngredientRow(m, item);
    }
  }
  if (mise.equipment.length) {
    h += m.note * 1.2 + 4;
    h += mise.equipment.length * (m.body * 1.28 + 1);
  }
  return h + 4;
}

/** Width-aware step prose wrap — number column + gap reserved. */
function stepTextCharsPerLine(m: PackMetrics): number {
  const textWidth = Math.max(40, m.width - STEP_NUMBER_COLUMN_PX - STEP_GRID_GAP_PX);
  return Math.max(8, Math.floor(textWidth / (m.ingredientBody * 0.48)));
}

function estimateStepRow(m: PackMetrics, text: string, hasTime: boolean): number {
  const lines = linesFor(text, stepTextCharsPerLine(m));
  const lineH = m.ingredientBody * STEP_LINE_HEIGHT;
  let h = Math.max(lineH, lines * lineH);
  if (hasTime) h = Math.max(h, STEP_TIME_CHROME_PX);
  return h + STEP_GAP_PX;
}

function estimateEquipmentList(m: PackMetrics, items: string[]): number {
  if (!items.length) return 0;
  const cols = m.width < 280 ? 1 : 2;
  const perCol = Math.ceil(items.length / cols);
  return perCol * (m.body * 1.28 + 1) + 4;
}

function heuristicHeights(m: PackMetrics, hasCover: boolean): HeightModel {
  return {
    metrics: m,
    methodSafety: METHOD_PACK_SAFETY,
    ingredientSlack: INGREDIENT_HEIGHT_SLACK_PX,
    // The inter-step gap is folded into `stepRow`, matching the estimates these
    // constants were tuned against.
    stepsGap: 0,
    titleBlock: () =>
      // eyebrow + title (~2 lines) + summary + hairline
      (hasCover ? COVER_ESTIMATE_PX : 0) + m.note * 1.3 + m.title * 1.18 * 2 + m.note * 1.25 + 10,
    continuationHeader: () => m.header * 0.9 + m.note + 10,
    sectionTopGap: (first) => (first ? FIRST_SECTION_GAP_PX : SECTION_GAP_PX),
    sectionLabel: () => m.sectionLabel * 1.2 + 3,
    groupLabel: () => m.note * 1.2 + 3,
    ingredientRow: (item) => estimateIngredientRow(m, item),
    equipmentList: (items) => estimateEquipmentList(m, items),
    processHeading: () => m.sectionLabel * 1.2 + PROCESS_HEADING_CHROME_PX,
    miseBlock: (mise) => estimateMiseBlock(m, mise),
    sectionNote: (text) => linesFor(text, m.charsPerLine) * m.note * 1.25 + 2,
    stepRow: (step) => estimateStepRow(m, step.text, Boolean(step.time)),
  };
}

function measuredHeights(
  m: PackMetrics,
  hasCover: boolean,
  measure: IndexCardMeasurements,
): HeightModel {
  const estimate = heuristicHeights(m, hasCover);
  /** Measured height, or the estimate for a block the probe never rendered. */
  const at = (key: string, fallback: number): number => measure.heights.get(key) ?? fallback;
  return {
    metrics: m,
    methodSafety: 1,
    ingredientSlack: 0,
    stepsGap: measure.stepsGap,
    titleBlock: () =>
      at(indexCardBlockKey.titleHead, estimate.titleBlock()) +
      (hasCover ? at(indexCardBlockKey.titleCover, COVER_ESTIMATE_PX) : 0),
    continuationHeader: () => at(indexCardBlockKey.continuationHead, estimate.continuationHeader()),
    sectionTopGap: (first) => (first ? measure.firstSectionGap : measure.sectionGap),
    sectionLabel: (text) => at(indexCardBlockKey.sectionLabel(text), estimate.sectionLabel(text)),
    groupLabel: (text) => at(indexCardBlockKey.groupLabel(text), estimate.groupLabel(text)),
    ingredientRow: (item) =>
      at(indexCardBlockKey.ingredientRow(item), estimate.ingredientRow(item)),
    equipmentList: (items) =>
      items.length ? at(indexCardBlockKey.equipmentList(items), estimate.equipmentList(items)) : 0,
    processHeading: (text) =>
      at(indexCardBlockKey.processHeading(text), estimate.processHeading(text)),
    miseBlock: (mise) =>
      mise.ingredients.length || mise.equipment.length
        ? at(indexCardBlockKey.mise(mise), estimate.miseBlock(mise))
        : 0,
    sectionNote: (text) => at(indexCardBlockKey.sectionNote(text), estimate.sectionNote(text)),
    stepRow: (step) => at(indexCardBlockKey.step(step), estimate.stepRow(step)),
  };
}

interface FlatStep {
  process: string;
  title?: string;
  note?: string;
  step: NarrativeStep;
  leadIn: boolean;
}

function flattenSteps(sections: MethodSection[]): FlatStep[] {
  const flat: FlatStep[] = [];
  for (const section of sections) {
    section.steps.forEach((step, index) => {
      flat.push({
        process: section.process,
        title: section.title,
        note: section.note,
        step,
        leadIn: index === 0,
      });
    });
  }
  return flat;
}

function miseMap(sections: MethodSection[]): Map<string, SectionMise> {
  return new Map(sections.map((section) => [section.process, section.mise]));
}

function countItems(groups: IngredientGroup[]): number {
  return groups.reduce((sum, group) => sum + group.items.length, 0);
}

/** Split ingredient groups into slices of at most `limit` items, preserving labels. */
export function chunkIngredientGroups(
  groups: IngredientGroup[],
  limit: number,
): IngredientGroup[][] {
  if (limit < 1) return groups.length ? [groups] : [];
  const slices: IngredientGroup[][] = [];
  let current: IngredientGroup[] = [];
  let count = 0;

  const pushCurrent = () => {
    if (current.length) {
      slices.push(current);
      current = [];
      count = 0;
    }
  };

  for (const group of groups) {
    let offset = 0;
    while (offset < group.items.length) {
      const room = limit - count;
      if (room <= 0) {
        pushCurrent();
        continue;
      }
      const take = Math.min(room, group.items.length - offset);
      const items = group.items.slice(offset, offset + take);
      const label = offset === 0 ? group.label : group.label ? `${group.label} (cont.)` : undefined;
      current.push({ label, items });
      count += take;
      offset += take;
      if (count >= limit) pushCurrent();
    }
  }
  pushCurrent();
  return slices;
}

function toBlocks(
  flat: FlatStep[],
  miseByProcess?: Map<string, SectionMise>,
): IndexCardMethodBlock[] {
  const blocks: IndexCardMethodBlock[] = [];
  for (const item of flat) {
    if (item.leadIn || blocks.length === 0) {
      const mise = item.leadIn && item.process && miseByProcess?.get(item.process);
      let title: string | undefined;
      if (item.leadIn) {
        title = item.title;
      } else if (item.title) {
        title = indexCardContinuationLabel(item.title);
      }
      blocks.push({
        title,
        note: item.leadIn ? item.note : undefined,
        mise: mise && (mise.ingredients.length || mise.equipment.length) ? mise : undefined,
        steps: [item.step],
      });
    } else {
      blocks[blocks.length - 1].steps.push(item.step);
    }
  }
  return blocks;
}

function finalize(pages: IndexCardPage[]): IndexCardPage[] {
  if (pages.length === 0) {
    return [
      {
        index: 0,
        total: 1,
        showTitle: true,
        continuation: false,
        ingredientGroups: [],
        equipment: [],
        methodBlocks: [],
      },
    ];
  }
  const total = pages.length;
  return pages.map((page, index) => ({ ...page, index, total }));
}

/** Suffix used on a heading or list label that carries over from a prior card. */
export function indexCardContinuationLabel(label: string | undefined): string | undefined {
  if (!label) return undefined;
  return label.endsWith("(cont.)") ? label : `${label} (cont.)`;
}

/** Section label when content may continue from a prior card (equipment, method). */
export function indexCardSectionLabel(
  label: string,
  page: Pick<IndexCardPage, "showTitle" | "ingredientGroups">,
): string {
  return page.showTitle || page.ingredientGroups.length ? label : `${label} (cont.)`;
}

/** Top-matter ingredient list label — continues before equipment/method on later cards. */
export function indexCardIngredientsSectionLabel(
  page: Pick<IndexCardPage, "showTitle" | "ingredientGroups">,
): string {
  return !page.showTitle && page.ingredientGroups.length ? "Ingredients (cont.)" : "Ingredients";
}

/** True while the next section placed on `page` will be its `:first-of-type`. */
function isFirstSection(page: IndexCardPage): boolean {
  return !page.ingredientGroups.length && !page.equipment.length && !page.methodBlocks.length;
}

function ingredientsSectionHeight(
  h: HeightModel,
  groups: IngredientGroup[],
  first: boolean,
  label: string,
): number {
  if (!groups.length) return 0;
  let used = h.sectionTopGap(first) + h.sectionLabel(label);
  for (const group of groups) {
    if (group.label) used += h.groupLabel(group.label);
    for (const item of group.items) {
      used += h.ingredientRow(item);
    }
  }
  return used;
}

function equipmentSectionHeight(
  h: HeightModel,
  items: string[],
  first: boolean,
  label: string,
): number {
  if (!items.length) return 0;
  return h.sectionTopGap(first) + h.sectionLabel(label) + h.equipmentList(items);
}

function methodSectionHeight(
  h: HeightModel,
  blocks: IndexCardMethodBlock[],
  first: boolean,
  label: string,
): number {
  let used = h.sectionTopGap(first) + h.sectionLabel(label);
  let children = 0;
  const add = (cost: number): void => {
    used += cost + (children > 0 ? h.stepsGap : 0);
    children += 1;
  };
  for (const block of blocks) {
    if (block.title) add(h.processHeading(block.title));
    if (block.mise) add(h.miseBlock(block.mise));
    if (block.note) add(h.sectionNote(block.note));
    for (const step of block.steps) {
      add(h.stepRow(step));
    }
  }
  return used;
}

function pageHeight(page: IndexCardPage, h: HeightModel): number {
  let used = 0;
  if (page.showTitle) used += h.titleBlock();
  else if (page.continuation) used += h.continuationHeader();

  let sections = 0;
  if (page.ingredientGroups.length) {
    used += ingredientsSectionHeight(
      h,
      page.ingredientGroups,
      sections === 0,
      indexCardIngredientsSectionLabel(page),
    );
    sections += 1;
  }
  if (page.equipment.length) {
    used += equipmentSectionHeight(
      h,
      page.equipment,
      sections === 0,
      indexCardSectionLabel("Equipment", page),
    );
    sections += 1;
  }
  if (page.methodBlocks.length) {
    used += methodSectionHeight(
      h,
      page.methodBlocks,
      sections === 0,
      indexCardSectionLabel("Method", page),
    );
  }
  return used;
}

/** Sum of block heights for a packed page (for tests / drift checks). */
export function estimateIndexCardPageHeight(
  page: IndexCardPage,
  m: PackMetrics,
  options: { measure?: IndexCardMeasurements; hasCover?: boolean } = {},
): number {
  const hasCover = options.hasCover ?? false;
  const h = options.measure
    ? measuredHeights(m, hasCover, options.measure)
    : heuristicHeights(m, hasCover);
  return pageHeight(page, h);
}

/** Vertical space left on a partially packed page (0 when full). */
function remainingPageRoom(page: IndexCardPage, h: HeightModel): number {
  return Math.max(0, h.metrics.height - pageHeight(page, h));
}

/** Take as many leading ingredient items as fit on `page`, preserving groups. */
function takeIngredients(
  groups: IngredientGroup[],
  page: IndexCardPage,
  h: HeightModel,
): { taken: IngredientGroup[]; rest: IngredientGroup[]; used: number } {
  if (!groups.length) return { taken: [], rest: groups, used: 0 };

  const first = isFirstSection(page);
  const label = indexCardIngredientsSectionLabel(page);
  const chrome = h.sectionTopGap(first) + h.sectionLabel(label);
  if (remainingPageRoom(page, h) < chrome + h.metrics.body) {
    return { taken: [], rest: groups, used: 0 };
  }

  const packLimit = Math.max(0, h.metrics.height - h.ingredientSlack);
  const baseUsed = pageHeight(page, h);
  let used = chrome;
  const taken: IngredientGroup[] = [];

  for (let groupIndex = 0; groupIndex < groups.length; groupIndex += 1) {
    const group = groups[groupIndex];
    const groupLabel = group.label;
    const labelH = groupLabel ? h.groupLabel(groupLabel) : 0;
    let current: IngredientGroup | null = null;
    let itemOffset = 0;

    for (; itemOffset < group.items.length; itemOffset += 1) {
      const item = group.items[itemOffset];
      const rowH = h.ingredientRow(item);
      const extra = current ? 0 : labelH;
      if (baseUsed + used + extra + rowH > packLimit) break;
      if (!current) {
        current = { label: groupLabel, items: [] };
        used += extra;
      }
      current.items.push(item);
      used += rowH;
    }

    if (current?.items.length) taken.push(current);

    if (itemOffset < group.items.length) {
      const rest: IngredientGroup[] = [
        {
          label: indexCardContinuationLabel(group.label),
          items: group.items.slice(itemOffset),
        },
        ...groups.slice(groupIndex + 1),
      ];
      return { taken, rest, used };
    }
  }

  return { taken, rest: [], used };
}

function takeSteps(
  flat: FlatStep[],
  page: IndexCardPage,
  h: HeightModel,
  mises?: Map<string, SectionMise>,
): { taken: FlatStep[]; rest: FlatStep[]; used: number } {
  if (!flat.length) return { taken: [], rest: [], used: 0 };

  const baseUsed = pageHeight(page, h);
  const methodRoom = Math.max(0, (h.metrics.height - baseUsed) * h.methodSafety);
  const first = isFirstSection(page);
  const label = indexCardSectionLabel("Method", page);

  let used = h.sectionTopGap(first) + h.sectionLabel(label);
  if (used > methodRoom) {
    return { taken: [], rest: flat, used: 0 };
  }

  const taken: FlatStep[] = [];
  let children = 0;
  let index = 0;
  while (index < flat.length) {
    const item = flat[index];
    const mise = item.leadIn ? mises?.get(item.process) : undefined;
    let cost = 0;
    let nextChildren = children;
    const add = (blockHeight: number): void => {
      cost += blockHeight + (nextChildren > 0 ? h.stepsGap : 0);
      nextChildren += 1;
    };
    if (item.leadIn && item.title) {
      add(h.processHeading(item.title));
    } else if (!taken.length && item.title) {
      // `toBlocks` re-heads a split section on the card it continues onto.
      add(h.processHeading(indexCardContinuationLabel(item.title) ?? item.title));
    }
    if (item.leadIn && mise) add(h.miseBlock(mise));
    if (item.leadIn && item.note) add(h.sectionNote(item.note));
    add(h.stepRow(item.step));

    if (baseUsed + used + cost > h.metrics.height) break;
    if (used + cost > methodRoom && taken.length > 0) break;
    taken.push(item);
    used += cost;
    children = nextChildren;
    index += 1;
  }

  return { taken, rest: flat.slice(index), used };
}

/** Place leftover equipment on the earliest page with room, or add a continuation card. */
function attachRemainingEquipment(
  pages: IndexCardPage[],
  equipment: string[],
  h: HeightModel,
): void {
  if (!equipment.length || !pages.length) return;
  // Prefer pages without method so equipment stays above steps in reading order.
  const candidates = [
    ...pages.filter((page) => !page.methodBlocks.length),
    ...pages.filter((page) => page.methodBlocks.length),
  ];
  for (const page of candidates) {
    if (remainingPageRoom(page, h) >= equipmentCost(page, equipment, h)) {
      page.equipment = [...page.equipment, ...equipment];
      return;
    }
  }
  pages.push({
    index: 0,
    total: 0,
    showTitle: false,
    continuation: true,
    ingredientGroups: [],
    equipment,
    methodBlocks: [],
  });
}

function equipmentCost(page: IndexCardPage, equipment: string[], h: HeightModel): number {
  return equipmentSectionHeight(
    h,
    equipment,
    isFirstSection(page),
    indexCardSectionLabel("Equipment", page),
  );
}

function tryAttachEquipment(page: IndexCardPage, equipment: string[], h: HeightModel): string[] {
  if (!equipment.length) return equipment;
  if (remainingPageRoom(page, h) >= equipmentCost(page, equipment, h)) {
    page.equipment = [...page.equipment, ...equipment];
    return [];
  }
  return equipment;
}

/**
 * Pack a recipe onto physical index cards.
 *
 * With `measure` the height budget is real rendered pixels and a page is filled
 * until the next block genuinely does not fit. Without it (vitest, or the frame
 * before the probe has been read) the same packer runs on font-metric estimates
 * with conservative headroom.
 */
export function paginateIndexCards(input: {
  format: Exclude<IndexCardFormat, "full">;
  ingredientGroups: IngredientGroup[];
  equipment: string[];
  sections: MethodSection[];
  /** @deprecated Prefer `bodyScale` via pack options; kept for callers. */
  bodyScale?: RecipeTypeScale;
  margin?: IndexCardMargin;
  headerScale?: RecipeTypeScale;
  annotationScale?: RecipeTypeScale;
  /** Reserve space for the “Card N of M” footer. Default true. */
  showPager?: boolean;
  /** Top ingredient list vs per-section mise (mirrors reading misePlacement). */
  miseLayout?: IndexCardMiseLayout;
  /** The title card also renders cover art. */
  hasCover?: boolean;
  /** Rendered heights from `measureIndexCard`; estimates are used without it. */
  measure?: IndexCardMeasurements;
}): IndexCardPage[] {
  const margin = input.margin ?? DEFAULT_INDEX_CARD_MARGIN;
  const bodyScale = input.bodyScale ?? "md";
  const showPager = input.showPager ?? true;
  const miseLayout = input.miseLayout ?? "top-matter";
  const m = cardContentBox(
    input.format,
    margin,
    {
      header: input.headerScale ?? "md",
      body: bodyScale,
      annotation: input.annotationScale ?? "md",
    },
    showPager,
    input.measure,
  );
  const hasCover = input.hasCover ?? false;
  const h = input.measure
    ? measuredHeights(m, hasCover, input.measure)
    : heuristicHeights(m, hasCover);

  if (miseLayout === "colocated") {
    return paginateColocated(h, input.sections);
  }

  let remainingIngredients = input.ingredientGroups;
  let remainingSteps = flattenSteps(input.sections);
  let remainingEquipment = input.equipment;
  const pages: IndexCardPage[] = [];
  let isFirst = true;

  // Generous ceiling — never drop leftover ingredients/steps.
  const maxPages = Math.max(4, countItems(input.ingredientGroups) + remainingSteps.length + 4);

  while (
    (remainingIngredients.length > 0 ||
      remainingSteps.length > 0 ||
      remainingEquipment.length > 0 ||
      pages.length === 0) &&
    pages.length < maxPages
  ) {
    const page: IndexCardPage = {
      index: 0,
      total: 0,
      showTitle: isFirst,
      continuation: !isFirst,
      ingredientGroups: [],
      equipment: [],
      methodBlocks: [],
    };

    if (remainingIngredients.length > 0) {
      const { taken, rest } = takeIngredients(remainingIngredients, page, h);
      page.ingredientGroups = taken;
      remainingIngredients = rest;
    }

    // Traditional order: title → ingredients → equipment → method.
    // Place equipment as soon as the ingredient list is complete, before method.
    if (remainingIngredients.length === 0 && remainingEquipment.length) {
      remainingEquipment = tryAttachEquipment(page, remainingEquipment, h);
    }

    // Co-locate method only after ingredients are placed and equipment is attached
    // (or deferred to a later card because it did not fit here).
    if (
      remainingIngredients.length === 0 &&
      remainingSteps.length > 0 &&
      remainingEquipment.length === 0
    ) {
      const { taken, rest } = takeSteps(remainingSteps, page, h);
      if (taken.length) {
        page.methodBlocks = toBlocks(taken);
        remainingSteps = rest;
      }
    }

    // If nothing fit (pathological), force one step or ingredient to advance.
    if (!page.ingredientGroups.length && !page.methodBlocks.length && !page.equipment.length) {
      if (remainingIngredients.length) {
        const { taken, rest } = takeIngredients(remainingIngredients, page, h);
        if (taken.length) {
          page.ingredientGroups = taken;
          remainingIngredients = rest;
        } else {
          const first = remainingIngredients[0];
          page.ingredientGroups = [{ label: first.label, items: [first.items[0]] }];
          remainingIngredients = [
            { label: indexCardContinuationLabel(first.label), items: first.items.slice(1) },
            ...remainingIngredients.slice(1),
          ].filter((group) => group.items.length > 0);
        }
      } else if (remainingEquipment.length) {
        page.equipment = [remainingEquipment[0]];
        remainingEquipment = remainingEquipment.slice(1);
      } else if (remainingSteps.length) {
        const { taken, rest } = takeSteps(remainingSteps, page, h);
        if (taken.length) {
          page.methodBlocks = toBlocks(taken);
          remainingSteps = rest;
        } else {
          page.methodBlocks = toBlocks([remainingSteps[0]]);
          remainingSteps = remainingSteps.slice(1);
        }
      }
    }

    pages.push(page);
    isFirst = false;

    if (
      remainingIngredients.length === 0 &&
      remainingSteps.length === 0 &&
      remainingEquipment.length === 0
    ) {
      break;
    }
  }

  // Never silently drop leftovers — append forced pages.
  while (
    remainingIngredients.length > 0 ||
    remainingSteps.length > 0 ||
    remainingEquipment.length > 0
  ) {
    const page: IndexCardPage = {
      index: 0,
      total: 0,
      showTitle: false,
      continuation: true,
      ingredientGroups: [],
      equipment: [],
      methodBlocks: [],
    };
    if (remainingIngredients.length) {
      const { taken, rest } = takeIngredients(remainingIngredients, page, h);
      if (taken.length) {
        page.ingredientGroups = taken;
        remainingIngredients = rest;
      } else {
        const first = remainingIngredients[0];
        page.ingredientGroups = [{ label: first.label, items: [first.items[0]] }];
        remainingIngredients = [
          { label: indexCardContinuationLabel(first.label), items: first.items.slice(1) },
          ...remainingIngredients.slice(1),
        ].filter((group) => group.items.length > 0);
      }
    } else if (remainingEquipment.length) {
      remainingEquipment = tryAttachEquipment(page, remainingEquipment, h);
      if (remainingEquipment.length) {
        page.equipment = [remainingEquipment[0]];
        remainingEquipment = remainingEquipment.slice(1);
      }
    } else if (remainingSteps.length) {
      const { taken, rest } = takeSteps(remainingSteps, page, h);
      if (taken.length) {
        page.methodBlocks = toBlocks(taken);
        remainingSteps = rest;
      } else {
        page.methodBlocks = toBlocks([remainingSteps[0]]);
        remainingSteps = remainingSteps.slice(1);
      }
    }
    pages.push(page);
  }

  attachRemainingEquipment(pages, remainingEquipment, h);

  return finalize(pages);
}

/** Pack per-section mise + steps (no top-matter ingredient list). */
function paginateColocated(h: HeightModel, sections: MethodSection[]): IndexCardPage[] {
  const mises = miseMap(sections);
  let remainingSteps = flattenSteps(sections);
  const pages: IndexCardPage[] = [];
  let isFirst = true;
  const maxPages = Math.max(4, remainingSteps.length + 4);

  while ((remainingSteps.length > 0 || pages.length === 0) && pages.length < maxPages) {
    const page: IndexCardPage = {
      index: 0,
      total: 0,
      showTitle: isFirst,
      continuation: !isFirst,
      ingredientGroups: [],
      equipment: [],
      methodBlocks: [],
    };

    const { taken, rest } = takeSteps(remainingSteps, page, h, mises);
    if (taken.length) {
      page.methodBlocks = toBlocks(taken, mises);
      remainingSteps = rest;
    } else if (remainingSteps.length) {
      page.methodBlocks = toBlocks([remainingSteps[0]], mises);
      remainingSteps = remainingSteps.slice(1);
    }

    pages.push(page);
    isFirst = false;
    if (!remainingSteps.length) break;
  }

  while (remainingSteps.length) {
    const page: IndexCardPage = {
      index: 0,
      total: 0,
      showTitle: false,
      continuation: true,
      ingredientGroups: [],
      equipment: [],
      methodBlocks: [],
    };
    const { taken, rest } = takeSteps(remainingSteps, page, h, mises);
    if (taken.length) {
      page.methodBlocks = toBlocks(taken, mises);
      remainingSteps = rest;
    } else {
      page.methodBlocks = toBlocks([remainingSteps[0]], mises);
      remainingSteps = remainingSteps.slice(1);
    }
    pages.push(page);
  }

  return finalize(pages);
}
