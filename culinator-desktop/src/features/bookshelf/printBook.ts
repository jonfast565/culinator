import type { BookExportOptions } from "../../domain/types";
import { exportBook } from "../../services/api/export-api";

const IFRAME_ID = "culinator-book-print-frame";

function decodeArchive(archiveBase64: string): string {
  const bytes = Uint8Array.from(atob(archiveBase64), (char) => char.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}

async function requestPrint(win: Window): Promise<void> {
  const result = win.print() as void | Promise<unknown>;
  await Promise.resolve(result);
}

/**
 * Export the book as Print HTML and open the system print dialog.
 * Save-as-PDF uses the book title as the document name.
 */
export async function printBook(
  bookId: string,
  bookTitle: string,
  options: Partial<BookExportOptions> = {},
): Promise<void> {
  const previousTitle = document.title;
  const title = (options.title ?? bookTitle).trim() || "Book";
  document.title = title;

  const bundle = await exportBook(bookId, {
    formats: ["print_html"],
    title,
    author: options.author ?? "",
    description: options.description ?? "",
    unitSystem: options.unitSystem ?? "metric",
    includeNutrition: options.includeNutrition ?? true,
    toc: options.toc ?? true,
    sectionDividers: options.sectionDividers ?? true,
  });

  document.getElementById(IFRAME_ID)?.remove();
  const iframe = document.createElement("iframe");
  iframe.id = IFRAME_ID;
  iframe.setAttribute("aria-hidden", "true");
  iframe.style.cssText =
    "position:fixed;right:0;bottom:0;width:0;height:0;border:0;opacity:0;pointer-events:none";
  document.body.appendChild(iframe);

  const doc = iframe.contentDocument;
  const win = iframe.contentWindow;
  if (!doc || !win) {
    document.title = previousTitle;
    iframe.remove();
    throw new Error("Could not open a print frame for the book");
  }

  const html = decodeArchive(bundle.archiveBase64);
  doc.open();
  doc.write(html);
  doc.close();
  // Match Save-as-PDF naming inside the iframe document too.
  doc.title = title;

  let cleaned = false;
  const cleanup = () => {
    if (cleaned) return;
    cleaned = true;
    iframe.remove();
    document.title = previousTitle;
  };

  win.addEventListener("afterprint", cleanup, { once: true });
  window.setTimeout(cleanup, 60_000);

  try {
    await new Promise<void>((resolve) => {
      if (doc.readyState === "complete") {
        requestAnimationFrame(() => resolve());
      } else {
        iframe.addEventListener("load", () => resolve(), { once: true });
        window.setTimeout(() => resolve(), 250);
      }
    });
    win.focus();
    await requestPrint(win);
  } catch (error) {
    cleanup();
    console.error("Book print failed — is core:webview:allow-print enabled?", error);
    throw error;
  }
}
