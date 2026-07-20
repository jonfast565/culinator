import { computed, onBeforeUnmount, ref, shallowRef, type Ref } from "vue";
import { PageFlip } from "page-flip";

// Wraps StPageFlip's imperative lifecycle. The caller renders `.page` elements
// into `container`, then calls mount(); this owns the instance, tracks the
// visible spread, and degrades gracefully (failed=true) if the library can't
// initialise — the book view then falls back to a plain scroll of leaves.
//
// Spread arithmetic, and why it matters: StPageFlip reports the *left* leaf of
// the visible spread as its "current page". In landscape with showCover=false
// it pairs leaves [0,1], [2,3], … so leaf 5 is reported as page 4. Treating
// that number as "the page you are on" is what made TOC jumps look off by one.
// Everything below is therefore expressed in spreads, with `currentPage` kept
// strictly as what it is: the left-hand leaf.
export function usePageFlip(container: Ref<HTMLElement | null>) {
  const instance = shallowRef<PageFlip | null>(null);
  /** Left-hand leaf of the visible spread, as reported by StPageFlip. */
  const currentPage = ref(0);
  const pageCount = ref(0);
  const isLandscape = ref(true);
  const failed = ref(false);

  const reducedMotion =
    typeof window !== "undefined" &&
    window.matchMedia?.("(prefers-reduced-motion: reduce)").matches === true;

  /** Leaves per spread: two in landscape, one in portrait. */
  const perSpread = computed(() => (isLandscape.value ? 2 : 1));
  /** Total number of flips-worth of spreads in the book. */
  const spreadCount = computed(() =>
    pageCount.value ? Math.ceil(pageCount.value / perSpread.value) : 0,
  );
  const spreadIndex = computed(() => Math.floor(currentPage.value / perSpread.value));

  /** Leaf indices currently on screen — [left, right], right absent in portrait. */
  const visiblePages = computed<number[]>(() => {
    if (!pageCount.value) return [];
    const first = spreadIndex.value * perSpread.value;
    const pages = [first];
    if (isLandscape.value && first + 1 < pageCount.value) pages.push(first + 1);
    return pages;
  });

  /** How many more times the reader can flip in each direction. */
  const flipsBack = computed(() => spreadIndex.value);
  const flipsForward = computed(() => Math.max(0, spreadCount.value - 1 - spreadIndex.value));

  function syncFromInstance(flip: PageFlip): void {
    isLandscape.value = flip.getOrientation() === "landscape";
    pageCount.value = flip.getPageCount();
    currentPage.value = flip.getCurrentPageIndex();
  }

  function mount(): void {
    const el = container.value;
    if (!el || instance.value) return;
    const pages = el.querySelectorAll<HTMLElement>(".page");
    if (!pages.length) return;
    if (el.offsetHeight < 40) {
      requestAnimationFrame(() => mount());
      return;
    }
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
        if (typeof event.data === "number") currentPage.value = event.data;
      });
      // Orientation flips the spread pairing, so the folio side and the
      // remaining-flip counts have to be recomputed when it changes.
      flip.on("changeOrientation", (event) => {
        isLandscape.value = event.data === "landscape";
      });
      flip.turnToPage(0);
      syncFromInstance(flip);

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
    const flip = instance.value;
    if (!flip || flipsForward.value <= 0) return;
    if (reducedMotion) flip.turnToNextPage();
    else flip.flipNext();
  }
  function prev(): void {
    const flip = instance.value;
    if (!flip || flipsBack.value <= 0) return;
    if (reducedMotion) flip.turnToPrevPage();
    else flip.flipPrev();
  }

  /**
   * Jump so that `page` is visible. Deliberately uses turnToPage rather than
   * the animated flip(): flip() pre-sets the spread index and relies on the
   * animation completing to advance it, which desyncs the book from its own
   * page counter if the animation is interrupted. A multi-page TOC jump
   * animates a single turn either way, so there is nothing to lose here.
   */
  function flipTo(page: number): void {
    const flip = instance.value;
    if (!flip) return;
    const clamped = Math.min(Math.max(page, 0), Math.max(0, pageCount.value - 1));
    flip.turnToPage(clamped);
    currentPage.value = flip.getCurrentPageIndex();
  }

  onBeforeUnmount(destroy);

  return {
    currentPage,
    pageCount,
    isLandscape,
    visiblePages,
    spreadIndex,
    spreadCount,
    flipsBack,
    flipsForward,
    failed,
    reducedMotion,
    mount,
    destroy,
    next,
    prev,
    flipTo,
  };
}
