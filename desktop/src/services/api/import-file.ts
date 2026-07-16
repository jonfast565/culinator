/* global HTMLInputElement, document */
export interface LoadedRecipeFile {
  fileName: string;
  sourceText: string;
}

// Recipe DSL source is exported as `recipe.cg`; plain `.txt` is accepted too so
// hand-authored sources import cleanly.
const RECIPE_FILE_EXTENSIONS = ["cg", "txt"];

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/**
 * Prompt the user for a recipe DSL source file and return its text. Inside the
 * Tauri desktop shell this opens a native file dialog and reads the chosen path
 * on the Rust side; in the browser it falls back to a hidden file input.
 * Returns `null` when no file is chosen.
 */
export async function openRecipeFile(): Promise<LoadedRecipeFile | null> {
  return isTauri() ? openViaTauri() : openViaBrowser();
}

function baseName(path: string): string {
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] || path;
}

async function openViaTauri(): Promise<LoadedRecipeFile | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const { invoke } = await import("@tauri-apps/api/core");
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Culinator recipe", extensions: RECIPE_FILE_EXTENSIONS }],
  });
  if (typeof selected !== "string") return null;
  const sourceText = await invoke<string>("read_recipe_file", { path: selected });
  return { fileName: baseName(selected), sourceText };
}

async function openViaBrowser(): Promise<LoadedRecipeFile | null> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = `${RECIPE_FILE_EXTENSIONS.map((ext) => `.${ext}`).join(",")},text/plain`;
    input.addEventListener(
      "change",
      () => {
        const file = (input as HTMLInputElement).files?.[0];
        if (!file) {
          resolve(null);
          return;
        }
        void file.text().then((sourceText) => resolve({ fileName: file.name, sourceText }));
      },
      { once: true },
    );
    input.click();
  });
}
