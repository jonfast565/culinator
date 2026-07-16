<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { Apple, Calculator, Link2, Search, Unlink } from "lucide-vue-next";
import type { UiResource } from "../../recipe-editor/model";
import type {
  NutritionSearchResult,
  RecipeNutritionResult,
  ResourceNutritionLink,
} from "../../../domain/types";
import * as api from "../../../services/api";

const props = defineProps<{
  recipeId: string;
  resources: UiResource[];
}>();

const catalogAvailable = ref(false);
const links = ref<ResourceNutritionLink[]>([]);
const result = ref<RecipeNutritionResult | null>(null);
const error = ref("");
const busy = ref(false);
const searchQuery = ref("");
const searchResults = ref<NutritionSearchResult[]>([]);
const activeSymbol = ref<string | null>(null);
const searching = ref(false);

const ingredients = computed(() =>
  props.resources.filter((resource) => resource.kind === "ingredient"),
);

const linkBySymbol = computed(() =>
  Object.fromEntries(links.value.map((link) => [link.resourceSymbol, link])),
);

async function refresh(): Promise<void> {
  const status = await api.getNutritionStatus();
  catalogAvailable.value = status.catalogAvailable;
  links.value = await api.listNutritionLinks(props.recipeId);
}

async function runSearch(): Promise<void> {
  if (!searchQuery.value.trim()) return;
  searching.value = true;
  error.value = "";
  try {
    searchResults.value = await api.searchNutritionFoods(searchQuery.value);
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

async function calculate(): Promise<void> {
  busy.value = true;
  error.value = "";
  try {
    result.value = await api.calculateRecipeNutrition(props.recipeId);
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    busy.value = false;
  }
}

onMounted(() => {
  void refresh();
});

watch(
  () => props.recipeId,
  () => {
    result.value = null;
    void refresh();
  },
);
</script>

<template>
  <section class="panel space-y-4">
    <div>
      <h3 class="flex items-center gap-2"><Apple :size="17" />Nutrition</h3>
      <p class="text-sm opacity-70">
        Link ingredients to USDA FoodData Central entries, then calculate Nutrition Facts from
        recipe quantities.
      </p>
    </div>

    <p v-if="!catalogAvailable" class="diagnostic warning text-sm">
      Nutrition database not found. Build <code>fdc.sqlite3</code> in the app data directory using
      <code>culinograph-fdc-build</code>.
    </p>

    <div class="space-y-2">
      <div class="text-sm font-semibold">Ingredients</div>
      <article v-for="resource in ingredients" :key="resource.symbol" class="card space-y-2">
        <div class="flex items-start justify-between gap-2">
          <div>
            <strong>{{ resource.name || resource.symbol }}</strong>
            <small class="block opacity-70">
              {{ resource.quantity || "no quantity" }}
            </small>
            <small v-if="linkBySymbol[resource.symbol]" class="block opacity-80">
              Linked: {{ linkBySymbol[resource.symbol].foodDescription }}
            </small>
          </div>
          <div class="flex gap-1">
            <button
              class="secondary"
              :disabled="!catalogAvailable || busy"
              @click="
                activeSymbol = activeSymbol === resource.symbol ? null : resource.symbol;
                searchQuery = resource.name || resource.symbol;
              "
            >
              <Link2 :size="14" />
            </button>
            <button
              v-if="linkBySymbol[resource.symbol]"
              class="secondary"
              :disabled="busy"
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
          <label class="field flex items-center gap-2">
            <Search :size="14" />
            <input v-model="searchQuery" placeholder="Search foods…" @keyup.enter="runSearch" />
          </label>
          <button class="secondary w-full justify-center" :disabled="searching" @click="runSearch">
            {{ searching ? "Searching…" : "Search FDC" }}
          </button>
          <button
            v-for="food in searchResults"
            :key="food.fdcId"
            class="w-full rounded border border-current/10 p-2 text-left text-sm hover:bg-current/5"
            @click="linkFood(resource.symbol, food)"
          >
            <strong class="block">{{ food.description }}</strong>
            <small class="opacity-70">
              FDC {{ food.fdcId }} · {{ food.dataType }}
              <span v-if="food.brandOwner"> · {{ food.brandOwner }}</span>
            </small>
          </button>
        </div>
      </article>
      <p v-if="!ingredients.length" class="empty text-sm">
        No ingredient resources in this recipe.
      </p>
    </div>

    <button
      class="primary w-full justify-center"
      :disabled="busy || !catalogAvailable"
      @click="calculate"
    >
      <Calculator :size="16" />{{ busy ? "Calculating…" : "Calculate nutrition facts" }}
    </button>

    <div v-if="result" class="space-y-2 rounded border border-current/15 p-3 text-sm">
      <div class="font-semibold">Calculated label (per serving)</div>
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
        <div>
          <dt>Total mass</dt>
          <dd>{{ result.totalMassGrams.toFixed(0) }} g</dd>
        </div>
      </dl>
      <p class="opacity-70">
        {{ result.linkedIngredientCount }} of {{ result.totalIngredientCount }} ingredients linked.
      </p>
      <ul v-if="result.warnings.length" class="space-y-1 text-xs opacity-80">
        <li v-for="warning in result.warnings" :key="warning">{{ warning }}</li>
      </ul>
    </div>

    <p v-if="error" class="diagnostic error">{{ error }}</p>
  </section>
</template>
