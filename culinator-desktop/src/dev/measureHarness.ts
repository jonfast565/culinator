/* Temporary verification harness for index-card measurement. Not shipped. */
import { computed, createApp, defineComponent, h, nextTick, provide, ref } from "vue";
import { initParser } from "../services/wasm/parser";
import { parseUiModel } from "../features/recipe-editor/model";
import { buildNarrative } from "../features/recipe-editor/narrative";
import type { NarrativeStep } from "../features/recipe-editor/narrative";
import IndexCardRecipe from "../features/reading/components/IndexCardRecipe.vue";
import IndexCardStage from "../features/reading/components/IndexCardStage.vue";
import {
  useViewSettings,
  VIEW_SETTINGS_KEY,
} from "../features/reading/composables/useViewSettings";
import type { IndexCardFormat } from "../features/reading/indexCardFormat";
import type { IndexCardMargin } from "../features/reading/indexCardMargin";
import type { MisePlacement } from "../features/reading/composables/useViewSettings";
import type { RecipeTypeScale } from "../features/reading/recipeTypeScale";
import applePie from "./apple_pie.cg?raw";
import beefStew from "./beef_stew.cg?raw";
import brigadeiro from "./brigadeiro.cg?raw";
import "../styles/base.css";
import "../features/reading/recipe-leaf.css";
import "../features/reading/index-card-view.css";

const RECIPES: Record<string, string> = { applePie, beefStew, brigadeiro };
const FORMATS = ["2.5x4", "4x2.5", "3x5", "5x3", "4x6", "6x4", "5x8", "8x5"] as const;
const MARGINS: IndexCardMargin[] = ["tight", "medium", "wide"];

const source = ref(applePie);

const settings = useViewSettings();

const Harness = defineComponent({
  setup() {
    provide(VIEW_SETTINGS_KEY, settings);
    const model = computed(() => parseUiModel(source.value));
    const narrative = computed(() => buildNarrative(source.value));
    const format = computed(() =>
      settings.indexCardFormat.value === "full"
        ? ("6x4" as const)
        : (settings.indexCardFormat.value as Exclude<IndexCardFormat, "full">),
    );
    return () =>
      h(IndexCardStage, null, {
        default: () =>
          h(IndexCardRecipe, {
            format: format.value,
            model: model.value,
            summary: narrative.value.summary,
            eyebrow: model.value.attribution || model.value.source || "Recipe",
            allergens: [],
            ingredientGroups: narrative.value.ingredientGroups,
            equipment: narrative.value.equipment,
            sections: narrative.value.sections,
            operationFor: (step: NarrativeStep) =>
              model.value.operations.find((operation) => operation.symbol === step.symbol),
          }),
      });
  },
});

interface CardAudit {
  card: number;
  overflowPx: number;
  scrollOverflowPx: number;
  freePx: number;
  sections: string[];
  steps: number;
  ingredients: number;
}

function auditCards(): CardAudit[] {
  const frames = Array.from(document.querySelectorAll<HTMLElement>(".index-card-deck .index-card-frame"));
  return frames.map((frame, index) => {
    const leaf = frame.querySelector<HTMLElement>(".leaf.index-card-leaf");
    if (!leaf) return { card: index + 1, overflowPx: 0, scrollOverflowPx: 0, freePx: 0, sections: [], steps: 0, ingredients: 0 };
    const style = window.getComputedStyle(leaf);
    const rect = leaf.getBoundingClientRect();
    const contentBottom = rect.bottom - Number.parseFloat(style.paddingBottom);
    let lowest = rect.top + Number.parseFloat(style.paddingTop);
    for (const child of Array.from(leaf.children)) {
      if (child.classList.contains("card-pager")) continue;
      lowest = Math.max(lowest, child.getBoundingClientRect().bottom);
    }
    return {
      card: index + 1,
      overflowPx: Math.round((lowest - contentBottom) * 100) / 100,
      scrollOverflowPx: leaf.scrollHeight - leaf.clientHeight,
      freePx: Math.round((contentBottom - lowest) * 100) / 100,
      sections: Array.from(leaf.querySelectorAll(".leaf-section")).map(
        (section) => section.querySelector(".section-label")?.textContent?.trim() ?? section.className,
      ),
      steps: leaf.querySelectorAll(".step").length,
      ingredients: leaf.querySelectorAll(".ingredient-groups .ingredient-row").length,
    };
  });
}

function settle(frames = 6): Promise<void> {
  return new Promise((resolve) => {
    let left = frames;
    const tick = (): void => {
      left -= 1;
      if (left <= 0) window.setTimeout(resolve, 60);
      else window.requestAnimationFrame(tick);
    };
    window.requestAnimationFrame(tick);
  });
}

interface Combo {
  recipe: string;
  format: string;
  margin: IndexCardMargin;
  mise: MisePlacement;
  scale: RecipeTypeScale;
  cards: number;
  totalSteps: number;
  totalIngredients: number;
  worstOverflow: number;
  audits: CardAudit[];
}

async function run(): Promise<void> {
  const report: Combo[] = [];
  const expected: Record<string, { steps: number; ingredients: number }> = {};
  for (const [name, text] of Object.entries(RECIPES)) {
    const narrative = buildNarrative(text);
    expected[name] = {
      steps: narrative.sections.reduce((sum, section) => sum + section.steps.length, 0),
      ingredients: narrative.ingredientGroups.reduce((sum, group) => sum + group.items.length, 0),
    };
  }

  const variants: { mise: MisePlacement; scale: RecipeTypeScale }[] = [
    { mise: "top-matter", scale: "md" },
    { mise: "top-matter", scale: "lg" },
    { mise: "colocated", scale: "md" },
  ];

  for (const [name, text] of Object.entries(RECIPES)) {
    source.value = text;
    for (const variant of variants) {
      settings.setMisePlacement(variant.mise);
      settings.setRecipeBodyScale(variant.scale);
      settings.setRecipeHeaderScale(variant.scale);
      settings.setRecipeAnnotationScale(variant.scale);
      for (const format of FORMATS) {
        for (const margin of MARGINS) {
          settings.setIndexCardFormat(format);
          settings.setIndexCardMargin(margin);
          await nextTick();
          await settle();
          const audits = auditCards();
          report.push({
            recipe: name,
            format,
            margin,
            mise: variant.mise,
            scale: variant.scale,
            cards: audits.length,
            totalSteps: audits.reduce((sum, audit) => sum + audit.steps, 0),
            totalIngredients: audits.reduce((sum, audit) => sum + audit.ingredients, 0),
            worstOverflow: audits.reduce((worst, audit) => Math.max(worst, audit.overflowPx, audit.scrollOverflowPx), 0),
            audits,
          });
        }
      }
    }
  }

  const failures = report.filter((entry) => {
    const want = expected[entry.recipe];
    const stepsOk = entry.totalSteps === want.steps;
    const ingredientsOk =
      entry.mise === "colocated" ? true : entry.totalIngredients === want.ingredients;
    return entry.worstOverflow > 0.5 || !stepsOk || !ingredientsOk;
  });

  const summary = {
    expected,
    combos: report.length,
    failures: failures.map((entry) => ({
      recipe: entry.recipe,
      format: entry.format,
      margin: entry.margin,
      mise: entry.mise,
      scale: entry.scale,
      cards: entry.cards,
      totalSteps: entry.totalSteps,
      totalIngredients: entry.totalIngredients,
      worstOverflow: entry.worstOverflow,
      audits: entry.audits,
    })),
    fill: report.map((entry) => ({
      key: `${entry.recipe} ${entry.format}/${entry.margin} ${entry.mise} ${entry.scale}`,
      cards: entry.cards,
      free: entry.audits.map((audit) => audit.freePx),
      steps: entry.audits.map((audit) => audit.steps),
      ingredients: entry.audits.map((audit) => audit.ingredients),
    })),
  };

  (window as unknown as { __report: unknown }).__report = summary;
  const output = document.getElementById("report");
  if (output) output.textContent = JSON.stringify(summary, null, 1);
  document.title = `HARNESS DONE failures=${failures.length}`;
}

await initParser();
createApp(Harness).mount("#harness");
await nextTick();
void run();
