import type { IndexCardFormat } from "./indexCardFormat";
import { INDEX_CARD_SPECS } from "./indexCardFormat";
import type { IndexCardMargin } from "./indexCardMargin";
import { DEFAULT_INDEX_CARD_MARGIN, indexCardPrintMarginCss } from "./indexCardMargin";

const PRINT_BODY_CLASS = "printing-index-card";
const PRINT_FORMAT_ATTR = "data-index-card-print";
const PAGE_STYLE_ID = "culinator-index-card-page";
const PRINT_ROOT_ID = "culinator-print-root";

let cleanupTimer: ReturnType<typeof setTimeout> | null = null;

function pageSizeCss(format: IndexCardFormat): string {
  if (format === "full") return "letter";
  return INDEX_CARD_SPECS[format].pageSize;
}

function installPageStyle(format: IndexCardFormat, margin: IndexCardMargin): void {
  removePageStyle();
  const style = document.createElement("style");
  style.id = PAGE_STYLE_ID;
  style.textContent = `@page { size: ${pageSizeCss(format)}; margin: ${indexCardPrintMarginCss(format, margin)}; }`;
  document.head.appendChild(style);
}

function removePageStyle(): void {
  document.getElementById(PAGE_STYLE_ID)?.remove();
}

function clearCleanupTimer(): void {
  if (cleanupTimer != null) {
    clearTimeout(cleanupTimer);
    cleanupTimer = null;
  }
}

/** Clone UI card frames (or the full-page leaf) into a body-level print root. */
function mountPrintRoot(format: IndexCardFormat): HTMLElement | null {
  document.getElementById(PRINT_ROOT_ID)?.remove();

  const root = document.createElement("div");
  root.id = PRINT_ROOT_ID;
  root.className = "culinator-print-root";
  root.setAttribute(PRINT_FORMAT_ATTR, format);

  if (format !== "full") {
    const frames = document.querySelectorAll(".index-card-deck .index-card-frame");
    if (!frames.length) return null;
    frames.forEach((frame) => {
      root.appendChild(frame.cloneNode(true));
    });
  } else {
    const leaf = document.querySelector(".reading-stage .leaf, .preview-stage .leaf");
    if (!leaf) return null;
    const page = document.createElement("div");
    page.className = "index-card-frame print-full-page";
    page.appendChild(leaf.cloneNode(true));
    root.appendChild(page);
  }

  document.body.appendChild(root);
  return root;
}

async function requestPrint(): Promise<void> {
  const result = window.print() as void | Promise<unknown>;
  await Promise.resolve(result);
}

export interface PrintIndexCardOptions {
  format: IndexCardFormat;
  margin?: IndexCardMargin;
  /** Used as the Save-as-PDF / print job document title. */
  title?: string;
}

/** Build a Save-as-PDF title: `Book – Recipe` when a book is known. */
export function printableDocumentTitle(recipeTitle: string, bookTitle?: string | null): string {
  const recipe = recipeTitle.trim() || "Recipe";
  const book = bookTitle?.trim();
  return book ? `${book} – ${recipe}` : recipe;
}

/** Trigger a print of the current recipe at the chosen index-card size. */
export async function printIndexCard(
  formatOrOptions: IndexCardFormat | PrintIndexCardOptions,
  marginArg: IndexCardMargin = DEFAULT_INDEX_CARD_MARGIN,
): Promise<void> {
  const options: PrintIndexCardOptions =
    typeof formatOrOptions === "string"
      ? { format: formatOrOptions, margin: marginArg }
      : formatOrOptions;
  const format = options.format;
  const margin = options.margin ?? DEFAULT_INDEX_CARD_MARGIN;
  const previousTitle = document.title;

  clearCleanupTimer();
  const root = mountPrintRoot(format);
  if (!root) {
    console.error("Print failed — no recipe card content found to print.");
    return;
  }

  if (options.title?.trim()) {
    document.title = options.title.trim();
  }
  document.body.classList.add(PRINT_BODY_CLASS);
  document.body.setAttribute(PRINT_FORMAT_ATTR, format);
  installPageStyle(format, margin);

  let cleaned = false;
  const cleanup = () => {
    if (cleaned) return;
    cleaned = true;
    clearCleanupTimer();
    document.body.classList.remove(PRINT_BODY_CLASS);
    document.body.removeAttribute(PRINT_FORMAT_ATTR);
    removePageStyle();
    document.getElementById(PRINT_ROOT_ID)?.remove();
    document.title = previousTitle;
  };

  window.addEventListener("afterprint", cleanup, { once: true });
  cleanupTimer = setTimeout(cleanup, 60_000);

  try {
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    await requestPrint();
  } catch (error) {
    cleanup();
    console.error("Print failed — is core:webview:allow-print enabled?", error);
    throw error;
  }
}
