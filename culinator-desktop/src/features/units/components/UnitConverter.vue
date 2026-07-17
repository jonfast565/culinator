<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ArrowRightLeft } from "lucide-vue-next";
import type { UnitSystem } from "../../../domain/types";
import { convertUnits, formatUnit } from "../../../services/api/units-api";
import { quantityDimension } from "../quantityConvert";
import UnitPicker from "./UnitPicker.vue";

const props = withDefaults(
  defineProps<{
    unitSystem?: UnitSystem;
    compact?: boolean;
  }>(),
  { unitSystem: "metric", compact: false },
);

const value = ref(1);
const fromUnit = ref("cup");
const toUnit = ref("ml");
const converted = ref("");
const dimension = ref("");
const error = ref("");

const sharedDimension = computed(() => {
  const from = quantityDimension(fromUnit.value);
  const to = quantityDimension(toUnit.value);
  if (from && to && from === to) return from;
  return null;
});

function swapUnits(): void {
  const previousFrom = fromUnit.value;
  fromUnit.value = toUnit.value;
  toUnit.value = previousFrom;
}

async function run(): Promise<void> {
  error.value = "";
  converted.value = "";
  dimension.value = "";
  if (!Number.isFinite(value.value)) {
    error.value = "Enter a valid number.";
    return;
  }
  try {
    const result = await convertUnits({
      value: value.value,
      fromUnit: fromUnit.value,
      toUnit: toUnit.value,
    });
    if (result.dimension === "unknown") {
      error.value = "These units cannot be converted directly.";
      return;
    }
    dimension.value = result.dimension;
    const formatted = await formatUnit({
      value: result.value,
      unit: result.unit,
      unitSystem: props.unitSystem ?? "metric",
    });
    converted.value = formatted.formatted;
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

watch([value, fromUnit, toUnit, () => props.unitSystem], () => void run(), { immediate: true });
</script>

<template>
  <section class="unit-converter panel" :class="{ compact }">
    <header class="panel-header">
      <h3><ArrowRightLeft :size="16" /> Unit converter</h3>
      <span v-if="sharedDimension" class="dimension-tag">{{ sharedDimension }}</span>
    </header>

    <div class="converter-grid">
      <label class="value-field">
        <span>Amount</span>
        <input v-model.number="value" type="number" step="any" min="0" />
      </label>
      <UnitPicker v-model="fromUnit" label="From" />
      <button class="swap" type="button" title="Swap units" @click="swapUnits">⇄</button>
      <UnitPicker v-model="toUnit" label="To" />
    </div>

    <p v-if="converted" class="result">{{ converted }}</p>
    <p v-if="error" class="error">{{ error }}</p>
  </section>
</template>

<style scoped>
.unit-converter {
  display: grid;
  gap: 14px;
}
.unit-converter.compact {
  padding: 0;
}
.panel-header {
  margin-bottom: 0;
}
.dimension-tag {
  align-self: center;
  padding: 3px 9px;
  border-radius: 999px;
  background: #e8f0e6;
  color: #28643b;
  font-size: 11px;
  font-weight: 600;
  text-transform: capitalize;
}
.converter-grid {
  display: grid;
  grid-template-columns: minmax(90px, 0.8fr) minmax(120px, 1fr) auto minmax(120px, 1fr);
  gap: 10px;
  align-items: end;
}
.compact .converter-grid {
  grid-template-columns: 1fr;
}
.value-field {
  display: grid;
  gap: 5px;
  margin: 0;
  font-size: 12px;
  color: #657169;
}
.value-field input {
  width: 100%;
  height: 38px;
}
.swap {
  width: 38px;
  height: 38px;
  padding: 0;
  border-radius: 999px;
  font-size: 16px;
}
.compact .swap {
  justify-self: start;
}
.result {
  margin: 0;
  padding: 12px 14px;
  border-radius: 10px;
  background: #f3f7f2;
  font-size: 22px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
.error {
  margin: 0;
  color: #b42318;
  font-size: 13px;
}
@media (max-width: 720px) {
  .converter-grid {
    grid-template-columns: 1fr 1fr;
  }
  .swap {
    grid-column: span 2;
  }
}
</style>
