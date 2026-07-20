<script setup lang="ts">
import { computed, ref } from "vue";
import { Eye, TriangleAlert } from "lucide-vue-next";
import type { UiRecipeModel } from "../../recipe-editor/model";
import RecipePage from "../../reading/components/RecipePage.vue";

/**
 * The live half of the builder: the same reading page the app renders
 * elsewhere, driven by the model the form is editing, plus the parser's
 * recovery diagnostics. Nothing here writes back — it is a mirror.
 */
const props = defineProps<{
  model: UiRecipeModel;
  source: string;
  recipeId?: string;
}>();

const tab = ref<"preview" | "issues">("preview");
const diagnostics = computed(() => props.model.diagnostics);
</script>

<template>
  <aside class="preview-pane">
    <nav class="tabs">
      <button :class="{ active: tab === 'preview' }" @click="tab = 'preview'">
        <Eye :size="14" /> Preview
      </button>
      <button :class="{ active: tab === 'issues' }" @click="tab = 'issues'">
        <TriangleAlert :size="14" /> Issues
        <span v-if="diagnostics.length" class="count">{{ diagnostics.length }}</span>
      </button>
    </nav>

    <div v-show="tab === 'preview'" class="preview-stage">
      <RecipePage :model="model" :source="source" :recipe-id="recipeId" />
    </div>

    <div v-show="tab === 'issues'" class="issues">
      <p v-if="!diagnostics.length" class="empty">No problems — the recipe parses cleanly.</p>
      <ul v-else>
        <li
          v-for="(item, index) in diagnostics"
          :key="`${item.message}-${index}`"
          class="diagnostic"
        >
          <TriangleAlert :size="14" />
          <span>{{ item.message }}</span>
        </li>
      </ul>
    </div>
  </aside>
</template>

<style scoped>
.preview-pane {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  background: #f7f6f2;
  border-left: 1px solid #d3d8d1;
}
.tabs {
  display: flex;
  gap: 3px;
  padding: 8px 10px;
  background: #fff;
  border-bottom: 1px solid #d8ddd9;
}
.tabs button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 6px 10px;
  font-size: 12px;
  background: transparent;
  border: 0;
  border-radius: 7px;
  color: #55635b;
}
.tabs button.active {
  background: #e4efe6;
  color: #28643b;
}
.count {
  min-width: 18px;
  padding: 0 5px;
  border-radius: 9px;
  background: #e7b8b3;
  color: #6f2018;
  font-size: 11px;
  text-align: center;
}
.preview-stage {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: clamp(16px, 3vw, 36px) 16px;
  background: radial-gradient(120% 80% at 50% -10%, #efece2 0%, #e7e3d6 55%, #e0dbcb 100%);
}
.issues {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 14px;
}
.issues ul {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 8px;
}
.issues .diagnostic {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  padding: 10px 12px;
  border-radius: 8px;
  background: #fbeceb;
  color: #8a3a32;
  font-size: 13px;
  border-left: 3px solid #cf6f64;
}
.empty {
  color: #8a938c;
  font-size: 13px;
}
</style>
