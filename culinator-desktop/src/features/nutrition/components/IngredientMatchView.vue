<script setup lang="ts">
/* global HTMLInputElement, KeyboardEvent */
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  Apple,
  ChevronLeft,
  Link2,
  Loader2,
  Search,
  Sparkles,
  Unlink,
  Wand2,
} from "lucide-vue-next";
import type { UiResource } from "../../recipe-editor/model";
import type {
  FuzzyFoodMatch,
  NutritionSearchResult,
  RecipeNutritionState,
} from "../../../domain/types";
import * as api from "../../../services/api";

const props = defineProps<{
  recipeId: string;
  resources: UiResource[];
  recipeTitle?: string;
  initialSymbol?: string | null;
}>();

const emit = defineEmits<{ (event: "back"): void }>();

type SearchMode = "fuzzy" | "catalog";

const catalogAvailable = ref(false);
const state = ref<RecipeNutritionState | null>(null);
const selectedSymbol = ref<string | null>(null);
const searchQuery = ref("");
const searchMode = ref<SearchMode>("fuzzy");
const hideBranded = ref(true);
const matches = ref<FuzzyFoodMatch[]>([]);
const searching = ref(false);
const busy = ref(false);
const autoLinking = ref(false);
const error = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
let searchTimer = 0;

const ingredients = computed(() =>
  props.resources.filter((resource) => resource.kind === "ingredient"),
);

const linkBySymbol = computed(() =>
  Object.fromEntries((state.value?.links ?? []).map((link) => [link.resourceSymbol, link])),
);

const selectedResource = computed(
  () => ingredients.value.find((resource) => resource.symbol === selectedSymbol.value) ?? null,
);

const selectedLink = computed(() =>
  selectedSymbol.value ? (linkBySymbol.value[selectedSymbol.value] ?? null) : null,
);

const visibleMatches = computed(() => {
  if (!hideBranded.value) return matches.value;
  return matches.value.filter((match) => !isBranded(match.result.dataType));
});

const linkedCount = computed(() =>
  ingredients.value.filter((resource) => linkBySymbol.value[resource.symbol]).length,
);

function isBranded(dataType: string): boolean {
  const lowered = dataType.toLowerCase().replace(/[\s-]/g, "_");
  return lowered === "branded_food" || lowered === "branded";
}

function dataTypeClass(dataType: string): string {
  const lowered = dataType.toLowerCase().replace(/[\s-]/g, "_");
  if (lowered.includes("foundation")) return "foundation";
  if (lowered.includes("sr_legacy") || lowered.includes("sr_legacy_food")) return "legacy";
  if (lowered.includes("survey") || lowered.includes("fndds")) return "survey";
  if (isBranded(dataType)) return "branded";
  return "other";
}

function scorePercent(score: number): number {
  return Math.max(0, Math.round(score * 100));
}

async function refresh(): Promise<void> {
  const status = await api.getNutritionStatus();
  catalogAvailable.value = status.catalogAvailable;
  state.value = await api.getNutritionState(props.recipeId);
}

async function runSearch(): Promise<void> {
  const query = searchQuery.value.trim();
  if (!query) {
    matches.value = [];
    return;
  }
  searching.value = true;
  error.value = "";
  const excludeBranded = hideBranded.value;
  try {
    if (searchMode.value === "fuzzy") {
      matches.value = await api.fuzzyMatchNutritionFoods(query, 20, { excludeBranded });
      if (matches.value.length === 0) {
        matches.value = (await api.searchNutritionFoods(query, 20, { excludeBranded })).map(
          (result, index) => ({
            result,
            score: Math.max(0.2, 1 - index * 0.03),
          }),
        );
      }
    } else {
      matches.value = (await api.searchNutritionFoods(query, 30, { excludeBranded })).map(
        (result, index) => ({
          result,
          score: Math.max(0.15, 1 - index * 0.02),
        }),
      );
    }
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
    matches.value = [];
  } finally {
    searching.value = false;
  }
}

function scheduleSearch(): void {
  window.clearTimeout(searchTimer);
  searchTimer = window.setTimeout(() => void runSearch(), 220);
}

function selectIngredient(symbol: string): void {
  selectedSymbol.value = symbol;
  const resource = ingredients.value.find((item) => item.symbol === symbol);
  searchQuery.value = resource?.name || resource?.symbol || symbol;
  void nextTick(() => searchInput.value?.focus());
  void runSearch();
}

async function linkFood(food: NutritionSearchResult): Promise<void> {
  if (!selectedSymbol.value) return;
  busy.value = true;
  error.value = "";
  try {
    await api.linkIngredientNutrition(props.recipeId, selectedSymbol.value, food.fdcId);
    await refresh();
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
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    busy.value = false;
  }
}

async function autoLinkAll(): Promise<void> {
  autoLinking.value = true;
  error.value = "";
  try {
    const result = await api.autoLinkIngredients(props.recipeId);
    await refresh();
    if (selectedSymbol.value) void runSearch();
    if (result.linked.length === 0 && result.skipped.length > 0) {
      error.value = `No confident matches for ${result.skipped.length} ingredient(s). Try searching manually.`;
    }
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    autoLinking.value = false;
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape") emit("back");
}

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
  void refresh().then(() => {
    const preferred =
      props.initialSymbol &&
      ingredients.value.some((resource) => resource.symbol === props.initialSymbol)
        ? props.initialSymbol
        : (ingredients.value[0]?.symbol ?? null);
    if (preferred) selectIngredient(preferred);
  });
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
  window.clearTimeout(searchTimer);
});

watch(
  () => props.recipeId,
  () => {
    void refresh();
  },
);

watch(
  () => props.initialSymbol,
  (symbol) => {
    if (symbol && ingredients.value.some((resource) => resource.symbol === symbol)) {
      selectIngredient(symbol);
    }
  },
);

watch(searchMode, () => {
  void runSearch();
});

watch(hideBranded, () => {
  void runSearch();
});
</script>

<template>
  <div class="match-view">
    <header class="match-head">
      <button class="ghost" @click="emit('back')"><ChevronLeft :size="16" /> Back</button>
      <div class="brand">
        <span class="brand-mark"><Apple :size="18" /></span>
        <span>
          <strong>Ingredient matcher</strong>
          <small>{{ recipeTitle || "Link recipe ingredients to USDA foods" }}</small>
        </span>
      </div>
      <div class="head-actions">
        <span class="link-count">{{ linkedCount }} / {{ ingredients.length }} linked</span>
        <button
          class="secondary"
          :disabled="autoLinking || !catalogAvailable || busy"
          @click="autoLinkAll"
        >
          <Wand2 :size="14" />{{ autoLinking ? "Matching…" : "Auto-link all" }}
        </button>
      </div>
    </header>

    <p v-if="!catalogAvailable" class="banner warning">
      Nutrition database not found. Build <code>fdc.sqlite3</code> to search FoodData Central.
    </p>
    <p v-if="error" class="banner error">{{ error }}</p>

    <div class="match-body">
      <aside class="ingredient-rail">
        <div class="rail-label">Recipe ingredients</div>
        <button
          v-for="resource in ingredients"
          :key="resource.symbol"
          type="button"
          class="ingredient-row"
          :class="{
            selected: selectedSymbol === resource.symbol,
            linked: Boolean(linkBySymbol[resource.symbol]),
          }"
          @click="selectIngredient(resource.symbol)"
        >
          <span class="status-dot" aria-hidden="true" />
          <span class="ingredient-copy">
            <strong>{{ resource.name || resource.symbol }}</strong>
            <small>{{ resource.quantity || "no quantity" }}</small>
            <small v-if="linkBySymbol[resource.symbol]" class="linked-desc">
              {{ linkBySymbol[resource.symbol].foodDescription }}
            </small>
          </span>
        </button>
        <p v-if="!ingredients.length" class="empty">No ingredients in this recipe.</p>
      </aside>

      <main class="search-stage">
        <section v-if="selectedResource" class="selected-card">
          <div>
            <div class="eyebrow">Matching</div>
            <h1>{{ selectedResource.name || selectedResource.symbol }}</h1>
            <p>
              {{ selectedResource.quantity || "No quantity" }}
              <template v-if="selectedResource.size"> · {{ selectedResource.size }}</template>
            </p>
          </div>
          <div v-if="selectedLink" class="current-link">
            <div>
              <div class="eyebrow">Current link</div>
              <strong>{{ selectedLink.foodDescription }}</strong>
              <small>FDC {{ selectedLink.fdcId }}</small>
            </div>
            <button class="secondary" :disabled="busy" @click="unlinkFood(selectedResource.symbol)">
              <Unlink :size="14" /> Unlink
            </button>
          </div>
          <p v-else class="unlinked-note">Not linked yet — pick a match below.</p>
        </section>

        <section class="search-panel">
          <div class="mode-tabs" role="tablist">
            <button
              type="button"
              role="tab"
              :aria-selected="searchMode === 'fuzzy'"
              :class="{ active: searchMode === 'fuzzy' }"
              @click="searchMode = 'fuzzy'"
            >
              <Sparkles :size="14" /> Fuzzy match
            </button>
            <button
              type="button"
              role="tab"
              :aria-selected="searchMode === 'catalog'"
              :class="{ active: searchMode === 'catalog' }"
              @click="searchMode = 'catalog'"
            >
              <Search :size="14" /> Catalog search
            </button>
          </div>

          <label class="search-input">
            <Search :size="16" />
            <input
              ref="searchInput"
              v-model="searchQuery"
              type="search"
              placeholder="Search foods… e.g. butter, whole milk, all-purpose flour"
              aria-label="Search USDA foods"
              :disabled="!catalogAvailable"
              @input="scheduleSearch"
              @keyup.enter="runSearch"
            />
            <Loader2 v-if="searching" :size="16" class="spin" />
          </label>

          <div class="search-meta">
            <label class="toggle">
              <input v-model="hideBranded" type="checkbox" />
              Hide branded products
            </label>
            <span v-if="matches.length" class="result-count">
              {{ visibleMatches.length }}
              <template v-if="hideBranded && visibleMatches.length !== matches.length">
                of {{ matches.length }}
              </template>
              result{{ visibleMatches.length === 1 ? "" : "s" }}
            </span>
          </div>
        </section>

        <ul v-if="visibleMatches.length" class="results">
          <li v-for="match in visibleMatches" :key="match.result.fdcId">
            <div class="result-copy">
              <div class="result-title">
                <strong>{{ match.result.description }}</strong>
                <span class="score" :title="`Match score ${match.score.toFixed(2)}`">
                  {{ scorePercent(match.score) }}%
                </span>
              </div>
              <div class="result-meta">
                <span class="type-pill" :class="dataTypeClass(match.result.dataType)">
                  {{ match.result.dataType }}
                </span>
                <span>FDC {{ match.result.fdcId }}</span>
                <span v-if="match.result.brandOwner">{{ match.result.brandOwner }}</span>
                <span v-if="match.result.servingSize">
                  serving {{ match.result.servingSize
                  }}{{ match.result.servingSizeUnit ? ` ${match.result.servingSizeUnit}` : "" }}
                </span>
              </div>
            </div>
            <button
              class="primary"
              :disabled="busy || !selectedSymbol"
              :class="{
                linked: selectedLink?.fdcId === match.result.fdcId,
              }"
              @click="linkFood(match.result)"
            >
              <Link2 :size="14" />
              {{ selectedLink?.fdcId === match.result.fdcId ? "Linked" : "Link" }}
            </button>
          </li>
        </ul>
        <p v-else-if="searching" class="empty stage-empty">
          <Loader2 :size="16" class="spin" /> Searching
          {{ hideBranded ? "generic foods…" : "catalog…" }}
        </p>
        <p v-else-if="searchQuery.trim()" class="empty stage-empty">
          No {{ hideBranded ? "generic " : "" }}matches. Try catalog search or a more specific name.
        </p>
        <p v-else-if="!selectedResource" class="empty stage-empty">
          Select an ingredient to start matching.
        </p>
      </main>
    </div>
  </div>
</template>

<style scoped>
.match-view {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: radial-gradient(120% 80% at 50% -10%, #efece2 0%, #e7e3d6 55%, #e0dbcb 100%);
}
.match-head {
  display: flex;
  align-items: center;
  gap: 16px;
  min-height: 72px;
  padding: 12px 18px;
  background: white;
  border-bottom: 1px solid #d8ddd9;
}
.match-head .ghost {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 34px;
  padding: 0 12px;
  font-size: 13px;
}
.brand {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}
.brand-mark {
  display: grid;
  place-items: center;
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: #d9f0df;
  color: #1f5130;
}
.brand strong {
  display: block;
  font-size: 17px;
}
.brand small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #718078;
}
.head-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}
.link-count {
  font-size: 13px;
  color: #5f6d65;
}
.banner {
  margin: 0;
  padding: 10px 18px;
  font-size: 13px;
}
.banner.warning {
  background: #fff6df;
  color: #6a5414;
  border-bottom: 1px solid #ead9a8;
}
.banner.error {
  background: #fdeceb;
  color: #8a2b2b;
  border-bottom: 1px solid #efc4c4;
}
.match-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(240px, 300px) 1fr;
}
.ingredient-rail {
  min-height: 0;
  overflow: auto;
  padding: 14px 12px;
  border-right: 1px solid #d8ddd9;
  background: rgba(255, 255, 255, 0.55);
}
.rail-label {
  margin: 0 8px 10px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #718078;
}
.ingredient-row {
  width: 100%;
  display: flex;
  gap: 10px;
  align-items: flex-start;
  margin-bottom: 6px;
  padding: 10px 10px;
  border: 1px solid transparent;
  border-radius: 10px;
  background: transparent;
  text-align: left;
  cursor: pointer;
}
.ingredient-row:hover {
  background: rgba(255, 255, 255, 0.72);
  border-color: #d8ddd9;
}
.ingredient-row.selected {
  background: #fff;
  border-color: #b7c9bb;
  box-shadow: 0 1px 0 rgba(31, 81, 48, 0.06);
}
.status-dot {
  flex: 0 0 auto;
  width: 8px;
  height: 8px;
  margin-top: 6px;
  border-radius: 999px;
  background: #c5cdc7;
}
.ingredient-row.linked .status-dot {
  background: #2f8a4c;
}
.ingredient-copy {
  min-width: 0;
  display: grid;
  gap: 2px;
}
.ingredient-copy strong {
  font-size: 14px;
}
.ingredient-copy small {
  color: #6d7972;
  font-size: 12px;
}
.linked-desc {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  color: #3f6b4d !important;
}
.search-stage {
  min-height: 0;
  overflow: auto;
  padding: clamp(18px, 3vw, 28px);
  display: grid;
  gap: 16px;
  align-content: start;
}
.selected-card {
  display: grid;
  gap: 14px;
  padding: 18px 20px;
  border: 1px solid #d8ddd9;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.92);
}
.eyebrow {
  margin-bottom: 4px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #718078;
}
.selected-card h1 {
  margin: 0;
  font-size: 24px;
  letter-spacing: -0.02em;
}
.selected-card p {
  margin: 4px 0 0;
  color: #5f6d65;
}
.current-link {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  padding-top: 12px;
  border-top: 1px solid #e5ebe7;
}
.current-link strong {
  display: block;
  font-size: 14px;
}
.current-link small {
  color: #6d7972;
}
.unlinked-note {
  margin: 0;
  padding-top: 4px;
  color: #7a6a3a;
  font-size: 13px;
}
.search-panel {
  display: grid;
  gap: 12px;
  padding: 16px 18px;
  border: 1px solid #d8ddd9;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.92);
}
.mode-tabs {
  display: flex;
  gap: 6px;
}
.mode-tabs button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding: 0 12px;
  border: 1px solid #cbd3cd;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.7);
  color: #4a5a52;
  font-size: 13px;
}
.mode-tabs button.active {
  border-color: #28643b;
  background: #e7f4eb;
  color: #1f5130;
}
.search-input {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 44px;
  padding: 0 14px;
  border: 1px solid #cbd3cd;
  border-radius: 999px;
  background: #fff;
}
.search-input input {
  flex: 1;
  border: 0;
  background: transparent;
  outline: none;
  font-size: 15px;
}
.search-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 13px;
  color: #5f6d65;
}
.toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}
.results {
  list-style: none;
  margin: 0;
  padding: 0;
  border: 1px solid #d8ddd9;
  border-radius: 14px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.95);
}
.results li {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
}
.results li + li {
  border-top: 1px solid #e5ebe7;
}
.result-copy {
  flex: 1;
  min-width: 0;
  display: grid;
  gap: 6px;
}
.result-title {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}
.result-title strong {
  flex: 1;
  font-size: 14px;
  line-height: 1.35;
}
.score {
  flex: 0 0 auto;
  min-width: 44px;
  padding: 2px 8px;
  border-radius: 999px;
  background: #eef5ef;
  color: #1f5130;
  font-size: 12px;
  font-weight: 700;
  text-align: center;
}
.result-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 12px;
  color: #6d7972;
  font-size: 12px;
}
.type-pill {
  padding: 1px 8px;
  border-radius: 999px;
  font-weight: 600;
}
.type-pill.foundation {
  background: #d9f0df;
  color: #1f5130;
}
.type-pill.legacy {
  background: #e3eef8;
  color: #1d4a73;
}
.type-pill.survey {
  background: #f3e9d4;
  color: #6a5414;
}
.type-pill.branded {
  background: #ececec;
  color: #555;
}
.type-pill.other {
  background: #ececec;
  color: #555;
}
.results .primary {
  flex: 0 0 auto;
}
.results .primary.linked {
  background: #2f8a4c;
}
.empty {
  margin: 12px 8px;
  color: #6d7972;
  font-size: 13px;
}
.stage-empty {
  margin: 0;
  padding: 28px 18px;
  border: 1px dashed #cbd3cd;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.45);
  text-align: center;
}
.spin {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
@media (max-width: 860px) {
  .match-body {
    grid-template-columns: 1fr;
  }
  .ingredient-rail {
    max-height: 220px;
    border-right: 0;
    border-bottom: 1px solid #d8ddd9;
  }
  .head-actions .secondary span,
  .link-count {
    display: none;
  }
}
</style>
