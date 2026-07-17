<script setup lang="ts">
import { ref, watch } from "vue";
import type { UnitSystem } from "../../../domain/types";
import { convertUnits, formatUnit } from "../../../services/api/units-api";

const props = defineProps<{ unitSystem?: UnitSystem }>();

const value = ref(100);
const fromUnit = ref("g");
const toUnit = ref("oz");
const converted = ref("");
const error = ref("");

async function run(): Promise<void> {
  error.value = "";
  try {
    const result = await convertUnits({
      value: value.value,
      fromUnit: fromUnit.value,
      toUnit: toUnit.value,
    });
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
  <section class="unit-converter panel">
    <h3>Unit converter</h3>
    <div class="row">
      <input v-model.number="value" type="number" step="any" />
      <input v-model="fromUnit" placeholder="from" />
      <span>→</span>
      <input v-model="toUnit" placeholder="to" />
    </div>
    <p v-if="converted" class="result">{{ converted }}</p>
    <p v-if="error" class="error">{{ error }}</p>
  </section>
</template>

<style scoped>
.unit-converter {
  display: grid;
  gap: 8px;
}
.row {
  display: grid;
  grid-template-columns: 1fr 80px auto 80px;
  gap: 8px;
  align-items: center;
}
input {
  height: 34px;
  border: 1px solid #cbd3cd;
  border-radius: 8px;
  padding: 0 8px;
}
.result {
  margin: 0;
  font-weight: 600;
}
.error {
  margin: 0;
  color: #b42318;
  font-size: 13px;
}
</style>
