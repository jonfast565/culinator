<script setup lang="ts">
/* global Event, HTMLSelectElement */
import { Plus, Trash2 } from "lucide-vue-next";
import type { BuilderYield } from "../composables/useRecipeBuilder";
import BuilderTextField from "./BuilderTextField.vue";

/** Yields and servings — how much the recipe makes. */
defineProps<{ yields: BuilderYield[]; disabled?: boolean }>();

const emit = defineEmits<{
  amount: [symbol: string, value: string];
  measurement: [symbol: string, value: string];
  add: [keyword: string];
  remove: [symbol: string];
}>();

const DIMENSIONS = ["count", "mass", "volume", "length", "area", "energy"];

function onMeasurement(symbol: string, event: Event): void {
  emit("measurement", symbol, (event.target as HTMLSelectElement).value);
}
</script>

<template>
  <section id="builder-yields" class="panel builder-section">
    <div class="panel-header">
      <h3>Yield</h3>
    </div>

    <p v-if="!yields.length" class="empty">No yield declared. Say how much the recipe makes.</p>

    <div class="rows">
      <div v-for="item in yields" :key="item.symbol" class="yield-row card">
        <span class="badge">{{ item.keyword }}</span>
        <strong class="yield-name">{{ item.symbol.replace(/_/g, " ") }}</strong>
        <label class="select-field">
          <span>Measured by</span>
          <select
            :value="item.measurement"
            :disabled="disabled"
            @change="onMeasurement(item.symbol, $event)"
          >
            <option value="">—</option>
            <option v-for="dimension in DIMENSIONS" :key="dimension" :value="dimension">
              {{ dimension }}
            </option>
          </select>
        </label>
        <BuilderTextField
          label="Amount"
          :model-value="item.amount"
          placeholder="e.g. 4 count"
          :disabled="disabled"
          @commit="emit('amount', item.symbol, $event)"
        />
        <button
          class="icon danger"
          title="Remove"
          :disabled="disabled"
          @click="emit('remove', item.symbol)"
        >
          <Trash2 :size="15" />
        </button>
      </div>
    </div>

    <div class="add-row">
      <button :disabled="disabled" @click="emit('add', 'yield')"><Plus :size="14" /> Yield</button>
      <button :disabled="disabled" @click="emit('add', 'serving')">
        <Plus :size="14" /> Serving
      </button>
    </div>
  </section>
</template>

<style scoped>
.empty {
  color: #8a938c;
  font-size: 13px;
  margin: 0 0 12px;
}
.rows {
  display: grid;
  gap: 10px;
}
.yield-row {
  display: grid;
  grid-template-columns: auto 1fr 150px 1fr auto;
  gap: 12px;
  align-items: end;
  padding: 12px 14px;
}
.badge {
  align-self: center;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #55635b;
  background: #eef1ec;
  padding: 3px 8px;
  border-radius: 10px;
}
.yield-name {
  align-self: center;
  text-transform: capitalize;
  font-size: 14px;
}
.select-field {
  display: grid;
  gap: 5px;
  margin: 0;
  font-size: 12px;
  color: #657169;
}
.icon {
  width: 32px;
  height: 34px;
  padding: 0;
  display: grid;
  place-items: center;
}
.icon.danger {
  color: #a83737;
}
.add-row {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}
.add-row button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 13px;
}
@media (max-width: 720px) {
  .yield-row {
    grid-template-columns: 1fr 1fr;
  }
}
</style>
