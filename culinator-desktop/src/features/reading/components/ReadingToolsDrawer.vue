<script setup lang="ts">
import { ref, watch } from "vue";
import { ChefHat, PackageOpen, X } from "lucide-vue-next";
import type { UiOperation, UiRecipeModel } from "../../recipe-editor/model";
import KitchenModePanel from "../../kitchen-mode/components/KitchenModePanel.vue";
import ExportPanel from "../../export/components/ExportPanel.vue";

const props = defineProps<{
  model: UiRecipeModel;
  recipeId: string;
  operations: UiOperation[];
  initialTab?: "kitchen" | "export";
}>();

const emit = defineEmits<{ close: [] }>();

const tab = ref<"kitchen" | "export">(props.initialTab ?? "kitchen");

watch(
  () => props.initialTab,
  (next) => {
    if (next) tab.value = next;
  },
);
</script>

<template>
  <Teleport to="body">
    <div class="tools-backdrop" @click.self="emit('close')">
      <aside class="tools-drawer" role="dialog" aria-label="Recipe tools">
        <header class="tools-head">
          <nav class="tools-tabs">
            <button :class="{ active: tab === 'kitchen' }" @click="tab = 'kitchen'">
              <ChefHat :size="15" /> Cook
            </button>
            <button :class="{ active: tab === 'export' }" @click="tab = 'export'">
              <PackageOpen :size="15" /> Export
            </button>
          </nav>
          <button class="icon" title="Close" @click="emit('close')"><X :size="16" /></button>
        </header>
        <div class="tools-body">
          <KitchenModePanel
            v-if="tab === 'kitchen'"
            :recipe-id="recipeId"
            @started="emit('close')"
          />
          <ExportPanel v-else :recipe-id="recipeId" :recipe-title="model.title" />
        </div>
      </aside>
    </div>
  </Teleport>
</template>

<style scoped>
.tools-backdrop {
  position: fixed;
  inset: 0;
  z-index: 40;
  background: rgba(20, 28, 24, 0.35);
}
.tools-drawer {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: min(100%, 480px);
  display: flex;
  flex-direction: column;
  background: #f7f6f2;
  border-left: 1px solid #d3d8d1;
  box-shadow: -12px 0 40px -20px rgba(0, 0, 0, 0.25);
}
.tools-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  background: #fff;
  border-bottom: 1px solid #d8ddd9;
}
.tools-tabs {
  display: flex;
  gap: 4px;
}
.tools-tabs button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 12px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  font-size: 13px;
  color: #55635b;
}
.tools-tabs button.active {
  background: #e4efe6;
  color: #28643b;
}
.tools-head .icon {
  width: 32px;
  height: 32px;
  padding: 0;
  display: grid;
  place-items: center;
}
.tools-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 12px;
}
</style>
