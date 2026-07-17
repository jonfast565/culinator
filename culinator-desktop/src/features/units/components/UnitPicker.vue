<script setup lang="ts">
import { computed } from "vue";
import { ALL_UNITS, UNIT_GROUPS, type UnitGroup } from "../unitsCatalog";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    group?: string | null;
    label?: string;
    id?: string;
  }>(),
  { group: null, label: "", id: "" },
);

const emit = defineEmits<{ (event: "update:modelValue", value: string): void }>();

const groups = computed<UnitGroup[]>(() =>
  props.group ? UNIT_GROUPS.filter((entry) => entry.id === props.group) : UNIT_GROUPS,
);

const flatUnits = computed(() =>
  props.group ? groups.value.flatMap((entry) => entry.units) : ALL_UNITS,
);
</script>

<template>
  <label class="unit-picker">
    <span v-if="label">{{ label }}</span>
    <select
      :id="id || undefined"
      :value="modelValue"
      @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
    >
      <template v-if="groups.length > 1">
        <optgroup v-for="entry in groups" :key="entry.id" :label="entry.label">
          <option v-for="unit in entry.units" :key="unit.value" :value="unit.value">
            {{ unit.label }}
          </option>
        </optgroup>
      </template>
      <template v-else>
        <option v-for="unit in flatUnits" :key="unit.value" :value="unit.value">
          {{ unit.label }}
        </option>
      </template>
    </select>
  </label>
</template>

<style scoped>
.unit-picker {
  display: grid;
  gap: 5px;
  margin: 0;
  font-size: 12px;
  color: #657169;
}
.unit-picker select {
  width: 100%;
  height: 38px;
}
</style>
