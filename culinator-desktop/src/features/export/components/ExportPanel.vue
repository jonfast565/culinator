<script setup lang="ts">
import { reactive, ref } from "vue";
import { Calculator, Download, PackageOpen } from "lucide-vue-next";
import type { RecipeExportFormat, RecipeExportOptions } from "../../../domain/types";
import { calculateRecipeNutrition, downloadExport, exportRecipe } from "../../../services/api";
const props = defineProps<{ recipeId: string; recipeTitle: string }>();
const busy = ref(false);
const calculating = ref(false);
const error = ref("");
const generated = ref<string[]>([]);
const options = reactive<RecipeExportOptions>({
  siteTitle: "My Recipe Book",
  author: "",
  description: "",
  includeSource: true,
  formats: ["web", "json", "markdown"],
  nutrition: {
    servingsPerContainer: 1,
    servingSize: "1 serving",
    servingSizeGrams: null,
    calories: 0,
    totalFatGrams: 0,
    saturatedFatGrams: 0,
    transFatGrams: 0,
    cholesterolMilligrams: 0,
    sodiumMilligrams: 0,
    totalCarbohydrateGrams: 0,
    dietaryFiberGrams: 0,
    totalSugarsGrams: 0,
    addedSugarsGrams: 0,
    proteinGrams: 0,
    vitaminDMicrograms: null,
    calciumMilligrams: null,
    ironMilligrams: null,
    potassiumMilligrams: null,
  },
});

const availableFormats: { value: RecipeExportFormat; label: string; detail: string }[] = [
  { value: "web", label: "Web page", detail: "Responsive HTML with embedded label" },
  { value: "print_html", label: "Print HTML", detail: "Printer/PDF-ready HTML" },
  { value: "markdown", label: "Markdown", detail: "Portable recipe document" },
  { value: "plain_text", label: "Plain text", detail: "Simple text for notes and email" },
  { value: "ingredient_csv", label: "Ingredient CSV", detail: "Spreadsheet-friendly ingredients" },
  { value: "json", label: "JSON", detail: "Structured recipe data" },
  { value: "epub", label: "EPUB", detail: "E-reader compatible recipe book file" },
];
function toggleFormat(format: RecipeExportFormat) {
  const index = options.formats.indexOf(format);
  if (index >= 0) options.formats.splice(index, 1);
  else options.formats.push(format);
}
const numericFields = [
  ["Calories", "calories"],
  ["Total fat (g)", "totalFatGrams"],
  ["Saturated fat (g)", "saturatedFatGrams"],
  ["Trans fat (g)", "transFatGrams"],
  ["Cholesterol (mg)", "cholesterolMilligrams"],
  ["Sodium (mg)", "sodiumMilligrams"],
  ["Carbohydrate (g)", "totalCarbohydrateGrams"],
  ["Fiber (g)", "dietaryFiberGrams"],
  ["Total sugars (g)", "totalSugarsGrams"],
  ["Added sugars (g)", "addedSugarsGrams"],
  ["Protein (g)", "proteinGrams"],
] as const;
async function calculateFromIngredients() {
  calculating.value = true;
  error.value = "";
  try {
    const result = await calculateRecipeNutrition(props.recipeId, {
      servingsPerContainer: options.nutrition.servingsPerContainer,
      servingSize: options.nutrition.servingSize,
      servingSizeGrams: options.nutrition.servingSizeGrams,
    });
    Object.assign(options.nutrition, result.facts);
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    calculating.value = false;
  }
}
async function generate() {
  busy.value = true;
  error.value = "";
  try {
    const result = await exportRecipe(props.recipeId, options);
    const saved = await downloadExport(result);
    generated.value = saved ? result.files : [];
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <section class="panel space-y-4">
    <div>
      <h3 class="flex items-center gap-2"><PackageOpen :size="17" />Export bundle</h3>
      <p class="text-sm opacity-70">
        Generate a standalone recipe webpage and matching Nutrition Facts label in one ZIP.
      </p>
    </div>
    <label class="field"><span>Site title</span><input v-model="options.siteTitle" /></label
    ><label class="field"><span>Author</span><input v-model="options.author" /></label
    ><label class="field"
      ><span>Description</span><textarea v-model="options.description" rows="2" />
    </label>
    <div class="grid grid-cols-2 gap-2">
      <label class="field"
        ><span>Servings</span
        ><input
          v-model.number="options.nutrition.servingsPerContainer"
          type="number"
          min="0"
          step="0.5" /></label
      ><label class="field"
        ><span>Serving size</span><input v-model="options.nutrition.servingSize" /></label
      ><label v-for="field in numericFields" :key="field[1]" class="field"
        ><span>{{ field[0] }}</span
        ><input v-model.number="options.nutrition[field[1]]" type="number" min="0" step="0.1"
      /></label>
    </div>
    <button
      class="secondary w-full justify-center"
      :disabled="calculating"
      @click="calculateFromIngredients"
    >
      <Calculator :size="16" />{{
        calculating ? "Calculating…" : "Calculate from linked ingredients"
      }}
    </button>
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
      <Download :size="16" />{{ busy ? "Generating…" : `Export ${recipeTitle}` }}
    </button>
    <p v-if="error" class="diagnostic error">{{ error }}</p>
    <div v-if="generated.length" class="text-xs opacity-70">
      Included: {{ generated.join(", ") }}
    </div>
  </section>
</template>
