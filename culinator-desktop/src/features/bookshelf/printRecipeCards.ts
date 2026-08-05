const PRINT_BODY_CLASS = "printing-recipe-cards";
const PAGE_STYLE_ID = "culinator-recipe-cards-page";

let cleanupTimer: ReturnType<typeof setTimeout> | null = null;

function installPageStyle(): void {
  document.getElementById(PAGE_STYLE_ID)?.remove();
  const style = document.createElement("style");
  style.id = PAGE_STYLE_ID;
  style.textContent = "@page { size: landscape; margin: 0.45in; }";
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

async function requestPrint(): Promise<void> {
  const result = window.print() as void | Promise<unknown>;
  await Promise.resolve(result);
}

/** Trigger a landscape print of the open book's recipe card grid. */
export async function printRecipeCards(bookTitle?: string): Promise<void> {
  clearCleanupTimer();
  const previousTitle = document.title;
  if (bookTitle?.trim()) document.title = bookTitle.trim();

  document.body.classList.add(PRINT_BODY_CLASS);
  installPageStyle();

  let cleaned = false;
  const cleanup = () => {
    if (cleaned) return;
    cleaned = true;
    clearCleanupTimer();
    document.body.classList.remove(PRINT_BODY_CLASS);
    removePageStyle();
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
