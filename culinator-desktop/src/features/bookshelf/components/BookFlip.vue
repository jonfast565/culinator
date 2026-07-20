<script setup lang="ts">
/* global HTMLElement, KeyboardEvent, EventTarget, requestAnimationFrame */
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { ChevronLeft, ChevronRight } from "lucide-vue-next";
import { usePageFlip } from "../usePageFlip";
import BookLeafView from "./BookLeafView.vue";
import type { BookLeaf } from "../bookContents";

const props = defineProps<{ leaves: BookLeaf[] }>();
const emit = defineEmits<{ (event: "open-recipe", recipeId: string): void }>();

const container = ref<HTMLElement | null>(null);
const fallbackRef = ref<HTMLElement | null>(null);
const flip = usePageFlip(container);

/**
 * Which corner a leaf's folio belongs in. Landscape pairs leaves [0,1], [2,3],
 * … so even indices sit on the left of the spread; portrait shows one leaf at
 * a time, which reads as a right-hand page.
 */
function sideFor(index: number): "left" | "right" {
  if (!flip.isLandscape.value) return "right";
  return index % 2 === 0 ? "left" : "right";
}

/** "6" for a single page, "6–7" for a spread. */
const pageLabel = computed(() => {
  const pages = flip.visiblePages.value;
  if (!pages.length) return "—";
  const numbers = pages.map((page) => page + 1);
  return numbers.length > 1 ? `${numbers[0]}–${numbers[numbers.length - 1]}` : `${numbers[0]}`;
});

function flipsLabel(count: number, direction: "back" | "forward"): string {
  if (count <= 0) return direction === "back" ? "Start of the book" : "End of the book";
  return `${direction === "back" ? "Back" : "Forward"} ${count} flip${count === 1 ? "" : "s"}`;
}

function openRecipe(recipeId: string): void {
  emit("open-recipe", recipeId);
}

function fallbackFlipTo(page: number): void {
  const pages = fallbackRef.value?.children;
  if (!pages?.length) return;
  const target = pages.item(Math.min(page, pages.length - 1)) as HTMLElement | null;
  target?.scrollIntoView({ behavior: "smooth", block: "start" });
}

// Arrow keys turn pages while the book is on screen. Ignored when the reader is
// typing or driving another widget, and when a modifier implies a browser/OS
// shortcut rather than a page turn.
function isTypingTarget(target: EventTarget | null): boolean {
  const el = target as HTMLElement | null;
  if (!el?.tagName) return false;
  return (
    ["INPUT", "TEXTAREA", "SELECT"].includes(el.tagName) ||
    el.isContentEditable ||
    Boolean(el.closest?.("[role='dialog'], [contenteditable='true']"))
  );
}

function onKeydown(event: KeyboardEvent): void {
  if (flip.failed.value) return;
  if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
  if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return;
  if (isTypingTarget(event.target)) return;
  event.preventDefault();
  if (event.key === "ArrowRight") flip.next();
  else flip.prev();
}

onMounted(async () => {
  await nextTick();
  requestAnimationFrame(() => flip.mount());
  window.addEventListener("keydown", onKeydown);
});
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));

defineExpose({ leafCount: () => props.leaves.length });
</script>

<template>
  <div class="book-flip-wrap">
    <div v-if="flip.failed.value" ref="fallbackRef" class="flip-fallback">
      <div v-for="(leaf, index) in leaves" :key="leaf.key" class="fallback-page">
        <BookLeafView
          :leaf="leaf"
          :page-number="index + 1"
          side="right"
          @open-recipe="openRecipe"
          @flip-to="fallbackFlipTo"
        />
      </div>
    </div>

    <div v-show="!flip.failed.value" ref="container" class="book-flip">
      <div
        v-for="(leaf, index) in leaves"
        :key="leaf.key"
        class="page"
        :data-density="leaf.kind === 'cover' ? 'hard' : 'soft'"
      >
        <BookLeafView
          :leaf="leaf"
          :page-number="index + 1"
          :side="sideFor(index)"
          @open-recipe="openRecipe"
          @flip-to="flip.flipTo"
        />
      </div>
    </div>

    <div v-if="!flip.failed.value" class="flip-controls">
      <button
        type="button"
        class="flip-arrow"
        :title="flipsLabel(flip.flipsBack.value, 'back') + ' (←)'"
        :disabled="flip.flipsBack.value <= 0"
        @click="flip.prev()"
      >
        <ChevronLeft :size="20" />
        <span class="flip-count">{{ flip.flipsBack.value }}</span>
      </button>
      <span class="page-indicator">{{ pageLabel }} / {{ flip.pageCount.value }}</span>
      <button
        type="button"
        class="flip-arrow"
        :title="flipsLabel(flip.flipsForward.value, 'forward') + ' (→)'"
        :disabled="flip.flipsForward.value <= 0"
        @click="flip.next()"
      >
        <ChevronRight :size="20" />
        <span class="flip-count">{{ flip.flipsForward.value }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.book-flip-wrap {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 8px 16px 16px;
  overflow: hidden;
}
.book-flip {
  flex: 1;
  min-height: 0;
  width: min(96vw, 1280px);
  max-height: 100%;
}
.page {
  background: #fbf9f3;
}

.flip-controls {
  flex-shrink: 0;
  position: relative;
  z-index: 5;
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 12px;
  padding-bottom: 4px;
}
.page-indicator {
  min-width: 96px;
  text-align: center;
  font-size: 13px;
  font-variant-numeric: tabular-nums;
  color: #6d7972;
}
.flip-arrow {
  position: relative;
  display: grid;
  place-items: center;
  width: 42px;
  height: 42px;
  border-radius: 50%;
  border: 1px solid #cbd3cd;
  background: #fff;
  color: #23302a;
  cursor: pointer;
}
/* Remaining flips available in this direction. */
.flip-count {
  position: absolute;
  top: -3px;
  right: -3px;
  min-width: 18px;
  padding: 0 4px;
  border-radius: 999px;
  background: #e7ece7;
  color: #46574e;
  font-size: 10px;
  line-height: 17px;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}
.flip-arrow:disabled .flip-count {
  background: #eef0ee;
}
.flip-arrow:hover:not(:disabled) {
  background: #f0f3f0;
  color: #28643b;
}
.flip-arrow:disabled {
  opacity: 0.35;
  cursor: default;
}

.flip-fallback {
  width: min(96vw, 900px);
  max-height: 100%;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 12px 0;
}
.fallback-page {
  background: #fbf9f3;
  border-radius: 4px;
  box-shadow: 0 10px 30px -18px rgba(40, 40, 30, 0.5);
  min-height: 420px;
  display: flex;
}
</style>
