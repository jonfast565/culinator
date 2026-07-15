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
    loading.value = true;
    try {
      const [nextBooks, nextRecipes] = await Promise.all([
        api.listRecipeBooks(),
        api.listRecipes(),
      ]);
      books.value = nextBooks;
      recipes.value = nextRecipes;
      selectedBookId.value ??= nextBooks[0]?.id ?? null;
      const id = preferredRecipeId ?? selectedRecipe.value?.id ?? nextRecipes[0]?.id;
      selectedRecipe.value = id ? await api.getRecipe(id) : null;
    } finally {
      loading.value = false;
    }
  }
  async function selectRecipe(id: string): Promise<void> {
    selectedRecipe.value = await api.getRecipe(id);
  }
  async function createRecipe(): Promise<void> {
    const recipe = await api.createRecipe(selectedBookId.value);
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
  };
}
