<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { CheckCircle2, Clock3, FlaskConical, Play, SkipForward, Timer } from "lucide-vue-next";
import type { UiOperation } from "../../recipe-editor/model";
import { useKitchenExecution } from "../composables/useKitchenExecution";

const props = defineProps<{
  recipeId: string;
  operations: UiOperation[];
}>();

const {
  tries,
  activeTry,
  error,
  liveTimer,
  nextPendingOperation,
  refresh,
  startTry,
  selectTry,
  saveFindings,
  completeTry,
  startOperation,
  completeOperation,
  skipOperation,
  recordObservation,
  formatClock,
  startClock,
} = useKitchenExecution(props.recipeId);

const tryTitle = ref("Kitchen try");
const tryNotes = ref("");
const findings = ref("");
const observationProperty = ref("note");
const observationValue = ref("");
const observationNotes = ref("");
const selectedTryId = ref<string | null>(null);

function describeOperation(symbol: string): string {
  const operation = props.operations.find((item) => item.symbol === symbol);
  if (!operation) return symbol;
  const mins = operation.durationMinutes;
  return `${operation.action} (${mins} min)`;
}

async function handleStartTry(): Promise<void> {
  try {
    await startTry(tryTitle.value, tryNotes.value || undefined);
    selectedTryId.value = activeTry.value?.id ?? null;
    findings.value = activeTry.value?.findings ?? "";
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

async function handleSaveFindings(): Promise<void> {
  try {
    await saveFindings(findings.value);
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

async function handleObserve(): Promise<void> {
  if (!observationValue.value.trim()) return;
  try {
    await recordObservation({
      operationSymbol:
        liveTimer.value?.operationSymbol ?? nextPendingOperation.value?.operationSymbol,
      propertyPath: observationProperty.value,
      value: observationValue.value,
      notes: observationNotes.value || undefined,
    });
    observationValue.value = "";
    observationNotes.value = "";
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

async function completeActiveStep(): Promise<void> {
  const timer = liveTimer.value;
  if (!timer || !activeTry.value) return;
  const operation = activeTry.value.operations.find(
    (item) => item.operationId === timer.operationId,
  );
  if (!operation) return;
  try {
    await completeOperation(operation);
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
}

onMounted(async () => {
  try {
    await refresh();
    if (activeTry.value) {
      selectedTryId.value = activeTry.value.id;
      findings.value = activeTry.value.findings ?? "";
      if (activeTry.value.status === "active") startClock();
    }
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  }
});

watch(
  () => props.recipeId,
  async () => {
    selectedTryId.value = null;
    await refresh();
  },
);
</script>

<template>
  <section class="panel kitchen-panel">
    <header class="panel-header">
      <div>
        <h3><Timer :size="17" /> Kitchen mode</h3>
        <small>Run recipe tries with live timers and experiment notes.</small>
      </div>
      <button class="primary" @click="handleStartTry"><Play :size="15" /> Start try</button>
    </header>

    <p v-if="error" class="error">{{ error }}</p>

    <div class="try-form">
      <label>Try title<input v-model="tryTitle" placeholder="e.g. Less salt experiment" /></label>
      <label>Hypothesis / setup<textarea v-model="tryNotes" rows="2" /></label>
    </div>

    <label v-if="tries.length">
      Past tries
      <select
        :value="selectedTryId ?? ''"
        @change="selectTry(($event.target as HTMLSelectElement).value)"
      >
        <option disabled value="">Select a try</option>
        <option v-for="item in tries" :key="item.id" :value="item.id">
          {{ item.title || "Untitled try" }} ({{ item.status }})
        </option>
      </select>
    </label>

    <template v-if="activeTry">
      <section v-if="liveTimer" class="timer-card" :class="{ overdue: liveTimer.overdue }">
        <header>
          <Clock3 :size="18" />
          <strong>{{ liveTimer.operationSymbol }}</strong>
        </header>
        <div class="timer-display">
          <div>
            <small>Elapsed</small>
            <span>{{ formatClock(liveTimer.elapsedSeconds) }}</span>
          </div>
          <div>
            <small>Remaining</small>
            <span>{{ formatClock(liveTimer.remainingSeconds) }}</span>
          </div>
        </div>
        <p>{{ describeOperation(liveTimer.operationSymbol) }}</p>
        <button class="primary" @click="completeActiveStep">
          <CheckCircle2 :size="15" /> Complete step
        </button>
      </section>

      <section class="kitchen-section">
        <header><h4>Steps</h4></header>
        <article
          v-for="operation in activeTry.operations"
          :key="operation.operationId"
          class="card"
          :class="operation.status"
        >
          <div class="card-header">
            <strong>{{ operation.operationSymbol }}</strong>
            <small>{{ operation.status }}</small>
          </div>
          <p>{{ describeOperation(operation.operationSymbol) }}</p>
          <small>Planned {{ formatClock(operation.durationSeconds) }}</small>
          <div class="step-actions">
            <button v-if="operation.status === 'pending'" @click="startOperation(operation)">
              <Play :size="14" /> Start timer
            </button>
            <button v-if="operation.status === 'active'" @click="completeOperation(operation)">
              <CheckCircle2 :size="14" /> Done
            </button>
            <button
              v-if="operation.status === 'pending' || operation.status === 'active'"
              @click="skipOperation(operation)"
            >
              <SkipForward :size="14" /> Skip
            </button>
          </div>
        </article>
        <p v-if="!activeTry.operations.length" class="empty">
          Save the recipe first so operations can be tracked.
        </p>
      </section>

      <section class="kitchen-section">
        <header>
          <h4><FlaskConical :size="15" /> Experiment findings</h4>
        </header>
        <label>
          What did you learn?
          <textarea
            v-model="findings"
            rows="4"
            placeholder="Document taste, texture, timing changes, and what to try next."
            @change="handleSaveFindings"
          />
        </label>
        <div class="observation-form">
          <label
            >Observation type<input v-model="observationProperty" placeholder="temperature"
          /></label>
          <label
            >Value<input v-model="observationValue" placeholder="e.g. 74 C or too salty"
          /></label>
          <label>Notes<textarea v-model="observationNotes" rows="2" /></label>
          <button class="primary" @click="handleObserve">Record observation</button>
        </div>
        <article
          v-for="observation in activeTry.observations"
          :key="observation.id"
          class="card observation"
        >
          <strong>{{ observation.propertyPath }}</strong>
          <small>{{ observation.observedAt }}</small>
          <span>{{ observation.value }}</span>
          <p v-if="observation.notes">{{ observation.notes }}</p>
        </article>
      </section>

      <footer class="kitchen-footer">
        <button v-if="activeTry.status === 'active'" class="primary" @click="completeTry">
          <CheckCircle2 :size="15" /> Finish try
        </button>
      </footer>
    </template>

    <p v-else class="empty">
      Start a try to cook with live step timers and document your experiment.
    </p>
  </section>
</template>

<style scoped>
.kitchen-panel {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.try-form,
.observation-form {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.timer-card {
  border: 1px solid var(--border, #d1d5db);
  border-radius: 0.75rem;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.timer-card.overdue {
  border-color: #f59e0b;
  background: #fffbeb;
}
.timer-card header {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}
.timer-display {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.5rem;
}
.timer-display span {
  font-size: 1.4rem;
  font-variant-numeric: tabular-nums;
}
.kitchen-section header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.step-actions {
  display: flex;
  gap: 0.4rem;
  flex-wrap: wrap;
}
.card.active {
  border-color: #2563eb;
}
.card.completed {
  opacity: 0.7;
}
.kitchen-footer {
  display: flex;
  justify-content: flex-end;
}
.error {
  color: #dc2626;
}
</style>
