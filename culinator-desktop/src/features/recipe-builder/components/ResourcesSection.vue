<script setup lang="ts">
import { ref } from "vue";
import { ChevronsDownUp, ChevronsUpDown, Plus } from "lucide-vue-next";
import type { BuilderResource } from "../composables/useRecipeBuilder";
import ResourceCard from "./ResourceCard.vue";

/**
 * Resources in source order. A flat list (rather than grouped by kind) keeps
 * reordering unambiguous — up/down swaps adjacent declarations — while each
 * card's kind selector still categorises it. Add buttons cover the common
 * kinds; the rarer ones (environment, labor) are reachable from that selector.
 */
defineProps<{
  resources: BuilderResource[];
  disabled?: boolean;
  focusedSymbol?: string | null;
  /** Symbols referenced by steps (or substitutes); others show an Unused badge. */
  usedSymbols?: Set<string>;
}>();

const emit = defineEmits<{
  string: [symbol: string, key: string, value: string];
  quantity: [symbol: string, value: string];
  flag: [symbol: string, key: string, value: boolean];
  kind: [symbol: string, value: string];
  measurement: [symbol: string, value: string];
  substitutes: [symbol: string, value: string[]];
  notes: [symbol: string, value: string[]];
  rename: [symbol: string, value: string];
  duplicate: [symbol: string];
  add: [kind: string];
  addNamed: [name: string];
  remove: [symbol: string];
  move: [symbol: string, direction: "up" | "down"];
  focus: [symbol: string];
}>();

const ADD_KINDS = ["ingredient", "material", "container", "equipment"];
const quickName = ref("");
const expandToken = ref(0);
const collapseToken = ref(0);

function submitQuickAdd(): void {
  const name = quickName.value.trim();
  if (!name) return;
  emit("addNamed", name);
  quickName.value = "";
}

function expandAll(): void {
  expandToken.value += 1;
}

function collapseAll(): void {
  collapseToken.value += 1;
}
</script>

<template>
  <section id="builder-resources" class="panel builder-section">
    <div class="panel-header">
      <h3>Resources</h3>
      <div v-if="resources.length > 1" class="panel-tools">
        <button
          type="button"
          class="ghost"
          :disabled="disabled"
          title="Expand all"
          @click="expandAll"
        >
          <ChevronsUpDown :size="14" /> Expand
        </button>
        <button
          type="button"
          class="ghost"
          :disabled="disabled"
          title="Collapse all"
          @click="collapseAll"
        >
          <ChevronsDownUp :size="14" /> Collapse
        </button>
      </div>
    </div>

    <form class="quick-add" @submit.prevent="submitQuickAdd">
      <input
        v-model="quickName"
        type="text"
        placeholder="Quick-add ingredient (e.g. flour)"
        :disabled="disabled"
        aria-label="Quick-add ingredient name"
      />
      <button type="submit" class="primary" :disabled="disabled || !quickName.trim()">
        <Plus :size="14" /> Add
      </button>
    </form>

    <div v-if="!resources.length" class="empty-cta">
      <p class="empty">No ingredients or equipment yet. Add what the recipe needs on hand.</p>
      <button class="primary" :disabled="disabled" @click="emit('add', 'ingredient')">
        <Plus :size="14" /> Add ingredient
      </button>
    </div>

    <div class="cards">
      <ResourceCard
        v-for="(resource, index) in resources"
        :key="resource.symbol"
        :resource="resource"
        :disabled="disabled"
        :can-move-up="index > 0"
        :can-move-down="index < resources.length - 1"
        :force-expanded="focusedSymbol === resource.symbol"
        :expand-generation="expandToken"
        :collapse-generation="collapseToken"
        :focused="focusedSymbol === resource.symbol"
        :unused="!!usedSymbols && !usedSymbols.has(resource.symbol)"
        @string="(key, value) => emit('string', resource.symbol, key, value)"
        @quantity="(value) => emit('quantity', resource.symbol, value)"
        @flag="(key, value) => emit('flag', resource.symbol, key, value)"
        @kind="(value) => emit('kind', resource.symbol, value)"
        @measurement="(value) => emit('measurement', resource.symbol, value)"
        @substitutes="(value) => emit('substitutes', resource.symbol, value)"
        @notes="(value) => emit('notes', resource.symbol, value)"
        @rename="(value) => emit('rename', resource.symbol, value)"
        @duplicate="emit('duplicate', resource.symbol)"
        @remove="emit('remove', resource.symbol)"
        @move="(direction) => emit('move', resource.symbol, direction)"
        @focus="emit('focus', resource.symbol)"
      />
    </div>

    <div v-if="resources.length" class="add-row">
      <button v-for="kind in ADD_KINDS" :key="kind" :disabled="disabled" @click="emit('add', kind)">
        <Plus :size="14" /> {{ kind }}
      </button>
    </div>
    <div v-else class="add-row">
      <button
        v-for="kind in ADD_KINDS.slice(1)"
        :key="kind"
        :disabled="disabled"
        @click="emit('add', kind)"
      >
        <Plus :size="14" /> {{ kind }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}
.panel-tools {
  display: flex;
  gap: 4px;
}
.panel-tools .ghost {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 4px 8px;
}
.quick-add {
  display: flex;
  gap: 8px;
  margin-bottom: 14px;
}
.quick-add input {
  flex: 1;
  min-width: 0;
}
.quick-add button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  white-space: nowrap;
}
.cards {
  display: grid;
  gap: 12px;
}
.empty-cta {
  display: grid;
  gap: 10px;
  justify-items: start;
  margin-bottom: 12px;
  padding: 14px 16px;
  border: 1px dashed #cfd6d0;
  border-radius: 10px;
  background: #fbfcfa;
}
.empty {
  color: #6d7972;
  font-size: 13px;
  margin: 0;
}
.empty-cta .primary {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.add-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}
.add-row button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 13px;
  text-transform: capitalize;
}
</style>
