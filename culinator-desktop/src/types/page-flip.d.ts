// Minimal type surface for the `page-flip` (StPageFlip) library, which ships no
// declarations. Only the members we use are typed.
declare module "page-flip" {
  export interface PageFlipSettings {
    width: number;
    height: number;
    size?: "fixed" | "stretch";
    minWidth?: number;
    maxWidth?: number;
    minHeight?: number;
    maxHeight?: number;
    startPage?: number;
    drawShadow?: boolean;
    flippingTime?: number;
    usePortrait?: boolean;
    startZIndex?: number;
    autoSize?: boolean;
    maxShadowOpacity?: number;
    showCover?: boolean;
    mobileScrollSupport?: boolean;
    clickEventForward?: boolean;
    useMouseEvents?: boolean;
    swipeDistance?: number;
    showPageCorners?: boolean;
    disableFlipByClick?: boolean;
  }

  export interface FlipEvent {
    data: number;
    object: PageFlip;
  }

  export class PageFlip {
    constructor(element: HTMLElement, settings: PageFlipSettings);
    loadFromHTML(items: NodeListOf<Element> | HTMLElement[]): void;
    updateFromHTML(items: NodeListOf<Element> | HTMLElement[]): void;
    turnToPage(page: number): void;
    turnToNextPage(): void;
    turnToPrevPage(): void;
    flip(page: number): void;
    flipNext(): void;
    flipPrev(): void;
    getPageCount(): number;
    getCurrentPageIndex(): number;
    getOrientation(): "portrait" | "landscape";
    on(
      event: "flip" | "changeState" | "changeOrientation" | "init",
      cb: (e: FlipEvent) => void,
    ): void;
    off(event: string): void;
    destroy(): void;
    clear(): void;
  }
}
