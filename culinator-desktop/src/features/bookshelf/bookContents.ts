import type { UiRecipeModel } from "../recipe-editor/model";
import { formatDuration, previewSteps, previewIngredients } from "../recipe-editor/narrative";

// Turns a book's loaded recipe models into an ordered list of book "leaves":
// cover → table of contents (front matter) → [section divider → recipe cards]…
// Leaf array order == StPageFlip page index, so the TOC can flip straight to a
// recipe by its page number.

export interface LoadedRecipe {
  id: string;
  model: UiRecipeModel;
}

export interface TocEntry {
  recipeId: string;
  title: string;
  section: string;
  page: number;
}

export type BookLeaf =
  | { kind: "cover"; key: string; title: string; subtitle: string }
  | { kind: "toc"; key: string; entries: TocEntry[] }
  | { kind: "section"; key: string; title: string }
  | {
      kind: "recipe";
      key: string;
      recipeId: string;
      eyebrow: string;
      title: string;
      summary: string;
      ingredients: string[];
      steps: string[];
      stepCount: number;
      cover?: string;
    };

const DEFAULT_SECTION = "Recipes";

export function sectionOf(model: UiRecipeModel): string {
  return model.section?.trim() || DEFAULT_SECTION;
}

export function summarize(model: UiRecipeModel): string {
  const ingredients = model.resources.filter((resource) => resource.kind === "ingredient");
  const steps = model.operations ?? [];
  const totalMinutes = steps.reduce((sum, operation) => sum + (operation.durationMinutes || 0), 0);
  const parts = [
    `${ingredients.length} ingredient${ingredients.length === 1 ? "" : "s"}`,
    `${steps.length} step${steps.length === 1 ? "" : "s"}`,
  ];
  const time = formatDuration(totalMinutes);
  if (time) parts.push(`~${time}`);
  return parts.join(" · ");
}

function topIngredients(model: UiRecipeModel, count = 5): string[] {
  return previewIngredients(model, count);
}

export function buildLeaves(bookTitle: string, recipes: LoadedRecipe[]): BookLeaf[] {
  const leaves: BookLeaf[] = [];
  leaves.push({
    kind: "cover",
    key: "cover",
    title: bookTitle,
    subtitle: `${recipes.length} recipe${recipes.length === 1 ? "" : "s"}`,
  });
  const toc: Extract<BookLeaf, { kind: "toc" }> = { kind: "toc", key: "toc", entries: [] };
  leaves.push(toc);

  // Group by section, preserving the order each section first appears.
  const order: string[] = [];
  const groups = new Map<string, LoadedRecipe[]>();
  for (const recipe of recipes) {
    const section = sectionOf(recipe.model);
    if (!groups.has(section)) {
      groups.set(section, []);
      order.push(section);
    }
    groups.get(section)!.push(recipe);
  }
  const showDividers = order.length > 1;

  for (const section of order) {
    if (showDividers) {
      leaves.push({ kind: "section", key: `sec-${section}`, title: section });
    }
    for (const recipe of groups.get(section)!) {
      const page = leaves.length;
      const title = recipe.model.title || "Untitled recipe";
      const operations = recipe.model.operations ?? [];
      leaves.push({
        kind: "recipe",
        key: `recipe-${recipe.id}`,
        recipeId: recipe.id,
        eyebrow: section,
        title,
        summary: summarize(recipe.model),
        ingredients: topIngredients(recipe.model),
        steps: previewSteps(recipe.model, 4),
        stepCount: operations.length,
        cover: recipe.model.coverImage,
      });
      toc.entries.push({ recipeId: recipe.id, title, section, page });
    }
  }

  return leaves;
}
