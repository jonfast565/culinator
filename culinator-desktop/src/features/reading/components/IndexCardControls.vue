<script setup lang="ts">
/* global Event, HTMLSelectElement */
import { computed, inject } from "vue";
import { Printer, Type } from "lucide-vue-next";
import type { IndexCardFormat } from "../indexCardFormat";
import { INDEX_CARD_PICKER_OPTIONS, indexCardFormatLabel } from "../indexCardFormat";
import {
  DEFAULT_INDEX_CARD_MARGIN,
  INDEX_CARD_MARGIN_LABELS,
  INDEX_CARD_MARGIN_NAMES,
} from "../indexCardMargin";
import { printIndexCard } from "../printIndexCard";
import { RECIPE_TYPE_SCALE_LABELS } from "../recipeTypeScale";
import { VIEW_SETTINGS_KEY } from "../composables/useViewSettings";

const props = withDefaults(
  defineProps<{
    /** Show the print button (hidden in kitchen mode). */
    showPrint?: boolean;
    compact?: boolean;
    /** Document title for Save-as-PDF / print job. */
    recipeTitle?: string;
  }>(),
  {
    showPrint: true,
    compact: false,
    recipeTitle: "Recipe",
  },
);

const viewSettings = inject(VIEW_SETTINGS_KEY, null);

const format = computed(() => viewSettings?.indexCardFormat.value ?? "full");
const marginLabel = computed(
  () =>
    INDEX_CARD_MARGIN_LABELS[viewSettings?.indexCardMargin.value ?? DEFAULT_INDEX_CARD_MARGIN],
);
const headerLabel = computed(
  () => RECIPE_TYPE_SCALE_LABELS[viewSettings?.recipeHeaderScale.value ?? "md"],
);
const bodyLabel = computed(
  () => RECIPE_TYPE_SCALE_LABELS[viewSettings?.recipeBodyScale.value ?? "md"],
);
const annotationLabel = computed(
  () => RECIPE_TYPE_SCALE_LABELS[viewSettings?.recipeAnnotationScale.value ?? "md"],
);

function onFormatChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value;
  viewSettings?.setIndexCardFormat(value as IndexCardFormat);
}

function onPrint(): void {
  void printIndexCard({
    format: format.value,
    margin: viewSettings?.indexCardMargin.value ?? DEFAULT_INDEX_CARD_MARGIN,
    title: props.recipeTitle,
  });
}
</script>

<template>
  <div v-if="viewSettings" class="index-card-controls" :class="{ compact }">
    <label class="format-picker">
      <span v-if="!compact" class="picker-label">Card</span>
      <select
        :value="format"
        :title="'Recipe card size'"
        aria-label="Index card size"
        @change="onFormatChange"
      >
        <option v-for="option in INDEX_CARD_PICKER_OPTIONS" :key="option.id" :value="option.id">
          {{ option.label }}
        </option>
      </select>
    </label>

    <div class="type-scales" title="Recipe type sizes and margins">
      <Type :size="12" class="type-icon" aria-hidden="true" />
      <button
        type="button"
        class="scale-btn"
        :title="`Title size: ${headerLabel}`"
        @click="viewSettings.cycleRecipeHeaderScale()"
      >
        Tit {{ headerLabel }}
      </button>
      <button
        type="button"
        class="scale-btn"
        :title="`Body size: ${bodyLabel}`"
        @click="viewSettings.cycleRecipeBodyScale()"
      >
        Body {{ bodyLabel }}
      </button>
      <button
        type="button"
        class="scale-btn"
        :title="`Notes size: ${annotationLabel}`"
        @click="viewSettings.cycleRecipeAnnotationScale()"
      >
        Notes {{ annotationLabel }}
      </button>
      <button
        type="button"
        class="scale-btn"
        :title="`Card margins: ${INDEX_CARD_MARGIN_NAMES[viewSettings.indexCardMargin.value]}`"
        @click="viewSettings.cycleIndexCardMargin()"
      >
        Marg {{ marginLabel }}
      </button>
      <button
        type="button"
        class="scale-btn"
        :title="
          viewSettings.showIndexCardPager.value
            ? 'Hide Card N of M indicator'
            : 'Show Card N of M indicator'
        "
        :aria-pressed="viewSettings.showIndexCardPager.value"
        @click="viewSettings.toggleIndexCardPager()"
      >
        #{{ viewSettings.showIndexCardPager.value ? "On" : "Off" }}
      </button>
    </div>

    <button
      v-if="showPrint"
      type="button"
      class="print-btn"
      :title="format === 'full' ? 'Print recipe' : `Print ${indexCardFormatLabel(format)}`"
      @click="onPrint"
    >
      <Printer :size="14" />
      <span v-if="!compact">Print</span>
    </button>
  </div>
</template>

<style scoped>
.index-card-controls {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px;
  border-radius: 8px;
  background: #eceee9;
}
.index-card-controls.compact {
  gap: 2px;
}
.format-picker {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.picker-label {
  padding-left: 6px;
  font-size: 11px;
  font-weight: 600;
  color: #55635b;
  white-space: nowrap;
}
.format-picker select {
  height: 28px;
  max-width: 11rem;
  padding: 0 22px 0 8px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: #23302a;
  font-size: 12px;
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6' viewBox='0 0 10 6'%3E%3Cpath fill='%2355635b' d='M1 1l4 4 4-4'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 7px center;
}
.format-picker select:hover,
.format-picker select:focus {
  background-color: #e4efe6;
  outline: none;
}
.type-scales {
  display: inline-flex;
  align-items: center;
  gap: 1px;
  margin-left: 2px;
  padding-left: 4px;
  border-left: 1px solid #d0d5ce;
}
.type-icon {
  margin: 0 2px 0 2px;
  color: #718078;
}
.scale-btn {
  height: 28px;
  padding: 0 6px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: #45524b;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.02em;
  white-space: nowrap;
  cursor: pointer;
}
.scale-btn:hover {
  background: #e4efe6;
  color: #28643b;
}
.print-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 28px;
  padding: 0 8px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: #45524b;
  font-size: 12px;
  cursor: pointer;
}
.print-btn:hover {
  background: #e4efe6;
  color: #28643b;
}
.compact .format-picker select {
  max-width: 7.5rem;
  font-size: 11px;
}
.compact .scale-btn {
  padding: 0 4px;
  font-size: 9px;
}
.compact .type-icon {
  display: none;
}
.compact .print-btn {
  width: 28px;
  padding: 0;
  justify-content: center;
}
</style>
