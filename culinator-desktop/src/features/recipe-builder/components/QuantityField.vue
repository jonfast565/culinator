<script setup lang="ts">
/* global HTMLElement */
import { ref, watch } from "vue";

/**
 * An amount + optional range + unit, committed together as one quantity string.
 *
 * The three parts never round-trip through an invalid intermediate: an amount
 * with no unit parses as a bare number (a different value), so the field only
 * emits when the user finishes editing, and emits the whole `"400 g"` /
 * `"2 to 3 clove"` at once. The unit is free text — the grammar keeps an
 * unrecognised unit verbatim, so a dropdown would be narrower than the language.
 */
const props = defineProps<{ modelValue: string; disabled?: boolean }>();
const emit = defineEmits<{ commit: [value: string] }>();

const amount = ref("");
const max = ref("");
const unit = ref("");
const root = ref<HTMLElement>();

function parse(quantity: string): void {
  const text = quantity.trim();
  if (!text) {
    amount.value = max.value = unit.value = "";
    return;
  }
  const split = (part: string): [string, string] => {
    const match = /^([\d./]+)\s*(.*)$/.exec(part.trim());
    return match ? [match[1], match[2]] : [part.trim(), ""];
  };
  if (/\s+to\s+/.test(text)) {
    const [low, high] = text.split(/\s+to\s+/, 2);
    const [lowAmount, lowUnit] = split(low);
    const [highAmount, highUnit] = split(high);
    amount.value = lowAmount;
    max.value = highAmount;
    unit.value = highUnit || lowUnit;
  } else {
    const [value, valueUnit] = split(text);
    amount.value = value;
    max.value = "";
    unit.value = valueUnit;
  }
}

watch(
  () => props.modelValue,
  (value) => {
    // Never reset the parts while the user is editing one of them.
    if (!root.value?.contains(document.activeElement)) parse(value);
  },
  { immediate: true },
);

function combined(): string {
  const low = amount.value.trim();
  const high = max.value.trim();
  const suffix = unit.value.trim();
  if (!low) return "";
  const head = high ? `${low} to ${high}` : low;
  return suffix ? `${head} ${suffix}` : head;
}

function commit(): void {
  const next = combined();
  if (next !== props.modelValue.trim()) emit("commit", next);
}
</script>

<template>
  <div ref="root" class="quantity-field">
    <label>
      <span>Amount</span>
      <input v-model="amount" :disabled="disabled" inputmode="decimal" @change="commit" />
    </label>
    <label class="to">
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
      <input v-model="unit" :disabled="disabled" placeholder="g, ml, clove…" @change="commit" />
    </label>
  </div>
</template>

<style scoped>
.quantity-field {
  display: grid;
  grid-template-columns: 1fr 0.7fr 1fr;
  gap: 8px;
  align-items: end;
}
.quantity-field label {
  display: grid;
  gap: 5px;
  margin: 0;
  font-size: 12px;
  color: #657169;
}
.quantity-field input {
  width: 100%;
}
.quantity-field .to span {
  text-align: center;
}
</style>
