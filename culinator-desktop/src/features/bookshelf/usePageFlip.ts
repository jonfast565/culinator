import { onBeforeUnmount, ref, shallowRef, type Ref } from "vue";
import { PageFlip } from "page-flip";

// Wraps StPageFlip's imperative lifecycle. The caller renders `.page` elements
// into `container`, then calls mount(); this owns the instance, tracks the
// current page, and degrades gracefully (failed=true) if the library can't
// initialise — the book view then falls back to a plain scroll of leaves.
export function usePageFlip(container: Ref<HTMLElement | null>) {
  const instance = shallowRef<PageFlip | null>(null);
  const currentPage = ref(0);
  const pageCount = ref(0);
  const failed = ref(false);

  const reducedMotion =
    typeof window !== "undefined" &&
    window.matchMedia?.("(prefers-reduced-motion: reduce)").matches === true;

  function mount(): void {
    const el = container.value;
    if (!el) return;
    const pages = el.querySelectorAll<HTMLElement>(".page");
    if (!pages.length) return;
    try {
      const flip = new PageFlip(el, {
        width: 520,
        height: 720,
        size: "stretch",
        minWidth: 320,
        maxWidth: 820,
        minHeight: 440,
        maxHeight: 980,
        startPage: 0,
        drawShadow: true,
        maxShadowOpacity: 0.5,
        flippingTime: reducedMotion ? 0 : 700,
        usePortrait: true,
        showCover: false,
        clickEventForward: true,
        mobileScrollSupport: false,
        showPageCorners: true,
      });
      flip.loadFromHTML(pages);
      flip.on("flip", (event) => {
        currentPage.value = event.data;
      });
      pageCount.value = flip.getPageCount();
      flip.turnToPage(0);
      currentPage.value = flip.getCurrentPageIndex();

      // StPageFlip re-parents page DOM; delegate TOC clicks so navigation still works.
      el.addEventListener("click", onDelegatedClick);

      instance.value = flip;
      failed.value = false;
    } catch (error) {
      console.error("page-flip failed to initialise; falling back to scroll", error);
      failed.value = true;
    }
  }

  function onDelegatedClick(event: MouseEvent): void {
    const target = (event.target as HTMLElement | null)?.closest<HTMLElement>("[data-flip-to]");
    if (!target) return;
    const page = Number(target.dataset.flipTo);
    if (Number.isFinite(page)) flipTo(page);
  }

  function destroy(): void {
    container.value?.removeEventListener("click", onDelegatedClick);
    try {
      instance.value?.destroy();
    } catch {
      // StPageFlip can throw during teardown if the DOM is already gone; ignore.
    }
    instance.value = null;
  }

  function next(): void {
    instance.value?.flipNext();
  }
  function prev(): void {
    instance.value?.flipPrev();
  }
  function flipTo(page: number): void {
    const flip = instance.value;
    if (!flip) return;
    if (reducedMotion) flip.turnToPage(page);
    else flip.flip(page);
  }

  onBeforeUnmount(destroy);

  return { currentPage, pageCount, failed, reducedMotion, mount, destroy, next, prev, flipTo };
}
