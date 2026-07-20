import { computed, ref, watch, type Ref } from "vue";
import type { RecipeDocument, ValidationResult } from "../../../domain/types";
import * as api from "../../../services/api";
import { parseUiModel } from "../model";

export type SaveStatus = "idle" | "saving" | "saved" | "error";

export function useRecipeEditor(recipe: Ref<RecipeDocument | null>) {
  const source = ref("");
  const dirty = ref(false);
  const saving = ref(false);
  const saveStatus = ref<SaveStatus>("idle");
  const validation = ref<ValidationResult | null>(null);
  const model = computed(() => parseUiModel(source.value));
  let validateTimer: number | undefined;
  let autoSaveTimer: number | undefined;
  let savedResetTimer: number | undefined;
  // Which recipe the buffer currently holds, and the text of the last save we
  // made. Saving emits `recipes.changed`, which refreshes the library and hands
  // this composable a *new* `RecipeDocument` object — so the watch below fires
  // on our own save. Without these we would reset the buffer to the server's
  // copy and silently discard anything typed while the save was in flight.
  let loadedId: string | null = null;
  let lastSavedSource: string | null = null;

  function hasValidationErrors(): boolean {
    return validation.value?.diagnostics.some((item) => item.severity === "error") ?? false;
  }

  watch(
    recipe,
    (next) => {
      // An echo of our own save: the recipe already in the buffer, with the
      // server handing back exactly the text we sent. Keep the buffer. Checking
      // `loadedId` (not just the saved id) matters when you save A, switch to
      // B, then come back to A — the buffer holds B, so A must still reload.
      // A genuine foreign edit has different text and still wins.
      if (next && next.id === loadedId && next.sourceText === lastSavedSource) return;
      loadedId = next?.id ?? null;
      source.value = next?.sourceText ?? "";
      dirty.value = false;
      saveStatus.value = "idle";
    },
    { immediate: true },
  );

  watch(source, (next) => {
    dirty.value = next !== (recipe.value?.sourceText ?? "");
    if (dirty.value && saveStatus.value === "saved") saveStatus.value = "idle";
    window.clearTimeout(validateTimer);
    validateTimer = window.setTimeout(
      () => void api.validateRecipe(next).then((result) => (validation.value = result)),
      250,
    );
    return () => window.clearTimeout(validateTimer);
  });

  watch([dirty, validation, source], () => {
    window.clearTimeout(autoSaveTimer);
    if (!dirty.value || !recipe.value || hasValidationErrors()) return;
    autoSaveTimer = window.setTimeout(() => {
      void performSave(true);
    }, 2000);
    return () => window.clearTimeout(autoSaveTimer);
  });

  async function performSave(auto = false): Promise<RecipeDocument | null> {
    if (!recipe.value || !dirty.value) return null;
    if (hasValidationErrors()) return null;
    saving.value = true;
    saveStatus.value = "saving";
    // Snapshot what we are actually persisting: `source` can change while the
    // request is in flight, and clearing `dirty` unconditionally on return used
    // to mark those newer keystrokes clean — which cancelled their pending
    // autosave, so they were never written at all.
    const persisted = source.value;
    try {
      const saved = await api.saveRecipe(recipe.value.id, persisted);
      lastSavedSource = persisted;
      const stillDirty = source.value !== persisted;
      dirty.value = stillDirty;
      // Don't claim "saved" while newer keystrokes are still pending; their own
      // autosave will report it.
      saveStatus.value = stillDirty ? "idle" : "saved";
      window.clearTimeout(savedResetTimer);
      savedResetTimer = window.setTimeout(() => {
        if (!dirty.value) saveStatus.value = "idle";
      }, 2000);
      return saved;
    } catch {
      saveStatus.value = "error";
      if (!auto) throw new Error("Save failed");
      return null;
    } finally {
      saving.value = false;
    }
  }

  async function save(): Promise<RecipeDocument | null> {
    return performSave(false);
  }

  async function remove(): Promise<void> {
    if (recipe.value) await api.deleteRecipe(recipe.value.id);
  }

  function appendSnippet(snippet: string): void {
    source.value = `${source.value.trimEnd()}\n\n${snippet}\n`;
  }

  return {
    source,
    dirty,
    saving,
    saveStatus,
    validation,
    model,
    save,
    remove,
    appendSnippet,
  };
}
