<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Trash2, FolderInput, X, CheckSquare } from "lucide-vue-next";
import { sectionOf, summarize, type LoadedRecipe } from "../bookContents";
import type { RecipeBookSummary } from "../../../domain/types";

const props = defineProps<{
  recipes: LoadedRecipe[];
  books: RecipeBookSummary[];
}>();
const emit = defineEmits<{
  (event: "open-recipe", recipeId: string): void;
  (event: "bulk-move", ids: string[], bookId: string | null): void;
  (event: "bulk-delete", ids: string[]): void;
}>();

const selected = ref<Set<string>>(new Set());
const moveTarget = ref("");

// Drop selections that no longer exist after a refresh.
watch(
  () => props.recipes.map((recipe) => recipe.id).join("|"),
  () => {
    const present = new Set(props.recipes.map((recipe) => recipe.id));
    selected.value = new Set([...selected.value].filter((id) => present.has(id)));
  },
);

interface Group {
  section: string;
  items: { id: string; title: string; summary: string }[];
}
const groups = computed<Group[]>(() => {
  const order: string[] = [];
  const bySection = new Map<string, Group["items"]>();
  for (const recipe of props.recipes) {
    const section = sectionOf(recipe.model);
    if (!bySection.has(section)) {
      bySection.set(section, []);
      order.push(section);
    }
    bySection.get(section)!.push({
      id: recipe.id,
      title: recipe.model.title || "Untitled recipe",
      summary: summarize(recipe.model),
    });
  }
  return order.map((section) => ({ section, items: bySection.get(section)! }));
});

const selectedCount = computed(() => selected.value.size);
const allSelected = computed(
  () => props.recipes.length > 0 && selected.value.size === props.recipes.length,
);

function toggle(id: string): void {
  const next = new Set(selected.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  selected.value = next;
}
function toggleAll(): void {
  selected.value = allSelected.value ? new Set() : new Set(props.recipes.map((r) => r.id));
}
function clearSelection(): void {
  selected.value = new Set();
}
function doMove(): void {
  const target = moveTarget.value === "__unfiled" ? null : moveTarget.value || null;
  emit("bulk-move", [...selected.value], target);
  moveTarget.value = "";
  clearSelection();
}
function doDelete(): void {
  emit("bulk-delete", [...selected.value]);
  clearSelection();
}
</script>

<template>
  <div class="manage">
    <div class="manage-scroll">
      <div class="manage-head">
        <button class="select-all" @click="toggleAll">
          <CheckSquare :size="15" /> {{ allSelected ? "Clear all" : "Select all" }}
        </button>
        <span class="count">{{ recipes.length }} recipe{{ recipes.length === 1 ? "" : "s" }}</span>
      </div>

      <div v-for="group in groups" :key="group.section" class="manage-group">
        <h3 class="manage-section">{{ group.section }}</h3>
        <ul class="manage-list">
          <li
            v-for="item in group.items"
            :key="item.id"
            class="manage-row"
            :class="{ selected: selected.has(item.id) }"
          >
            <label class="check" @click.stop>
              <input type="checkbox" :checked="selected.has(item.id)" @change="toggle(item.id)" />
            </label>
            <button class="row-open" @click="emit('open-recipe', item.id)">
              <span class="row-title">{{ item.title }}</span>
              <span class="row-summary">{{ item.summary }}</span>
            </button>
          </li>
        </ul>
      </div>
    </div>

    <div v-if="selectedCount" class="bulk-bar">
      <span class="bulk-count">{{ selectedCount }} selected</span>
      <div class="bulk-actions">
        <label class="bulk-move">
          <FolderInput :size="15" />
          <select v-model="moveTarget" @change="doMove">
            <option value="" disabled selected>Move to…</option>
            <option value="__unfiled">Unfiled</option>
            <option v-for="book in books" :key="book.id" :value="book.id">{{ book.title }}</option>
          </select>
        </label>
        <button class="danger" @click="doDelete"><Trash2 :size="15" /> Delete</button>
        <button class="icon" title="Clear selection" @click="clearSelection">
          <X :size="16" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.manage {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.manage-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  max-width: 760px;
  width: 100%;
  margin: 0 auto;
  padding: 20px 20px 40px;
}
.manage-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
  margin-bottom: 12px;
  border-bottom: 1px solid rgba(60, 50, 30, 0.16);
}
.select-all {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 12px;
  font-size: 13px;
  background: rgba(255, 255, 255, 0.7);
  border: 1px solid #cbd3cd;
  border-radius: 8px;
  color: #23302a;
}
.count {
  font-size: 12px;
  color: #6d7972;
}
.manage-section {
  margin: 18px 0 8px;
  font-family: "Iowan Old Style", "Palatino Linotype", Palatino, Georgia, serif;
  font-size: 15px;
  font-weight: 600;
  color: #28643b;
}
.manage-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.manage-row {
  display: flex;
  align-items: stretch;
  gap: 10px;
  background: #fbf9f3;
  border: 1px solid #e3ddcd;
  border-radius: 8px;
  overflow: hidden;
}
.manage-row.selected {
  border-color: #28643b;
  box-shadow: 0 0 0 1px #28643b inset;
}
.check {
  display: flex;
  align-items: center;
  padding: 0 4px 0 12px;
}
.check input {
  width: 16px;
  height: 16px;
  accent-color: #28643b;
}
.row-open {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  padding: 11px 14px 11px 4px;
  background: transparent;
  border: 0;
  text-align: left;
  cursor: pointer;
}
.row-open:hover {
  background: rgba(40, 100, 59, 0.05);
}
.row-title {
  font-size: 15px;
  color: #23302a;
}
.row-summary {
  font-size: 12px;
  color: #6d7972;
}
.bulk-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 18px;
  background: #23302a;
  color: #f2efe6;
}
.bulk-count {
  font-size: 13px;
  font-weight: 600;
}
.bulk-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.bulk-move {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 10px;
  height: 34px;
  background: #2f4034;
  border-radius: 8px;
  color: #dfe9e0;
}
.bulk-move select {
  height: 34px;
  background: transparent;
  border: 0;
  color: #dfe9e0;
  font-size: 13px;
}
.bulk-move select option {
  color: #23302a;
}
.bulk-actions .danger {
  height: 34px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  background: #7a2f2f;
  color: #fff;
  border: 0;
  border-radius: 8px;
}
.bulk-actions .icon {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  background: transparent;
  border: 1px solid #3d4c41;
  border-radius: 8px;
  color: #dfe9e0;
}
</style>
