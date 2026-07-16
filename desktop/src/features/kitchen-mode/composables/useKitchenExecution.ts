import { computed, onUnmounted, ref } from "vue";
import type { RecipeTryDocument, TryOperation } from "../../../domain/types";
import * as api from "../../../services/api";

export interface LiveTimerState {
  operationId: string;
  operationSymbol: string;
  elapsedSeconds: number;
  remainingSeconds: number;
  durationSeconds: number;
  overdue: boolean;
}

export function useKitchenExecution(recipeId: string) {
  const tries = ref<RecipeTryDocument[]>([]);
  const activeTry = ref<RecipeTryDocument | null>(null);
  const error = ref("");
  const now = ref(Date.now());
  let timerHandle: number | undefined;

  function startClock(): void {
    if (timerHandle) return;
    timerHandle = window.setInterval(() => {
      now.value = Date.now();
    }, 1000);
  }

  function stopClock(): void {
    if (timerHandle) window.clearInterval(timerHandle);
    timerHandle = undefined;
  }

  async function refresh(): Promise<void> {
    const summaries = await api.listRecipeTries(recipeId);
    tries.value = await Promise.all(summaries.map((item) => api.getRecipeTry(item.id)));
    if (!activeTry.value && tries.value[0]?.status === "active") {
      activeTry.value = tries.value[0];
    }
  }

  async function startTry(title?: string, notes?: string): Promise<void> {
    error.value = "";
    const created = await api.startRecipeTry(recipeId, { title, notes });
    activeTry.value = created;
    await refresh();
    startClock();
  }

  async function selectTry(tryId: string): Promise<void> {
    activeTry.value = await api.getRecipeTry(tryId);
    if (activeTry.value.status === "active") startClock();
    else stopClock();
  }

  async function saveFindings(findings: string): Promise<void> {
    if (!activeTry.value) return;
    activeTry.value = await api.updateRecipeTry(activeTry.value.id, { findings });
    await refresh();
  }

  async function completeTry(): Promise<void> {
    if (!activeTry.value) return;
    activeTry.value = await api.updateRecipeTry(activeTry.value.id, { status: "completed" });
    stopClock();
    await refresh();
  }

  async function startOperation(operation: TryOperation): Promise<void> {
    if (!activeTry.value) return;
    activeTry.value = await api.updateTryOperation(activeTry.value.id, operation.operationId, {
      status: "active",
    });
    await refresh();
    startClock();
  }

  async function completeOperation(operation: TryOperation): Promise<void> {
    if (!activeTry.value) return;
    activeTry.value = await api.updateTryOperation(activeTry.value.id, operation.operationId, {
      status: "completed",
    });
    await refresh();
  }

  async function skipOperation(operation: TryOperation): Promise<void> {
    if (!activeTry.value) return;
    activeTry.value = await api.updateTryOperation(activeTry.value.id, operation.operationId, {
      status: "skipped",
    });
    await refresh();
  }

  async function recordObservation(input: {
    operationSymbol?: string;
    propertyPath: string;
    value: string;
    unit?: string;
    notes?: string;
  }): Promise<void> {
    if (!activeTry.value) return;
    activeTry.value = await api.addTryObservation(activeTry.value.id, {
      ...input,
      value: input.value,
    });
    await refresh();
  }

  const activeOperation = computed(() =>
    activeTry.value?.operations.find((item) => item.status === "active"),
  );

  const liveTimer = computed<LiveTimerState | null>(() => {
    const operation = activeOperation.value;
    if (!operation?.actualStart) return null;
    const started = Date.parse(operation.actualStart);
    if (Number.isNaN(started)) return null;
    const elapsedSeconds = Math.max(0, Math.floor((now.value - started) / 1000));
    const remainingSeconds = Math.max(0, operation.durationSeconds - elapsedSeconds);
    return {
      operationId: operation.operationId,
      operationSymbol: operation.operationSymbol,
      elapsedSeconds,
      remainingSeconds,
      durationSeconds: operation.durationSeconds,
      overdue: elapsedSeconds > operation.durationSeconds,
    };
  });

  const nextPendingOperation = computed(() =>
    activeTry.value?.operations.find((item) => item.status === "pending"),
  );

  function formatClock(totalSeconds: number): string {
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
  }

  onUnmounted(stopClock);

  return {
    tries,
    activeTry,
    error,
    liveTimer,
    activeOperation,
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
    stopClock,
  };
}
