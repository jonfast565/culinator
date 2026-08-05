/**
 * Real rendered heights for index-card pagination.
 *
 * The packer used to guess block heights from font metrics (chars-per-line
 * estimates plus fudge factors), which meant one section stopped clipping every
 * time another started. Instead, an offscreen probe card — same classes, same
 * `data-*` attributes, same custom properties, same physical width as a real
 * card — renders every block the deck can contain, each tagged with
 * `data-measure-key`. This module configures that probe, waits for fonts and
 * images, and reads back exact pixel heights for `paginateIndexCards`.
 *
 * The probe is only a host: `IndexCardProbe.vue` teleports the real card
 * components into it, so scoped component styles and the global
 * `index-card-view.css` overrides apply exactly as they do on screen.
 */
import type { IndexCardFormat } from "./indexCardFormat";
import { INDEX_CARD_SPECS } from "./indexCardFormat";
import type { IndexCardMargin } from "./indexCardMargin";
import { INDEX_CARD_MARGIN_IN } from "./indexCardMargin";
import type { RecipeTypeScale } from "./recipeTypeScale";
import { recipeTypeScaleFactor } from "./recipeTypeScale";
import type {
  IngredientDisplayParts,
  NarrativeStep,
  SectionMise,
} from "../recipe-editor/narrative";

/** Everything outside the recipe text that changes a rendered height. */
export interface IndexCardMeasureContext {
  format: Exclude<IndexCardFormat, "full">;
  margin: IndexCardMargin;
  headerScale: RecipeTypeScale;
  bodyScale: RecipeTypeScale;
  annotationScale: RecipeTypeScale;
  /**
   * `:root[data-text-size]` preset. The probe inherits `--reading-scale` from
   * the document root, so this is carried only to invalidate the cache.
   */
  textSize: string;
}

export interface IndexCardMeasurements {
  /** Effective height (border box + margins) in px, per block key. */
  heights: ReadonlyMap<string, number>;
  /** Leaf content box, measured on the probe (padding and border removed). */
  contentWidth: number;
  contentHeight: number;
  /** `.leaf-section` margin-top, and its `:first-of-type` variant. */
  sectionGap: number;
  firstSectionGap: number;
  /** `.steps` row-gap, charged between method children. */
  stepsGap: number;
  /** Height the absolutely positioned pager takes out of the content box. */
  pagerReserve: number;
}

export interface IndexCardProbeHost {
  stage: HTMLElement;
  frame: HTMLElement;
  /** Teleport target: the offscreen `.leaf.index-card-leaf`. */
  leaf: HTMLElement;
}

/** Unit separator — cannot appear in recipe prose, so keys stay unambiguous. */
const SEP = "\u001f";

function ingredientRowKey(item: IngredientDisplayParts): string {
  return ["ingredient", item.symbol, item.amount, item.description, item.aside ?? ""].join(SEP);
}

/**
 * Block keys shared by the probe (which renders them) and the packer (which
 * looks them up). Keys are content-derived, so the packer can ask for any
 * subset it ends up placing without tracking probe indices.
 */
export const indexCardBlockKey = {
  titleCover: "title-cover",
  titleHead: "title-head",
  continuationHead: "continuation-head",
  sectionLabel: (text: string): string => `section-label${SEP}${text}`,
  groupLabel: (text: string): string => `group-label${SEP}${text}`,
  ingredientRow: ingredientRowKey,
  equipmentList: (items: readonly string[]): string => `equipment${SEP}${items.join(SEP)}`,
  processHeading: (text: string): string => `process-heading${SEP}${text}`,
  mise: (mise: SectionMise): string =>
    ["mise", mise.ingredients.map(ingredientRowKey).join(SEP), mise.equipment.join(SEP)].join(
      `${SEP}|${SEP}`,
    ),
  sectionNote: (text: string): string => `section-note${SEP}${text}`,
  step: (step: NarrativeStep): string =>
    ["step", step.symbol, String(step.number), step.text, step.time ?? ""].join(SEP),
};

/** Fallbacks for a probe that rendered without the corresponding element. */
const DEFAULT_SECTION_GAP_PX = 8;
const DEFAULT_FIRST_SECTION_GAP_PX = 6;
const DEFAULT_STEPS_GAP_PX = 2;
const DEFAULT_PAGER_RESERVE_PX = 14;

/** Give up waiting on a stalled web font or image and measure what we have. */
const PROBE_ASSET_TIMEOUT_MS = 1500;

const OFFSCREEN_STAGE_CSS = [
  "position: fixed",
  "top: 0",
  "left: -20000px",
  "display: block",
  "width: auto",
  "max-width: none",
  "min-height: 0",
  "visibility: hidden",
  "pointer-events: none",
  "z-index: -1",
].join("; ");

function px(value: string): number {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

/** Create the offscreen card shell. Returns null outside a browser. */
export function createIndexCardProbeHost(): IndexCardProbeHost | null {
  if (typeof document === "undefined") return null;
  const stage = document.createElement("div");
  stage.className = "index-card-stage index-card-measure-stage";
  stage.setAttribute("aria-hidden", "true");
  stage.style.cssText = OFFSCREEN_STAGE_CSS;

  const frame = document.createElement("div");
  frame.className = "index-card-frame";

  const leaf = document.createElement("article");
  leaf.className = "leaf index-card-leaf";

  frame.appendChild(leaf);
  stage.appendChild(frame);
  document.body.appendChild(stage);
  return { stage, frame, leaf };
}

export function destroyIndexCardProbeHost(host: IndexCardProbeHost | null): void {
  host?.stage.remove();
}

/**
 * Point the probe at one format / margin / type-scale combination.
 *
 * The frame keeps its 1px border and the leaf keeps its margin padding, so the
 * probe's content width is the on-screen content width — not the slightly wider
 * print box. Wrapping therefore matches the card the reader sees, and print (a
 * hair wider) can only wrap less. The frame is given the card's real height so
 * percentage limits like the cover's `max-height: 10%` resolve as they do on a
 * card; content is free to overflow it while blocks report natural heights.
 */
export function configureIndexCardProbeHost(
  host: IndexCardProbeHost,
  context: IndexCardMeasureContext,
): void {
  const spec = INDEX_CARD_SPECS[context.format];
  const marginIn = INDEX_CARD_MARGIN_IN[context.margin].card;
  const { stage, frame, leaf } = host;

  stage.dataset.format = context.format;
  stage.dataset.typeTier = spec.typeTier;
  stage.dataset.margin = context.margin;
  stage.style.setProperty(
    "--recipe-header-scale",
    String(recipeTypeScaleFactor(context.headerScale)),
  );
  stage.style.setProperty("--recipe-body-scale", String(recipeTypeScaleFactor(context.bodyScale)));
  stage.style.setProperty(
    "--recipe-annotation-scale",
    String(recipeTypeScaleFactor(context.annotationScale)),
  );
  stage.style.setProperty("--index-card-margin", `${marginIn}in`);
  stage.style.setProperty("--index-card-width", `${spec.widthIn}in`);
  stage.style.setProperty("--index-card-aspect-ratio", `${spec.widthIn} / ${spec.heightIn}`);

  frame.style.width = `${spec.widthIn}in`;
  frame.style.height = `${spec.heightIn}in`;
  frame.style.aspectRatio = "auto";
  frame.style.maxWidth = "none";
  frame.style.flex = "0 0 auto";
  frame.style.overflow = "visible";

  leaf.style.height = "100%";
  leaf.style.maxHeight = "none";
  leaf.style.overflow = "visible";
}

function timeout(ms: number): Promise<void> {
  return new Promise((resolve) => {
    window.setTimeout(resolve, ms);
  });
}

/** Web-font swap and image decode both change heights — wait them out. */
export async function indexCardProbeReady(host: IndexCardProbeHost): Promise<void> {
  const pending: Promise<unknown>[] = [];
  const fonts = document.fonts;
  if (fonts) pending.push(fonts.ready);
  for (const image of host.leaf.querySelectorAll("img")) {
    if (image.complete) continue;
    pending.push(
      new Promise<void>((resolve) => {
        const settle = (): void => resolve();
        image.addEventListener("load", settle, { once: true });
        image.addEventListener("error", settle, { once: true });
      }),
    );
  }
  if (!pending.length) return;
  await Promise.race([Promise.all(pending), timeout(PROBE_ASSET_TIMEOUT_MS)]);
}

/**
 * Space a block occupies in the card's flex column: its border box plus its own
 * margins. Card sections and method children are flex items, so margins never
 * collapse and this sum is exactly what the packer has to budget. A block the
 * card hides (the Method label in the colocated layout) costs nothing.
 */
function effectiveHeight(element: HTMLElement): number {
  const style = window.getComputedStyle(element);
  if (style.display === "none") return 0;
  return element.getBoundingClientRect().height + px(style.marginTop) + px(style.marginBottom);
}

function sectionMarginTop(element: HTMLElement | null, fallback: number): number {
  if (!element) return fallback;
  return px(window.getComputedStyle(element).marginTop);
}

function pagerReserve(leaf: HTMLElement, pager: HTMLElement | null): number {
  if (!pager) return DEFAULT_PAGER_RESERVE_PX;
  const pagerStyle = window.getComputedStyle(pager);
  const leafStyle = window.getComputedStyle(leaf);
  // `bottom` is measured from the frame; only the part above the leaf padding
  // eats into the content box.
  const inset = Math.max(0, px(pagerStyle.bottom) - px(leafStyle.paddingBottom));
  return pager.getBoundingClientRect().height + inset;
}

/** Read every tagged block in the probe. Forces one layout pass. */
export function readIndexCardMeasurements(host: IndexCardProbeHost): IndexCardMeasurements {
  const { leaf } = host;
  const leafStyle = window.getComputedStyle(leaf);
  const rect = leaf.getBoundingClientRect();

  const heights = new Map<string, number>();
  for (const element of leaf.querySelectorAll<HTMLElement>("[data-measure-key]")) {
    const key = element.dataset.measureKey;
    if (key) heights.set(key, effectiveHeight(element));
  }

  const gapProbes = leaf.querySelectorAll<HTMLElement>('[data-measure-role="section-gap"]');
  const steps = leaf.querySelector<HTMLElement>('[data-measure-role="steps"]');

  return {
    heights,
    contentWidth: Math.max(1, rect.width - px(leafStyle.paddingLeft) - px(leafStyle.paddingRight)),
    contentHeight: Math.max(
      1,
      rect.height - px(leafStyle.paddingTop) - px(leafStyle.paddingBottom),
    ),
    firstSectionGap: sectionMarginTop(
      leaf.querySelector<HTMLElement>("section.leaf-section"),
      DEFAULT_FIRST_SECTION_GAP_PX,
    ),
    sectionGap: sectionMarginTop(gapProbes[gapProbes.length - 1] ?? null, DEFAULT_SECTION_GAP_PX),
    stepsGap: steps ? px(window.getComputedStyle(steps).rowGap) : DEFAULT_STEPS_GAP_PX,
    pagerReserve: pagerReserve(
      leaf,
      leaf.querySelector<HTMLElement>('[data-measure-role="pager"]'),
    ),
  };
}

/**
 * Cache key for one probe pass: the visual context plus every piece of text the
 * probe renders. Any change to either invalidates the entry.
 */
export function indexCardMeasureSignature(
  context: IndexCardMeasureContext,
  content: unknown,
): string {
  return JSON.stringify([
    context.format,
    context.margin,
    context.headerScale,
    context.bodyScale,
    context.annotationScale,
    context.textSize,
    content,
  ]);
}

/** Keeps format/margin toggling instant instead of re-probing every flip. */
const MEASURE_CACHE_LIMIT = 16;
const measureCache = new Map<string, IndexCardMeasurements>();

export function cachedIndexCardMeasurements(signature: string): IndexCardMeasurements | null {
  return measureCache.get(signature) ?? null;
}

export function cacheIndexCardMeasurements(
  signature: string,
  measurements: IndexCardMeasurements,
): void {
  measureCache.delete(signature);
  measureCache.set(signature, measurements);
  if (measureCache.size > MEASURE_CACHE_LIMIT) {
    const oldest = measureCache.keys().next();
    if (!oldest.done) measureCache.delete(oldest.value);
  }
}
