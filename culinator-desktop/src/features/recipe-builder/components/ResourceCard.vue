<script setup lang="ts">
/* global Event, HTMLSelectElement */
import { computed } from "vue";
import { ChevronDown, ChevronUp, Copy, Trash2 } from "lucide-vue-next";
import type { BuilderResource } from "../composables/useRecipeBuilder";
import BuilderTextField from "./BuilderTextField.vue";
import QuantityField from "./QuantityField.vue";
import NoteList from "./NoteList.vue";

const props = defineProps<{
  resource: BuilderResource;
  disabled?: boolean;
  canMoveUp?: boolean;
  canMoveDown?: boolean;
}>();

const emit = defineEmits<{
  string: [key: string, value: string];
  quantity: [value: string];
  flag: [key: string, value: boolean];
  kind: [value: string];
  measurement: [value: string];
  substitutes: [value: string[]];
  notes: [value: string[]];
  rename: [value: string];
  duplicate: [];
  remove: [];
  move: [direction: "up" | "down"];
}>();

const KINDS = ["ingredient", "material", "container", "equipment", "environment", "labor"];
const DIMENSIONS = [
  "mass",
  "volume",
  "count",
  "time",
  "temperature",
  "length",
  "area",
  "energy",
  "ratio",
  "concentration",
];

const title = computed(() => props.resource.name || props.resource.symbol);
const substitutesText = computed(() => props.resource.substitutes.join(", "));

function onKind(event: Event): void {
  emit("kind", (event.target as HTMLSelectElement).value);
}
function onMeasurement(event: Event): void {
  emit("measurement", (event.target as HTMLSelectElement).value);
}
function commitSubstitutes(value: string): void {
  emit(
    "substitutes",
    value
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean),
  );
}
</script>

<template>
  <article class="card resource-card" :class="{ disabled }">
    <header class="card-head">
      <select
        class="kind-select"
        :value="resource.kind"
        :disabled="disabled"
        aria-label="Resource kind"
        @change="onKind"
      >
        <option v-for="option in KINDS" :key="option" :value="option">{{ option }}</option>
      </select>
      <strong class="card-title">{{ title }}</strong>
      <div class="card-tools">
        <button
          class="icon"
          title="Move up"
          :disabled="disabled || !canMoveUp"
          @click="emit('move', 'up')"
        >
          <ChevronUp :size="15" />
        </button>
        <button
          class="icon"
          title="Move down"
          :disabled="disabled || !canMoveDown"
          @click="emit('move', 'down')"
        >
          <ChevronDown :size="15" />
        </button>
        <button class="icon" title="Duplicate" :disabled="disabled" @click="emit('duplicate')">
          <Copy :size="14" />
        </button>
        <button
          class="icon danger"
          title="Remove resource"
          :disabled="disabled"
          @click="emit('remove')"
        >
          <Trash2 :size="15" />
        </button>
      </div>
    </header>

    <BuilderTextField
      label="Name"
      :model-value="resource.name"
      :disabled="disabled"
      @commit="emit('string', 'name', $event)"
    />

    <div class="measure-row">
      <label class="select-field">
        <span>Measured by</span>
        <select :value="resource.measurement" :disabled="disabled" @change="onMeasurement">
          <option value="">—</option>
          <option v-for="dimension in DIMENSIONS" :key="dimension" :value="dimension">
            {{ dimension }}
          </option>
        </select>
      </label>
      <div class="quantity-slot">
        <QuantityField
          v-if="!resource.divided"
          :model-value="resource.quantity"
          :disabled="disabled"
          @commit="emit('quantity', $event)"
        />
        <p v-else class="divided-note">Amounts are set on each step that uses this ingredient.</p>
      </div>
    </div>

    <div class="flags">
      <label
        ><input
          type="checkbox"
          :checked="resource.optional"
          :disabled="disabled"
          @change="emit('flag', 'optional', ($event.target as HTMLInputElement).checked)"
        />
        Optional</label
      >
      <label
        ><input
          type="checkbox"
          :checked="resource.divided"
          :disabled="disabled"
          @change="emit('flag', 'divided', ($event.target as HTMLInputElement).checked)"
        />
        Divided</label
      >
      <label
        ><input
          type="checkbox"
          :checked="resource.toTaste"
          :disabled="disabled"
          @change="emit('flag', 'to_taste', ($event.target as HTMLInputElement).checked)"
        />
        To taste</label
      >
    </div>

    <details class="more">
      <summary>More options</summary>
      <div class="more-fields">
        <div class="triple">
          <BuilderTextField
            label="State"
            :model-value="resource.state"
            placeholder="e.g. grated"
            :disabled="disabled"
            @commit="emit('string', 'state', $event)"
          />
          <BuilderTextField
            label="Size"
            :model-value="resource.size"
            placeholder="e.g. medium"
            :disabled="disabled"
            @commit="emit('string', 'size', $event)"
          />
          <BuilderTextField
            label="Variant"
            :model-value="resource.variant"
            placeholder="e.g. sweet"
            :disabled="disabled"
            @commit="emit('string', 'variant', $event)"
          />
        </div>
        <BuilderTextField
          label="Substitutes (comma-separated)"
          :model-value="substitutesText"
          placeholder="e.g. butter, margarine"
          :disabled="disabled"
          @commit="commitSubstitutes"
        />
        <NoteList
          label="Notes"
          :notes="resource.notes"
          :disabled="disabled"
          @commit="emit('notes', $event)"
        />
        <BuilderTextField
          label="Identifier (used in steps)"
          :model-value="resource.symbol"
          :disabled="disabled"
          @commit="emit('rename', $event)"
        />
      </div>
    </details>

    <div v-if="resource.unknown.length" class="chips">
      <span class="chips-label">Kept as written:</span>
      <span v-for="(item, index) in resource.unknown" :key="index" class="chip">
        {{ item.keyword }} {{ item.text }}
      </span>
    </div>
  </article>
</template>

<style scoped>
.resource-card {
  display: grid;
  gap: 12px;
  padding: 14px;
}
.resource-card.disabled {
  opacity: 0.6;
}
.card-head {
  display: flex;
  align-items: center;
  gap: 10px;
}
.kind-select {
  width: auto;
  font-size: 12px;
  padding: 4px 8px;
  color: #45524b;
  text-transform: capitalize;
}
.card-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 15px;
}
.card-tools {
  display: flex;
  gap: 4px;
}
.icon {
  width: 30px;
  height: 30px;
  padding: 0;
  display: grid;
  place-items: center;
}
.icon.danger {
  color: #a83737;
}
.measure-row {
  display: grid;
  grid-template-columns: 150px 1fr;
  gap: 12px;
  align-items: end;
}
.select-field {
  display: grid;
  gap: 5px;
  margin: 0;
  font-size: 12px;
  color: #657169;
}
.divided-note {
  margin: 0;
  font-size: 12px;
  color: #8a938c;
  align-self: center;
}
.flags {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
}
.flags label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin: 0;
  font-size: 13px;
  color: #3d4842;
}
.flags input {
  width: auto;
}
.more summary {
  cursor: pointer;
  font-size: 12px;
  color: #55635b;
  user-select: none;
}
.more-fields {
  display: grid;
  gap: 12px;
  margin-top: 10px;
}
.triple {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 10px;
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
  padding-top: 4px;
  border-top: 1px dashed #dfe3de;
}
.chips-label {
  font-size: 11px;
  color: #8a938c;
}
.chip {
  font-size: 11px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  padding: 2px 8px;
  border-radius: 10px;
  background: #eef1ec;
  color: #55635b;
}
@media (max-width: 620px) {
  .measure-row,
  .triple {
    grid-template-columns: 1fr;
  }
}
</style>
