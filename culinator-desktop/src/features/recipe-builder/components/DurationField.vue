<script setup lang="ts">
/* global HTMLElement */
import { ref, watch } from "vue";

/**
 * A step duration: an amount and unit, an optional range maximum, and an
 * `estimated` / `up to` qualifier. Parsed from and emitted as the verbatim
 * `duration` value, so `9 minutes to 10 minutes` or `estimated 15 min` survive.
 */
const props = defineProps<{ modelValue: string; disabled?: boolean }>();
const emit = defineEmits<{ commit: [value: string] }>();

type Qualifier = "exact" | "estimated" | "up to";
const qualifier = ref<Qualifier>("exact");
const amount = ref("");
const unit = ref("");
const max = ref("");
const root = ref<HTMLElement>();

function parse(text: string): void {
  let rest = text.trim();
  qualifier.value = "exact";
  if (/^estimated\s+/.test(rest)) {
    qualifier.value = "estimated";
    rest = rest.replace(/^estimated\s+/, "");
  } else if (/^up\s+to\s+/.test(rest)) {
    qualifier.value = "up to";
    rest = rest.replace(/^up\s+to\s+/, "");
  }
  const split = (part: string): [string, string] => {
    const match = /^([\d./]+)\s*(.*)$/.exec(part.trim());
    return match ? [match[1], match[2]] : [part.trim(), ""];
  };
  if (/\s+to\s+/.test(rest)) {
    const [low, high] = rest.split(/\s+to\s+/, 2);
    const [lowAmount, lowUnit] = split(low);
    const [highAmount, highUnit] = split(high);
    amount.value = lowAmount;
    max.value = highAmount;
    unit.value = highUnit || lowUnit;
  } else {
    const [value, valueUnit] = split(rest);
    amount.value = value;
    max.value = "";
    unit.value = valueUnit;
  }
}

watch(
  () => props.modelValue,
  (value) => {
    if (!root.value?.contains(document.activeElement)) parse(value);
  },
  { immediate: true },
);

function combined(): string {
  const low = amount.value.trim();
  const high = max.value.trim();
  const suffix = unit.value.trim();
  if (!low) return "";
  const one = suffix ? `${low} ${suffix}` : low;
  if (qualifier.value === "estimated") return `estimated ${one}`;
  if (qualifier.value === "up to") return `up to ${one}`;
  if (high) return suffix ? `${low} ${suffix} to ${high} ${suffix}` : `${low} to ${high}`;
  return one;
}

function commit(): void {
  const next = combined();
  if (next !== props.modelValue.trim()) emit("commit", next);
}
</script>

<template>
  <div ref="root" class="duration-field">
    <label>
      <span>Duration</span>
      <select v-model="qualifier" :disabled="disabled" @change="commit">
        <option value="exact">exactly</option>
        <option value="estimated">estimated</option>
        <option value="up to">up to</option>
      </select>
    </label>
    <label>
      <span>Amount</span>
      <input v-model="amount" :disabled="disabled" inputmode="decimal" @change="commit" />
    </label>
    <label v-if="qualifier === 'exact'" class="to">
      <span>to</span>
      <input
        v-model="max"
        :disabled="disabled"
        inputmode="decimal"
        placeholder="—"
        @change="commit"
      />
    </label>
    <label>
      <span>Unit</span>
      <input v-model="unit" :disabled="disabled" placeholder="min" @change="commit" />
    </label>
  </div>
</template>

<style scoped>
.duration-field {
  display: grid;
  grid-template-columns: auto 1fr 0.7fr 1fr;
  gap: 8px;
  align-items: end;
}
.duration-field label {
  display: grid;
  gap: 5px;
  margin: 0;
  font-size: 12px;
  color: #657169;
}
.duration-field input,
.duration-field select {
  width: 100%;
}
.duration-field .to span {
  text-align: center;
}
</style>
