<script setup lang="ts">
import { Plus } from "lucide-vue-next";
import type { BuilderResource } from "../composables/useRecipeBuilder";
import ResourceCard from "./ResourceCard.vue";

/**
 * Resources in source order. A flat list (rather than grouped by kind) keeps
 * reordering unambiguous — up/down swaps adjacent declarations — while each
 * card's kind selector still categorises it. Add buttons cover the common
 * kinds; the rarer ones (environment, labor) are reachable from that selector.
 */
defineProps<{ resources: BuilderResource[]; disabled?: boolean }>();

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
  remove: [symbol: string];
  move: [symbol: string, direction: "up" | "down"];
}>();

const ADD_KINDS = ["ingredient", "material", "container", "equipment"];
</script>

<template>
  <section id="builder-resources" class="panel builder-section">
    <div class="panel-header">
      <h3>Resources</h3>
    </div>

    <p v-if="!resources.length" class="empty">
      No ingredients or equipment yet. Add one to get started.
    </p>

    <div class="cards">
      <ResourceCard
        v-for="(resource, index) in resources"
        :key="resource.symbol"
        :resource="resource"
        :disabled="disabled"
        :can-move-up="index > 0"
        :can-move-down="index < resources.length - 1"
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
      />
    </div>

    <div class="add-row">
      <button v-for="kind in ADD_KINDS" :key="kind" :disabled="disabled" @click="emit('add', kind)">
        <Plus :size="14" /> {{ kind }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.cards {
  display: grid;
  gap: 12px;
}
.empty {
  color: #8a938c;
  font-size: 13px;
  margin: 0 0 12px;
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
