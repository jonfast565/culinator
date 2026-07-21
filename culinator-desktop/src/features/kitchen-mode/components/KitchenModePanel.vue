<script setup lang="ts">
import { ref } from "vue";
import { Play } from "lucide-vue-next";
import { useKitchenExecution } from "../composables/useKitchenExecution";

const props = defineProps<{ recipeId: string }>();
const emit = defineEmits<{ started: [] }>();
const { error, startTry } = useKitchenExecution(props.recipeId);
const hypothesis = ref("");
const starting = ref(false);

async function handleStartTry(): Promise<void> {
  if (starting.value) return;
  starting.value = true;
  try {
    await startTry("Kitchen cook", hypothesis.value.trim() || undefined);
    emit("started");
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    starting.value = false;
  }
}
</script>

<template>
  <form class="kitchen-start" @submit.prevent="handleStartTry">
    <label for="kitchen-hypothesis">Starting hypothesis</label>
    <textarea
      id="kitchen-hypothesis"
      v-model="hypothesis"
      autofocus
      rows="4"
      placeholder="What are you testing or changing in this cook?"
    />
    <p v-if="error" class="error">{{ error }}</p>
    <button class="primary start-button" type="submit" :disabled="starting">
      <Play :size="16" /> {{ starting ? "Starting…" : "Start cooking" }}
    </button>
  </form>
</template>

<style scoped>
.kitchen-start {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: clamp(20px, 4vw, 36px);
}
.kitchen-start label {
  color: #23302a;
  font-size: 14px;
  font-weight: 700;
}
.kitchen-start textarea {
  min-height: 112px;
  resize: vertical;
  font-size: 15px;
  line-height: 1.5;
}
.start-button {
  align-self: flex-end;
  display: inline-flex;
  align-items: center;
  gap: 7px;
}
.error {
  margin: 0;
  color: #dc2626;
}
</style>
