<script setup lang="ts">
/* global KeyboardEvent */
import { computed, onBeforeUnmount, onMounted } from "vue";
import { X } from "lucide-vue-next";
import type { UiRecipeModel } from "../model";
import InspectorPanel, { type InspectorTabId } from "./InspectorPanel.vue";

const props = defineProps<{
  tool: InspectorTabId;
  model: UiRecipeModel;
  recipeId?: string;
  source: string;
}>();

const emit = defineEmits<{
  close: [];
  "update:source": [value: string];
  "kitchen-started": [];
}>();

const toolMeta: Record<InspectorTabId, { group: string; title: string; description: string }> = {
  narrative: {
    group: "Preview",
    title: "Recipe narrative",
    description: "Review the recipe as it will read in print and on the page.",
  },
  outline: {
    group: "Preview",
    title: "Recipe outline",
    description: "Inspect the recipe structure and its declared resources.",
  },
  ingredients: {
    group: "Preview",
    title: "Ingredients",
    description: "Review ingredient quantities, states, and alternatives.",
  },
  author: {
    group: "Author",
    title: "Workflow graph",
    description: "Shape the recipe as a visual sequence of ingredients and operations.",
  },
  timeline: {
    group: "Plan",
    title: "Production timeline",
    description: "See dependencies, active work, and unattended time.",
  },
  formula: {
    group: "Plan",
    title: "Formula editor",
    description: "Scale production and balance baker’s percentages.",
  },
  haccp: {
    group: "Produce",
    title: "Food safety",
    description: "Document hazards, controls, and critical limits.",
  },
  kitchen: {
    group: "Produce",
    title: "Cook mode",
    description: "Run the method with focused steps and timers.",
  },
  nutrition: {
    group: "Produce",
    title: "Nutrition",
    description: "Link ingredients and calculate nutrition facts.",
  },
  export: {
    group: "Share",
    title: "Export recipe",
    description: "Prepare the recipe for print, web, or sharing.",
  },
};

const meta = computed(() => toolMeta[props.tool]);

function onKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape") emit("close");
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <Teleport to="body">
    <div class="tool-dialog-backdrop" @click.self="emit('close')">
      <section
        class="tool-dialog"
        :class="{ 'kitchen-dialog': tool === 'kitchen' }"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="`tool-dialog-${tool}`"
      >
        <header class="tool-dialog-head">
          <div>
            <span>{{ meta.group }}</span>
            <h2 :id="`tool-dialog-${tool}`">{{ meta.title }}</h2>
            <p>{{ meta.description }}</p>
          </div>
          <button autofocus class="close-tool" aria-label="Close tool" @click="emit('close')">
            <X :size="18" />
          </button>
        </header>
        <div class="tool-dialog-body">
          <InspectorPanel
            :model="model"
            :recipe-id="recipeId"
            :source="source"
            :initial-tab="tool"
            :show-navigation="false"
            @update:source="emit('update:source', $event)"
            @kitchen-started="emit('kitchen-started')"
          />
        </div>
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
.tool-dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 80;
  display: grid;
  place-items: center;
  padding: clamp(14px, 3vw, 34px);
  background: rgba(12, 22, 16, 0.56);
  backdrop-filter: blur(3px);
}
.tool-dialog {
  width: min(1180px, 100%);
  height: min(820px, 100%);
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid rgba(255, 255, 255, 0.65);
  border-radius: 15px;
  background: #f6f7f3;
  box-shadow: 0 32px 90px -30px rgba(0, 0, 0, 0.62);
}
.tool-dialog.kitchen-dialog {
  width: min(560px, 100%);
  height: auto;
}
.tool-dialog-head {
  flex: 0 0 auto;
  display: flex;
  justify-content: space-between;
  gap: 20px;
  padding: 17px 20px 15px;
  color: #eef5ef;
  background: linear-gradient(95deg, rgba(200, 223, 183, 0.08), transparent 45%), #1a2b22;
  border-bottom: 1px solid #0f1c15;
}
.tool-dialog-head span {
  display: block;
  margin-bottom: 3px;
  color: #aebfae;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}
.tool-dialog-head h2 {
  margin: 0;
  font-size: 20px;
  letter-spacing: -0.02em;
}
.tool-dialog-head p {
  margin: 4px 0 0;
  color: #bdcbbf;
  font-size: 12px;
}
.close-tool {
  flex: 0 0 34px;
  width: 34px;
  height: 34px;
  padding: 0;
  border-color: rgba(255, 255, 255, 0.16);
  background: rgba(255, 255, 255, 0.08);
  color: white;
}
.close-tool:hover {
  background: rgba(255, 255, 255, 0.16);
}
.tool-dialog-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.tool-dialog-body :deep(.inspector) {
  height: 100%;
  border: 0;
  background: #f6f7f3;
}
@media (max-width: 640px) {
  .tool-dialog-backdrop {
    padding: 0;
  }
  .tool-dialog {
    width: 100%;
    height: 100%;
    border: 0;
    border-radius: 0;
  }
  .tool-dialog-head p {
    display: none;
  }
}
</style>
