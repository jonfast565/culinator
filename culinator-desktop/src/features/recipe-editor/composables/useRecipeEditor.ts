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

  function hasValidationErrors(): boolean {
    return validation.value?.diagnostics.some((item) => item.severity === "error") ?? false;
  }

  watch(
    recipe,
    (next) => {
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
    try {
      const saved = await api.saveRecipe(recipe.value.id, source.value);
      dirty.value = false;
      saveStatus.value = "saved";
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
