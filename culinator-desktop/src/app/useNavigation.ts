import { ref } from "vue";

// The app's view-state machine. There is no router; the desktop app moves
// between a small set of full-window views:
//   shelf             → the bookshelf (home)
//   book              → an open book: flip through / search its recipes
//   reading           → a recipe as a full-screen book page
//   editing           → the source + inspector workspace
//   building          → the structured, form-based recipe builder
//   measures          → kitchen unit conversion
//   ingredient-match  → full-window USDA ingredient matcher
export type AppView =
  | "shelf"
  | "book"
  | "reading"
  | "editing"
  | "building"
  | "measures"
  | "ingredient-match";

export function useNavigation() {
  const view = ref<AppView>("shelf");
  // The book currently open (null = the "Unfiled" pseudo-book).
  const bookId = ref<string | null>(null);
  // Where "ingredient-match" should return (never itself).
  const ingredientMatchReturn = ref<AppView>("reading");

  function shelf(): void {
    view.value = "shelf";
  }
  function openBook(id: string | null): void {
    bookId.value = id;
    view.value = "book";
  }
  function read(): void {
    view.value = "reading";
  }
  function edit(): void {
    view.value = "editing";
  }
  function toggleEdit(): void {
    view.value = view.value === "editing" ? "reading" : "editing";
  }
  function build(): void {
    view.value = "building";
  }
  function measures(): void {
    view.value = "measures";
  }
  function ingredientMatch(): void {
    if (view.value !== "ingredient-match") {
      ingredientMatchReturn.value = view.value;
    }
    view.value = "ingredient-match";
  }
  function backFromIngredientMatch(): void {
    const target = ingredientMatchReturn.value;
    view.value = target === "ingredient-match" ? "reading" : target;
  }

  return {
    view,
    bookId,
    shelf,
    openBook,
    read,
    edit,
    toggleEdit,
    build,
    measures,
    ingredientMatch,
    backFromIngredientMatch,
  };
}
