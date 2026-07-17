<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { AlertCircle, CheckCircle2, Database, Loader2, UtensilsCrossed } from "lucide-vue-next";
import type { InitPhase } from "../../services/api/init-api";
import { fetchInitStatus, runInitialization } from "../../services/api/init-api";
import { onConnectionStatus } from "../../services/transport/websocket-client";

const emit = defineEmits<{ ready: [] }>();

const phase = ref<InitPhase>("connecting");
const detail = ref("Waiting for local service…");
const error = ref("");
const retrying = ref(false);

const steps = computed(() => [
  { id: "connecting", label: "Connect" },
  { id: "initializing_catalog", label: "Catalog" },
  { id: "loading_nutrition", label: "Nutrition" },
  { id: "seeding_samples", label: "Samples" },
  { id: "ready", label: "Ready" },
]);

function stepState(id: string): "done" | "active" | "pending" | "failed" {
  const order = steps.value.map((step) => step.id);
  const current = order.indexOf(phase.value);
  const index = order.indexOf(id);
  if (phase.value === "failed") return index <= current ? "failed" : "pending";
  if (index < current) return "done";
  if (index === current) return "active";
  return "pending";
}

async function bootstrap(): Promise<void> {
  error.value = "";
  try {
    phase.value = "connecting";
    detail.value = "Connected to Culinator service";
    phase.value = "initializing_catalog";
    detail.value = "Checking recipe catalog…";
    const status = await fetchInitStatus();
    phase.value = "loading_nutrition";
    detail.value = status.nutritionReady
      ? "Nutrition dictionary loaded"
      : "Preparing nutrition dictionary…";
    phase.value = "seeding_samples";
    detail.value = status.recipesSeeded
      ? `${status.recipeCount} recipes in library`
      : "Adding sample recipes…";
    const report = await runInitialization();
    phase.value = "ready";
    detail.value = report.nutritionStarter
      ? `Ready — starter nutrition catalog (${report.recipeCount} recipes)`
      : `Ready — ${report.recipeCount} recipes`;
    window.setTimeout(() => emit("ready"), 350);
  } catch (cause) {
    phase.value = "failed";
    error.value = cause instanceof Error ? cause.message : String(cause);
    detail.value = "Initialization failed";
  }
}

async function retry(): Promise<void> {
  retrying.value = true;
  phase.value = "connecting";
  await bootstrap();
  retrying.value = false;
}

onMounted(() => {
  const stop = onConnectionStatus((status) => {
    if (status === "connected" && phase.value === "connecting") void bootstrap();
    if (status === "disconnected" && phase.value !== "ready") {
      detail.value = "Reconnecting to local service…";
    }
  });
  const onReady = (): void => {
    void bootstrap();
  };
  window.addEventListener("culinator:service-ready", onReady, { once: true });
  if (window.__CULINATOR_SERVICE__) void bootstrap();
  onBeforeUnmount(() => {
    stop();
    window.removeEventListener("culinator:service-ready", onReady);
  });
});
</script>

<template>
  <div class="init-screen">
    <div class="init-card">
      <header class="init-brand">
        <span class="init-mark"><UtensilsCrossed :size="22" /></span>
        <div>
          <strong>Culinator</strong>
          <small>Preparing your kitchen</small>
        </div>
      </header>

      <ol class="init-steps">
        <li v-for="step in steps" :key="step.id" :class="stepState(step.id)">
          <span class="dot">
            <CheckCircle2 v-if="stepState(step.id) === 'done'" :size="16" />
            <Loader2 v-else-if="stepState(step.id) === 'active'" :size="16" class="spin" />
            <AlertCircle v-else-if="stepState(step.id) === 'failed'" :size="16" />
          </span>
          {{ step.label }}
        </li>
      </ol>

      <p class="init-detail"><Database :size="14" /> {{ detail }}</p>
      <p v-if="error" class="init-error">{{ error }}</p>
      <button v-if="phase === 'failed'" class="primary" :disabled="retrying" @click="retry">
        {{ retrying ? "Retrying…" : "Retry" }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.init-screen {
  min-height: 100%;
  display: grid;
  place-items: center;
  padding: 24px;
  background: radial-gradient(120% 80% at 50% -10%, #efece2 0%, #e7e3d6 55%, #ddd7c6 100%);
}
.init-card {
  width: min(420px, 100%);
  padding: 28px 26px;
  border-radius: 16px;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid #d8ddd9;
  box-shadow: 0 18px 40px -24px rgba(35, 48, 42, 0.45);
}
.init-brand {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 22px;
}
.init-mark {
  display: grid;
  place-items: center;
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: #d9f0df;
  color: #1f5130;
}
.init-brand strong {
  display: block;
  font-size: 18px;
}
.init-brand small {
  color: #6d7972;
}
.init-steps {
  list-style: none;
  margin: 0 0 18px;
  padding: 0;
  display: grid;
  gap: 10px;
}
.init-steps li {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  color: #6d7972;
}
.init-steps li.active {
  color: #23302a;
  font-weight: 600;
}
.init-steps li.done {
  color: #28643b;
}
.init-steps li.failed {
  color: #b42318;
}
.dot {
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
}
.init-detail {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-size: 13px;
  color: #55635b;
}
.init-error {
  margin: 10px 0 0;
  color: #b42318;
  font-size: 13px;
}
button.primary {
  margin-top: 14px;
  width: 100%;
  height: 38px;
}
.spin {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
