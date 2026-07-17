<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import {
  Apple,
  Calculator,
  Link2,
  PenLine,
  Save,
  Search,
  Sparkles,
  Unlink,
  Wand2,
} from "lucide-vue-next";
import type { UiResource } from "../../recipe-editor/model";
import type {
  FuzzyFoodMatch,
  NutritionFacts,
  NutritionSearchResult,
  RecipeNutritionResult,
  RecipeNutritionState,
} from "../../../domain/types";
import * as api from "../../../services/api";

const props = defineProps<{
  recipeId: string;
  resources: UiResource[];
}>();

const catalogAvailable = ref(false);
const state = ref<RecipeNutritionState | null>(null);
const result = ref<RecipeNutritionResult | null>(null);
const error = ref("");
const busy = ref(false);
const saving = ref(false);
const autoLinking = ref(false);
const searchQuery = ref("");
const searchResults = ref<FuzzyFoodMatch[]>([]);
const activeSymbol = ref<string | null>(null);
const manualSymbol = ref<string | null>(null);
const searching = ref(false);

const manualOverride = ref(false);
const recipeFacts = reactive<NutritionFacts>(api.emptyNutritionFacts());
const manualDrafts = reactive<Record<string, NutritionFacts>>({});

const ingredients = computed(() =>
  props.resources.filter((resource) => resource.kind === "ingredient"),
);

const linkBySymbol = computed(() =>
  Object.fromEntries((state.value?.links ?? []).map((link) => [link.resourceSymbol, link])),
);

const manualBySymbol = computed(() =>
  Object.fromEntries(
    (state.value?.manualIngredients ?? []).map((entry) => [entry.resourceSymbol, entry]),
  ),
);

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

function linkTooltip(symbol: string): string {
  const link = linkBySymbol.value[symbol];
  if (link) return `Linked: ${link.foodDescription}`;
  const manual = manualBySymbol.value[symbol];
  if (manual) return "Manual nutrition facts (per 100 g)";
  return "Not linked — click to map or enter manual facts";
}

function linkStatus(symbol: string): "linked" | "manual" | "none" {
  if (linkBySymbol.value[symbol]) return "linked";
  if (manualBySymbol.value[symbol]) return "manual";
  return "none";
}

function cloneFacts<T>(value: T): T {
  return globalThis.structuredClone(value);
}

function ensureManualDraft(symbol: string): NutritionFacts {
  if (!manualDrafts[symbol]) {
    manualDrafts[symbol] = cloneFacts(
      manualBySymbol.value[symbol]?.factsPer100g ?? api.per100gFacts(),
    );
  }
  return manualDrafts[symbol];
}

async function refresh(): Promise<void> {
  const status = await api.getNutritionStatus();
  catalogAvailable.value = status.catalogAvailable;
  state.value = await api.getNutritionState(props.recipeId);
  manualOverride.value = state.value.manualOverride;
  Object.assign(recipeFacts, state.value.manualFacts ?? api.emptyNutritionFacts());
  for (const entry of state.value.manualIngredients) {
    manualDrafts[entry.resourceSymbol] = cloneFacts(entry.factsPer100g);
  }
}

async function runSearch(symbol?: string): Promise<void> {
  if (!searchQuery.value.trim()) return;
  searching.value = true;
  error.value = "";
  try {
    searchResults.value = await api.fuzzyMatchNutritionFoods(searchQuery.value);
    if (symbol && searchResults.value.length === 0) {
      searchResults.value = (await api.searchNutritionFoods(searchQuery.value)).map(
        (result, index) => ({ result, score: 1 - index * 0.05 }),
      );
    }
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    searching.value = false;
  }
}

async function linkFood(symbol: string, food: NutritionSearchResult): Promise<void> {
  busy.value = true;
  error.value = "";
  try {
    await api.linkIngredientNutrition(props.recipeId, symbol, food.fdcId);
    await refresh();
    activeSymbol.value = null;
    searchResults.value = [];
    searchQuery.value = "";
    result.value = null;
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    busy.value = false;
  }
}

async function unlinkFood(symbol: string): Promise<void> {
  busy.value = true;
  error.value = "";
  try {
    await api.unlinkIngredientNutrition(props.recipeId, symbol);
    await refresh();
    result.value = null;
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    busy.value = false;
  }
}

async function saveManualIngredient(symbol: string): Promise<void> {
  saving.value = true;
  error.value = "";
  try {
    await api.saveIngredientManualNutrition(props.recipeId, {
      resourceSymbol: symbol,
      factsPer100g: ensureManualDraft(symbol),
    });
    await refresh();
    manualSymbol.value = null;
    result.value = null;
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    saving.value = false;
  }
}

async function clearManualIngredient(symbol: string): Promise<void> {
  saving.value = true;
  error.value = "";
  try {
    await api.deleteIngredientManualNutrition(props.recipeId, symbol);
    delete manualDrafts[symbol];
    await refresh();
    manualSymbol.value = null;
    result.value = null;
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    saving.value = false;
  }
}

async function saveRecipeFacts(): Promise<void> {
  saving.value = true;
  error.value = "";
  try {
    state.value = await api.saveRecipeNutrition(props.recipeId, {
      manualOverride: manualOverride.value,
      facts: manualOverride.value ? cloneFacts(recipeFacts) : null,
    });
    result.value = null;
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    saving.value = false;
  }
}

async function autoLinkAll(): Promise<void> {
  autoLinking.value = true;
  error.value = "";
  try {
    const autoResult = await api.autoLinkIngredients(props.recipeId);
    await refresh();
    result.value = null;
    if (autoResult.linked.length === 0 && autoResult.skipped.length > 0) {
      error.value = `No confident matches found for ${autoResult.skipped.length} ingredient(s). Try manual mapping.`;
    }
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    autoLinking.value = false;
  }
}

async function calculate(): Promise<void> {
  busy.value = true;
  error.value = "";
  try {
    result.value = await api.calculateRecipeNutrition(props.recipeId, {
      servingsPerContainer: recipeFacts.servingsPerContainer,
      servingSize: recipeFacts.servingSize,
      servingSizeGrams: recipeFacts.servingSizeGrams,
    });
    if (!manualOverride.value && result.value.calculated) {
      Object.assign(recipeFacts, result.value.facts);
    }
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    busy.value = false;
  }
}

function openMapping(resource: UiResource): void {
  activeSymbol.value = activeSymbol.value === resource.symbol ? null : resource.symbol;
  manualSymbol.value = null;
  searchQuery.value = resource.name || resource.symbol;
  searchResults.value = [];
  if (activeSymbol.value) void runSearch(resource.symbol);
}

function openManual(resource: UiResource): void {
  manualSymbol.value = manualSymbol.value === resource.symbol ? null : resource.symbol;
  activeSymbol.value = null;
  ensureManualDraft(resource.symbol);
}

onMounted(() => {
  void refresh();
});

watch(
  () => props.recipeId,
  () => {
    result.value = null;
    activeSymbol.value = null;
    manualSymbol.value = null;
    void refresh();
  },
);
</script>

<template>
  <section class="panel space-y-4">
    <div>
      <h3 class="flex items-center gap-2"><Apple :size="17" />Nutrition</h3>
      <p class="text-sm opacity-70">
        Link ingredients to USDA FoodData Central, enter manual facts when needed, and save
        recipe-level nutrition labels.
      </p>
    </div>

    <p v-if="!catalogAvailable" class="diagnostic warning text-sm">
      Nutrition database not found. Build <code>fdc.sqlite3</code> or use manual entry per
      ingredient.
    </p>

    <div class="flex flex-wrap gap-2">
      <button
        class="secondary"
        :disabled="autoLinking || !catalogAvailable || busy"
        @click="autoLinkAll"
      >
        <Wand2 :size="14" />{{ autoLinking ? "Matching…" : "Auto-link ingredients (fuzzy)" }}
      </button>
    </div>

    <div class="space-y-2">
      <div class="text-sm font-semibold">Ingredients</div>
      <article v-for="resource in ingredients" :key="resource.symbol" class="card space-y-2">
        <div class="flex items-start justify-between gap-2">
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <button
                class="secondary icon nutrition-link-btn"
                :class="linkStatus(resource.symbol)"
                :title="linkTooltip(resource.symbol)"
                @click="openMapping(resource)"
              >
                <Link2 v-if="linkStatus(resource.symbol) === 'linked'" :size="14" />
                <PenLine v-else-if="linkStatus(resource.symbol) === 'manual'" :size="14" />
                <Link2 v-else :size="14" class="opacity-40" />
              </button>
              <strong>{{ resource.name || resource.symbol }}</strong>
            </div>
            <small class="block opacity-70">
              {{ resource.quantity || "no quantity" }}
            </small>
            <small v-if="linkBySymbol[resource.symbol]" class="block opacity-80">
              {{ linkBySymbol[resource.symbol].foodDescription }}
            </small>
            <small v-else-if="manualBySymbol[resource.symbol]" class="block opacity-80">
              Manual facts per 100 g
            </small>
          </div>
          <div class="flex gap-1">
            <button
              class="secondary"
              :title="'Enter manual nutrition facts'"
              :disabled="busy || saving"
              @click="openManual(resource)"
            >
              <PenLine :size="14" />
            </button>
            <button
              v-if="linkBySymbol[resource.symbol]"
              class="secondary"
              :disabled="busy"
              title="Unlink FDC entry"
              @click="unlinkFood(resource.symbol)"
            >
              <Unlink :size="14" />
            </button>
          </div>
        </div>

        <div
          v-if="activeSymbol === resource.symbol"
          class="space-y-2 rounded border border-current/15 p-2"
        >
          <div class="text-xs font-semibold uppercase tracking-wide opacity-70">
            Map to FDC entry
          </div>
          <label class="field flex items-center gap-2">
            <Search :size="14" />
            <input
              v-model="searchQuery"
              placeholder="Search foods…"
              @keyup.enter="runSearch(resource.symbol)"
            />
          </label>
          <button
            class="secondary w-full justify-center"
            :disabled="searching"
            @click="runSearch(resource.symbol)"
          >
            <Sparkles :size="14" />{{ searching ? "Searching…" : "Fuzzy search FDC" }}
          </button>
          <button
            v-for="match in searchResults"
            :key="match.result.fdcId"
            class="w-full rounded border border-current/10 p-2 text-left text-sm hover:bg-current/5"
            @click="linkFood(resource.symbol, match.result)"
          >
            <strong class="block">{{ match.result.description }}</strong>
            <small class="opacity-70">
              FDC {{ match.result.fdcId }} · {{ match.result.dataType }}
              <span v-if="match.result.brandOwner"> · {{ match.result.brandOwner }}</span>
              · match {{ Math.round(match.score * 100) }}%
            </small>
          </button>
        </div>

        <div
          v-if="manualSymbol === resource.symbol"
          class="space-y-2 rounded border border-current/15 p-2"
        >
          <div class="text-xs font-semibold uppercase tracking-wide opacity-70">
            Manual facts per 100 g
          </div>
          <div class="grid grid-cols-2 gap-2">
            <label v-for="field in numericFields" :key="field[1]" class="field text-sm">
              <span>{{ field[0] }}</span>
              <input
                v-model.number="ensureManualDraft(resource.symbol)[field[1]]"
                type="number"
                min="0"
                step="0.1"
              />
            </label>
          </div>
          <div class="flex gap-2">
            <button
              class="primary flex-1 justify-center"
              :disabled="saving"
              @click="saveManualIngredient(resource.symbol)"
            >
              <Save :size="14" />{{ saving ? "Saving…" : "Save manual facts" }}
            </button>
            <button
              v-if="manualBySymbol[resource.symbol]"
              class="secondary"
              :disabled="saving"
              @click="clearManualIngredient(resource.symbol)"
            >
              Clear
            </button>
          </div>
        </div>
      </article>
      <p v-if="!ingredients.length" class="empty text-sm">
        No ingredient resources in this recipe.
      </p>
    </div>

    <div class="space-y-3 rounded border border-current/15 p-3">
      <div class="flex items-center justify-between gap-2">
        <div class="text-sm font-semibold">Recipe nutrition facts</div>
        <label class="flex items-center gap-2 text-sm">
          <input v-model="manualOverride" type="checkbox" />
          Manual override (whole recipe)
        </label>
      </div>
      <p class="text-xs opacity-70">
        {{
          manualOverride
            ? "Manual recipe facts override all ingredient-derived calculations."
            : "Facts are calculated from linked FDC entries and manual ingredient entries."
        }}
      </p>
      <div class="grid grid-cols-2 gap-2">
        <label class="field text-sm">
          <span>Servings</span>
          <input
            v-model.number="recipeFacts.servingsPerContainer"
            type="number"
            min="0"
            step="0.5"
          />
        </label>
        <label class="field text-sm">
          <span>Serving size</span>
          <input v-model="recipeFacts.servingSize" />
        </label>
        <label
          v-for="field in numericFields"
          :key="field[1]"
          class="field text-sm"
          :class="{ 'opacity-50': !manualOverride && !result }"
        >
          <span>{{ field[0] }}</span>
          <input
            v-model.number="recipeFacts[field[1]]"
            type="number"
            min="0"
            step="0.1"
            :readonly="!manualOverride"
          />
        </label>
      </div>
      <div class="flex flex-wrap gap-2">
        <button
          v-if="!manualOverride"
          class="secondary flex-1 justify-center"
          :disabled="busy || (!catalogAvailable && !Object.keys(manualBySymbol).length)"
          @click="calculate"
        >
          <Calculator :size="16" />{{ busy ? "Calculating…" : "Calculate from ingredients" }}
        </button>
        <button class="primary flex-1 justify-center" :disabled="saving" @click="saveRecipeFacts">
          <Save :size="16" />{{ saving ? "Saving…" : "Save nutrition" }}
        </button>
      </div>
    </div>

    <div v-if="result" class="space-y-2 rounded border border-current/15 p-3 text-sm">
      <div class="font-semibold">
        {{ result.manualOverride ? "Saved manual label" : "Calculated label (per serving)" }}
      </div>
      <dl class="grid grid-cols-2 gap-1">
        <div>
          <dt>Calories</dt>
          <dd>{{ Math.round(result.facts.calories) }}</dd>
        </div>
        <div>
          <dt>Protein</dt>
          <dd>{{ result.facts.proteinGrams.toFixed(1) }} g</dd>
        </div>
        <div>
          <dt>Total fat</dt>
          <dd>{{ result.facts.totalFatGrams.toFixed(1) }} g</dd>
        </div>
        <div>
          <dt>Carbs</dt>
          <dd>{{ result.facts.totalCarbohydrateGrams.toFixed(1) }} g</dd>
        </div>
        <div>
          <dt>Sodium</dt>
          <dd>{{ Math.round(result.facts.sodiumMilligrams) }} mg</dd>
        </div>
        <div v-if="result.calculated">
          <dt>Total mass</dt>
          <dd>{{ result.totalMassGrams.toFixed(0) }} g</dd>
        </div>
      </dl>
      <p v-if="result.calculated" class="opacity-70">
        {{ result.linkedIngredientCount }} of {{ result.totalIngredientCount }} ingredients sourced
        (FDC link or manual).
      </p>
      <ul v-if="result.warnings.length" class="space-y-1 text-xs opacity-80">
        <li v-for="warning in result.warnings" :key="warning">{{ warning }}</li>
      </ul>
      <div v-if="result.ingredients.length" class="space-y-1 border-t border-current/10 pt-2">
        <div class="text-xs font-semibold uppercase tracking-wide opacity-70">Breakdown</div>
        <div
          v-for="row in result.ingredients"
          :key="row.resourceSymbol"
          class="flex justify-between gap-2 text-xs"
        >
          <span>{{ row.resourceName || row.resourceSymbol }}</span>
          <span class="opacity-70">
            <template v-if="row.linked">FDC</template>
            <template v-else-if="row.manual">Manual</template>
            <template v-else>Unlinked</template>
            <template v-if="row.massGrams"> · {{ row.massGrams.toFixed(0) }} g</template>
          </span>
        </div>
      </div>
    </div>

    <p v-if="error" class="diagnostic error">{{ error }}</p>
  </section>
</template>

<style scoped>
.nutrition-link-btn.linked {
  color: var(--accent, #16a34a);
}
.nutrition-link-btn.manual {
  color: var(--accent, #2563eb);
}
</style>
