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
const flip = usePageFlip(container);

function openRecipe(recipeId: string): void {
  emit("open-recipe", recipeId);
}

onMounted(async () => {
  await nextTick();
  // One frame so the flex container has measured its size before StPageFlip
  // reads the bounding rect.
  requestAnimationFrame(() => flip.mount());
});

defineExpose({ leafCount: () => props.leaves.length });
</script>

<template>
  <div class="book-flip-wrap">
    <!-- Fallback: plain scroll of leaves if StPageFlip can't initialise -->
    <div v-if="flip.failed.value" class="flip-fallback">
      <div v-for="leaf in leaves" :key="leaf.key" class="fallback-page">
        <BookLeafView :leaf="leaf" @open-recipe="openRecipe" @flip-to="() => {}" />
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
}
.book-flip {
  width: min(94vw, 1000px);
  height: min(76vh, 700px);
}
.page {
  background: #fbf9f3;
}

.flip-controls {
  display: flex;
  gap: 12px;
  margin-top: 16px;
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
  width: min(94vw, 720px);
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
  min-height: 320px;
  display: flex;
}
</style>
