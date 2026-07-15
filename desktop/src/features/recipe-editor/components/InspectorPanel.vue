<script setup lang="ts">
import { computed, ref } from "vue";
import type { ValidationResult } from "../../../domain/types";
import type { UiRecipeModel } from "../model";
import FormulaCalculator from "../../formulas/components/FormulaCalculator.vue";
import ExportPanel from "../../export/components/ExportPanel.vue";
import VisualAuthoringPanel from "../../visual-authoring/components/VisualAuthoringPanel.vue";
import GanttSchedule from "../../scheduling/components/GanttSchedule.vue";
const props = defineProps<{
  model: UiRecipeModel;
  validation: ValidationResult | null;
  recipeId?: string;
  source: string;
}>();
const emit = defineEmits<{ "update:source": [value: string] }>();
const tab = ref<
  "outline" | "ingredients" | "author" | "timeline" | "formula" | "export" | "diagnostics"
>("outline");
const operations = computed(() => props.model.operations ?? []);
</script>
<template>
  <aside class="inspector">
    <nav class="tabs">
      <button
        v-for="item in [
          'outline',
          'ingredients',
          'author',
          'timeline',
          'formula',
          'export',
          'diagnostics',
        ]"
        :key="item"
        :class="{ active: tab === item }"
        @click="tab = item as typeof tab"
      >
        {{ item }}
      </button>
    </nav>
    <section v-if="tab === 'outline'" class="panel">
      <h3>{{ model.title || "Untitled recipe" }}</h3>
      <dl>
        <div>
          <dt>Symbol</dt>
          <dd>{{ model.symbol }}</dd>
        </div>
        <div>
          <dt>Resources</dt>
          <dd>{{ model.resources.length }}</dd>
        </div>
        <div>
          <dt>Processes</dt>
          <dd>{{ model.processes.length }}</dd>
        </div>
        <div>
          <dt>Operations</dt>
          <dd>{{ operations.length }}</dd>
        </div>
      </dl>
    </section>
    <section v-else-if="tab === 'ingredients'" class="panel">
      <h3>Resources</h3>
      <article v-for="resource in model.resources" :key="resource.symbol" class="card">
        <strong>{{ resource.name || resource.symbol }}</strong
        ><small>{{ resource.kind }} · {{ resource.measurement || "untyped" }}</small
        ><span v-if="resource.quantity">{{ resource.quantity }}</span>
      </article>
    </section>
    <VisualAuthoringPanel
      v-else-if="tab === 'author'"
      :source="source"
      :model="model"
      @update:source="emit('update:source', $event)"
    />
    <GanttSchedule v-else-if="tab === 'timeline'" :source="source" />
    <FormulaCalculator v-else-if="tab === 'formula' && recipeId" :recipe-id="recipeId" />
    <ExportPanel
      v-else-if="tab === 'export' && recipeId"
      :recipe-id="recipeId"
      :recipe-title="model.title"
    />
    <section v-else class="panel">
      <h3>Diagnostics</h3>
      <p v-if="!validation?.diagnostics.length" class="empty">No diagnostics.</p>
      <article
        v-for="item in validation?.diagnostics"
        :key="item.message"
        class="diagnostic"
        :class="item.severity"
      >
        <strong>{{ item.severity }}</strong
        ><span>{{ item.message }}</span>
      </article>
    </section>
  </aside>
</template>
