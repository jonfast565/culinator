import { computed, onMounted, ref } from "vue";
import * as api from "../../../services/api";
import { onServiceEvent } from "../../../services/transport/websocket-client";
import type { RecipeBookSummary, RecipeDocument, RecipeSummary } from "../../../domain/types";

export function useRecipeLibrary() {
  const books = ref<RecipeBookSummary[]>([]);
  const recipes = ref<RecipeSummary[]>([]);
  const selectedBookId = ref<string | null>(null);
  const selectedRecipe = ref<RecipeDocument | null>(null);
  const loading = ref(false);

  // The recipe the user means to have selected — the single source of truth for
  // selection, set synchronously on intent (open/create/select). `refresh`
  // always honours it, so a server-pushed `recipes.changed` refresh (which
  // carries no id) can't reselect the previous recipe. Creating a recipe fires
  // that event *and* triggers an explicit refresh; without this pin the two
  // interleaving async refreshes raced, and "New recipe" often reopened the
  // recipe that was already selected instead of the new skeleton.
  const intendedRecipeId = ref<string | null>(null);
  let refreshing = false;
  let refreshAgain = false;

  const groupedRecipes = computed(
    () =>
      new Map(
        books.value.map((book) => [
          book.id,
          recipes.value.filter((recipe) => recipe.bookId === book.id),
        ]),
      ),
  );
  const unfiled = computed(() => recipes.value.filter((recipe) => !recipe.bookId));

  async function refresh(preferredRecipeId?: string): Promise<void> {
    if (preferredRecipeId) intendedRecipeId.value = preferredRecipeId;
    // Coalesce concurrent refreshes: if one is already running (e.g. a server
    // event arrived mid-refresh) let it repeat once more rather than racing it.
    if (refreshing) {
      refreshAgain = true;
      return;
    }
    refreshing = true;
    loading.value = true;
    try {
      do {
        refreshAgain = false;
        const [nextBooks, nextRecipes] = await Promise.all([
          api.listRecipeBooks(),
          api.listRecipes(),
        ]);
        books.value = nextBooks;
        recipes.value = nextRecipes;
        selectedBookId.value ??= nextBooks[0]?.id ?? null;
        const wanted = intendedRecipeId.value ?? selectedRecipe.value?.id;
        // Honour the intended recipe when it still exists, else fall back to the
        // first — a pinned id that was just deleted must not strand the view.
        const id = wanted && nextRecipes.some((r) => r.id === wanted) ? wanted : nextRecipes[0]?.id;
        selectedRecipe.value = id ? await api.getRecipe(id) : null;
        intendedRecipeId.value = id ?? null;
      } while (refreshAgain);
    } finally {
      refreshing = false;
      loading.value = false;
    }
  }
  async function selectRecipe(id: string): Promise<void> {
    intendedRecipeId.value = id;
    selectedRecipe.value = await api.getRecipe(id);
  }
  async function createRecipe(): Promise<void> {
    const recipe = await api.createRecipe(selectedBookId.value);
    // Pin the new recipe before refreshing so the `recipes.changed` echo can't
    // reselect the old one.
    intendedRecipeId.value = recipe.id;
    await refresh(recipe.id);
  }
  async function createBook(title: string): Promise<void> {
    const book = await api.createRecipeBook(title);
    selectedBookId.value = book.id;
    await refresh();
  }
  async function renameBook(book: RecipeBookSummary, title: string): Promise<void> {
    await api.updateRecipeBook(book.id, title, book.description ?? undefined);
    await refresh(selectedRecipe.value?.id);
  }
  async function deleteBook(book: RecipeBookSummary): Promise<void> {
    await api.deleteRecipeBook(book.id);
    selectedBookId.value = null;
    await refresh(selectedRecipe.value?.id);
  }
  async function moveSelected(bookId: string | null): Promise<void> {
    if (!selectedRecipe.value) return;
    await api.moveRecipeToBook(selectedRecipe.value.id, bookId);
    selectedBookId.value = bookId;
    await refresh(selectedRecipe.value.id);
  }
  async function moveRecipes(ids: string[], bookId: string | null): Promise<void> {
    if (!ids.length) return;
    await Promise.all(ids.map((id) => api.moveRecipeToBook(id, bookId)));
    await refresh(selectedRecipe.value?.id);
  }
  async function deleteRecipes(ids: string[]): Promise<void> {
    if (!ids.length) return;
    await Promise.all(ids.map((id) => api.deleteRecipe(id)));
    await refresh();
  }

  onMounted(() => {
    void refresh();
    onServiceEvent((event) => {
      if (["recipes.changed", "books.changed"].includes(event.event)) void refresh();
    });
  });
  return {
    books,
    recipes,
    groupedRecipes,
    unfiled,
    selectedBookId,
    selectedRecipe,
    loading,
    refresh,
    selectRecipe,
    createRecipe,
    createBook,
    renameBook,
    deleteBook,
    moveSelected,
    moveRecipes,
    deleteRecipes,
  };
}
