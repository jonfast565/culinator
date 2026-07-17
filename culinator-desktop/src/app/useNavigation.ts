import { ref } from "vue";

// The app's view-state machine. There is no router; the desktop app moves
// between a small set of full-window views:
//   shelf   → the bookshelf (home)
//   book    → an open book: flip through / search its recipes
//   reading → a recipe as a full-screen book page
//   editing → the source + inspector workspace
export type AppView = "shelf" | "book" | "reading" | "editing";

export function useNavigation() {
  const view = ref<AppView>("shelf");
  // The book currently open (null = the "Unfiled" pseudo-book).
  const bookId = ref<string | null>(null);

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

  return { view, bookId, shelf, openBook, read, edit, toggleEdit };
}
