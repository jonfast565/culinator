<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { Download, PackageOpen } from "lucide-vue-next";
import type { RecipeExportFormat, RecipeExportOptions } from "../../../domain/types";
import {
  calculateRecipeNutrition,
  downloadExport,
  emptyNutritionFacts,
  exportRecipe,
  getNutritionState,
} from "../../../services/api";
const props = defineProps<{ recipeId: string; recipeTitle: string }>();
const busy = ref(false);
const loadingNutrition = ref(false);
const error = ref("");
const generated = ref<string[]>([]);
const nutritionNote = ref("");
const options = reactive<RecipeExportOptions>({
  siteTitle: "My Recipe Book",
  author: "",
  description: "",
  includeSource: false,
  formats: ["print_html"],
  nutrition: emptyNutritionFacts(),
});

const downloadLabel = computed(() => {
  if (options.formats.length + (options.includeSource ? 1 : 0) > 1) {
    return `Export ${props.recipeTitle} (zip)`;
  }
  return `Export ${props.recipeTitle}`;
});

const previewFacts = computed(() => options.nutrition);

const availableFormats: { value: RecipeExportFormat; label: string; detail: string }[] = [
  { value: "web", label: "Web page", detail: "Responsive HTML with embedded label" },
  { value: "print_html", label: "Print HTML", detail: "Printer/PDF-ready HTML" },
  { value: "markdown", label: "Markdown", detail: "Portable recipe document" },
  { value: "plain_text", label: "Plain text", detail: "Simple text for notes and email" },
  { value: "ingredient_csv", label: "Ingredient CSV", detail: "Spreadsheet-friendly ingredients" },
  { value: "json", label: "JSON", detail: "Structured recipe data" },
  { value: "epub", label: "EPUB", detail: "E-reader compatible recipe book file" },
];

const hasNutritionData = computed(
  () =>
    previewFacts.value.calories > 0 ||
    previewFacts.value.proteinGrams > 0 ||
    previewFacts.value.totalFatGrams > 0 ||
    previewFacts.value.totalCarbohydrateGrams > 0,
);

function toggleFormat(format: RecipeExportFormat) {
  const index = options.formats.indexOf(format);
  if (index >= 0) options.formats.splice(index, 1);
  else options.formats.push(format);
}

async function loadRecipeNutrition(): Promise<void> {
  loadingNutrition.value = true;
  nutritionNote.value = "";
  try {
    const state = await getNutritionState(props.recipeId);
    if (state.manualOverride && state.manualFacts) {
      Object.assign(options.nutrition, state.manualFacts);
      nutritionNote.value = "Using saved manual nutrition from the Nutrition panel.";
      return;
    }
    const result = await calculateRecipeNutrition(props.recipeId);
    Object.assign(options.nutrition, result.facts);
    if (result.manualOverride) {
      nutritionNote.value = "Using saved manual nutrition from the Nutrition panel.";
    } else if (result.calculated && result.linkedIngredientCount > 0) {
      nutritionNote.value = `Calculated from ${result.linkedIngredientCount} of ${result.totalIngredientCount} ingredients. Edit links in the Nutrition panel.`;
    } else {
      nutritionNote.value =
        "No ingredient nutrition linked yet. Configure in the Nutrition panel before exporting.";
    }
  } catch (cause) {
    nutritionNote.value = "Could not load nutrition. Configure in the Nutrition panel.";
    Object.assign(options.nutrition, emptyNutritionFacts());
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    loadingNutrition.value = false;
  }
}

async function generate() {
  busy.value = true;
  error.value = "";
  try {
    await loadRecipeNutrition();
    const result = await exportRecipe(props.recipeId, options);
    const saved = await downloadExport(result);
    generated.value = saved ? result.files : [];
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

onMounted(loadRecipeNutrition);
</script>
<template>
  <section class="panel space-y-4">
    <div>
      <h3 class="flex items-center gap-2"><PackageOpen :size="17" />Export</h3>
      <p class="text-sm opacity-70">
        Download one file for a single format, or a zip when you pick multiple formats (or include
        source). Nutrition comes from the Nutrition panel.
      </p>
    </div>
    <label class="field"><span>Site title</span><input v-model="options.siteTitle" /></label
    ><label class="field"><span>Author</span><input v-model="options.author" /></label
    ><label class="field"
      ><span>Description</span><textarea v-model="options.description" rows="2" />
    </label>
    <div class="rounded border border-current/15 p-3 text-sm">
      <div class="font-semibold">Nutrition label preview</div>
      <p v-if="loadingNutrition" class="opacity-70">Loading recipe nutrition…</p>
      <template v-else>
        <p class="opacity-70">{{ nutritionNote }}</p>
        <dl v-if="hasNutritionData" class="mt-2 grid grid-cols-2 gap-x-4 gap-y-1">
          <div>
            <dt class="opacity-70">Calories</dt>
            <dd>{{ previewFacts.calories }}</dd>
          </div>
          <div>
            <dt class="opacity-70">Protein</dt>
            <dd>{{ previewFacts.proteinGrams }} g</dd>
          </div>
          <div>
            <dt class="opacity-70">Total fat</dt>
            <dd>{{ previewFacts.totalFatGrams }} g</dd>
          </div>
          <div>
            <dt class="opacity-70">Carbs</dt>
            <dd>{{ previewFacts.totalCarbohydrateGrams }} g</dd>
          </div>
          <div class="col-span-2">
            <dt class="opacity-70">Serving size</dt>
            <dd>{{ previewFacts.servingSize }}</dd>
          </div>
        </dl>
      </template>
    </div>
    <div class="space-y-2">
      <div class="text-sm font-semibold">Formats</div>
      <div class="grid gap-2 sm:grid-cols-2">
        <label
          v-for="format in availableFormats"
          :key="format.value"
          class="flex cursor-pointer gap-2 rounded border border-current/15 p-2 text-sm"
        >
          <input
            type="checkbox"
            :checked="options.formats.includes(format.value)"
            @change="toggleFormat(format.value)"
          />
          <span
            ><strong class="block">{{ format.label }}</strong
            ><small class="opacity-65">{{ format.detail }}</small></span
          >
        </label>
      </div>
    </div>
    <label class="flex items-center gap-2 text-sm"
      ><input v-model="options.includeSource" type="checkbox" /> Include DSL source</label
    ><button class="primary w-full justify-center" :disabled="busy" @click="generate">
      <Download :size="16" />{{ busy ? "Generating…" : downloadLabel }}
    </button>
    <p v-if="error" class="diagnostic error">{{ error }}</p>
    <div v-if="generated.length" class="text-xs opacity-70">
      Included: {{ generated.join(", ") }}
    </div>
  </section>
</template>
