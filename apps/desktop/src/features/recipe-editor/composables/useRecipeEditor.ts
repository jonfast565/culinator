import { computed, ref, watch, type Ref } from "vue";
import type { RecipeDocument, ValidationResult } from "../../../domain/types";
import * as api from "../../../services/api";
import { parseUiModel } from "../model";

export function useRecipeEditor(recipe: Ref<RecipeDocument | null>) {
  const source = ref("");
  const dirty = ref(false);
  const saving = ref(false);
  const validation = ref<ValidationResult | null>(null);
  const model = computed(() => parseUiModel(source.value));
  watch(
    recipe,
    (next) => {
      source.value = next?.sourceText ?? "";
      dirty.value = false;
    },
    { immediate: true },
  );
  watch(source, (next) => {
    dirty.value = next !== (recipe.value?.sourceText ?? "");
    const timer = window.setTimeout(
      () => void api.validateRecipe(next).then((result) => (validation.value = result)),
      250,
    );
    return () => window.clearTimeout(timer);
  });
  async function save(): Promise<RecipeDocument | null> {
    if (!recipe.value) return null;
    saving.value = true;
    try {
      const saved = await api.saveRecipe(recipe.value.id, source.value);
      dirty.value = false;
      return saved;
    } finally {
      saving.value = false;
    }
  }
  async function remove(): Promise<void> {
    if (recipe.value) await api.deleteRecipe(recipe.value.id);
  }
  function appendSnippet(snippet: string): void {
    source.value = `${source.value.trimEnd()}\n\n${snippet}\n`;
  }
  return { source, dirty, saving, validation, model, save, remove, appendSnippet };
}
