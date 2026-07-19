<script setup lang="ts">
import { ref, watch } from "vue";
import { useAppDialog } from "../composables/useAppDialog";

const { state, resolve } = useAppDialog();
const inputValue = ref("");

watch(
  () => state.open,
  (open) => {
    if (open && state.kind === "prompt") inputValue.value = state.defaultValue;
  },
);

function confirm(): void {
  if (state.kind === "prompt") resolve(inputValue.value.trim() || null);
  else resolve(true);
}

function cancel(): void {
  resolve(state.kind === "prompt" ? null : false);
}
</script>

<template>
  <Teleport to="body">
    <div v-if="state.open" class="dialog-backdrop" @click.self="cancel">
      <section class="dialog" role="dialog" aria-modal="true" :aria-labelledby="'dialog-title'">
        <h2 id="dialog-title" class="dialog-title">{{ state.title }}</h2>
        <p class="dialog-message">{{ state.message }}</p>
        <input
          v-if="state.kind === 'prompt'"
          v-model="inputValue"
          class="dialog-input"
          type="text"
          autofocus
          @keyup.enter="confirm"
          @keyup.escape="cancel"
        />
        <div class="dialog-actions">
          <button v-if="state.kind !== 'alert'" class="ghost" @click="cancel">
            {{ state.cancelLabel }}
          </button>
          <button class="primary" @click="confirm">{{ state.confirmLabel }}</button>
        </div>
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: grid;
  place-items: center;
  padding: 20px;
  background: rgba(20, 28, 24, 0.45);
}
.dialog {
  width: min(100%, 420px);
  padding: 20px;
  border-radius: 12px;
  background: #fff;
  box-shadow: 0 20px 50px -20px rgba(0, 0, 0, 0.35);
}
.dialog-title {
  margin: 0 0 8px;
  font-size: 17px;
  color: #23302a;
}
.dialog-message {
  margin: 0 0 16px;
  font-size: 14px;
  line-height: 1.45;
  color: #4a5a52;
  white-space: pre-wrap;
}
.dialog-input {
  width: 100%;
  height: 36px;
  margin-bottom: 16px;
  padding: 0 10px;
  border: 1px solid #cbd3cd;
  border-radius: 7px;
  font-size: 14px;
}
.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
.dialog-actions button {
  height: 34px;
  padding: 0 14px;
  font-size: 13px;
}
</style>
