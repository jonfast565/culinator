<script setup lang="ts">
/* global HTMLElement, requestAnimationFrame */
import { nextTick, onMounted, ref } from "vue";
import { ChevronLeft, ChevronRight } from "lucide-vue-next";
import { usePageFlip } from "../usePageFlip";
import BookLeafView from "./BookLeafView.vue";
import type { BookLeaf } from "../bookContents";

const props = defineProps<{ leaves: BookLeaf[] }>();
const emit = defineEmits<{ (event: "open-recipe", recipeId: string): void }>();

const container = ref<HTMLElement | null>(null);
const fallbackRef = ref<HTMLElement | null>(null);
const flip = usePageFlip(container);

function openRecipe(recipeId: string): void {
  emit("open-recipe", recipeId);
}

function fallbackFlipTo(page: number): void {
  const pages = fallbackRef.value?.children;
  if (!pages?.length) return;
  const target = pages.item(Math.min(page, pages.length - 1)) as HTMLElement | null;
  target?.scrollIntoView({ behavior: "smooth", block: "start" });
}

onMounted(async () => {
  await nextTick();
  requestAnimationFrame(() => flip.mount());
});

defineExpose({ leafCount: () => props.leaves.length });
</script>

<template>
  <div class="book-flip-wrap">
    <div v-if="flip.failed.value" ref="fallbackRef" class="flip-fallback">
      <div v-for="leaf in leaves" :key="leaf.key" class="fallback-page">
        <BookLeafView
          :leaf="leaf"
          @open-recipe="openRecipe"
          @flip-to="fallbackFlipTo"
        />
      </div>
    </div>

    <div v-show="!flip.failed.value" ref="container" class="book-flip">
      <div
        v-for="leaf in leaves"
        :key="leaf.key"
        class="page"
        :data-density="leaf.kind === 'cover' ? 'hard' : 'soft'"
      >
        <BookLeafView :leaf="leaf" @open-recipe="openRecipe" @flip-to="flip.flipTo" />
      </div>
    </div>

    <div v-if="!flip.failed.value" class="flip-controls">
      <button class="flip-arrow" title="Previous page" @click="flip.prev()">
        <ChevronLeft :size="20" />
      </button>
      <span class="page-indicator">{{ flip.currentPage.value + 1 }} / {{ flip.pageCount.value }}</span>
      <button class="flip-arrow" title="Next page" @click="flip.next()">
        <ChevronRight :size="20" />
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
  padding: 8px 0 12px;
}
.book-flip {
  width: min(96vw, 1280px);
  height: min(88vh, 920px);
}
.page {
  background: #fbf9f3;
}

.flip-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 12px;
}
.page-indicator {
  min-width: 72px;
  text-align: center;
  font-size: 13px;
  font-variant-numeric: tabular-nums;
  color: #6d7972;
}
.flip-arrow {
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
.flip-arrow:hover {
  background: #f0f3f0;
  color: #28643b;
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
