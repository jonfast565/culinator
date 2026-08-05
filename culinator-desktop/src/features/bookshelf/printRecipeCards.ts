const PRINT_BODY_CLASS = "printing-recipe-cards";

/** Trigger a landscape print of the open book's recipe card grid. */
export function printRecipeCards(): void {
  document.body.classList.add(PRINT_BODY_CLASS);
  window.print();
  window.addEventListener(
    "afterprint",
    () => {
      document.body.classList.remove(PRINT_BODY_CLASS);
    },
    { once: true },
  );
}
