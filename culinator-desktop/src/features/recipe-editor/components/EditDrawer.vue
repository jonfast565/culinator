<script setup lang="ts">
/* global Event, HTMLInputElement */
import { ref, watch } from "vue";
import {
  Save,
  Trash2,
  Plus,
  Route,
  X,
  FileCode2,
  SlidersHorizontal,
  ListTree,
  ImagePlus,
  Loader2,
} from "lucide-vue-next";
import type { RecipeBookSummary, ValidationResult } from "../../../domain/types";
import type { UiRecipeModel } from "../model";
import { setRecipeProperty } from "../sourcePatch";
import { fileToBase64, uploadRecipeImage } from "../../../services/api";
import RecipeImage from "../../reading/components/RecipeImage.vue";
import SourceEditor from "./SourceEditor.vue";
import InspectorPanel from "./InspectorPanel.vue";
import StepPhotos from "./StepPhotos.vue";

const props = defineProps<{
  source: string;
  model: UiRecipeModel;
  validation: ValidationResult | null;
  recipeId?: string;
  books: RecipeBookSummary[];
  currentBookId: string | null;
  dirty: boolean;
  saving: boolean;
}>();
const emit = defineEmits<{
  (event: "update:source", value: string): void;
  (event: "save"): void;
  (event: "close"): void;
  (event: "move-book", bookId: string | null): void;
  (event: "delete"): void;
  (event: "insert-ingredient"): void;
  (event: "insert-operation"): void;
}>();

type DrawerTab = "details" | "source" | "tools";
const tab = ref<DrawerTab>("details");

// Structured fields patch the source in place; local refs keep the caret stable
// while the live preview re-parses.
const titleField = ref(props.model.title);
const sectionField = ref(props.model.section ?? "");
watch(
  () => props.model.title,
  (value) => {
    if (value !== titleField.value) titleField.value = value;
  },
);
watch(
  () => props.model.section,
  (value) => {
    const next = value ?? "";
    if (next !== sectionField.value) sectionField.value = next;
  },
);
function commitTitle(): void {
  emit("update:source", setRecipeProperty(props.source, "title", titleField.value));
}
function commitSection(): void {
  emit("update:source", setRecipeProperty(props.source, "section", sectionField.value));
}

// Cover image: either an external URL (stored as-is in the .cg) or an uploaded
// file (persisted, referenced by a generated asset handle).
const coverUrl = ref("");
const uploading = ref(false);
function setCover(reference: string): void {
  emit("update:source", setRecipeProperty(props.source, "image", reference));
}
function commitCoverUrl(): void {
  const value = coverUrl.value.trim();
  if (value) setCover(value);
}
async function onCoverFile(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file || !props.recipeId) return;
  uploading.value = true;
  try {
    const dataBase64 = await fileToBase64(file);
    const asset = await uploadRecipeImage(props.recipeId, {
      role: "cover",
      mediaType: file.type || "image/jpeg",
      fileName: file.name,
      dataBase64,
    });
    setCover(asset.handle);
  } finally {
    uploading.value = false;
  }
}
function removeCover(): void {
  coverUrl.value = "";
  setCover("");
}

const errorCount = () =>
  props.validation?.diagnostics.filter((d) => d.severity === "error").length ?? 0;
</script>

<template>
  <aside class="edit-drawer">
    <header class="drawer-head">
      <nav class="drawer-tabs">
        <button :class="{ active: tab === 'details' }" @click="tab = 'details'">
          <SlidersHorizontal :size="14" /> Details
        </button>
        <button :class="{ active: tab === 'source' }" @click="tab = 'source'">
          <FileCode2 :size="14" /> Source
        </button>
        <button :class="{ active: tab === 'tools' }" @click="tab = 'tools'">
          <ListTree :size="14" /> Tools
        </button>
      </nav>
      <div class="drawer-actions">
        <button
          class="primary"
          :disabled="!dirty || saving"
          :title="dirty ? 'Save changes' : 'No changes'"
          @click="emit('save')"
        >
          <Save :size="15" /> {{ saving ? "Saving…" : "Save" }}
        </button>
        <button class="icon" title="Done editing" @click="emit('close')"><X :size="16" /></button>
      </div>
    </header>

    <!-- Details: structured edit-in-place fields -->
    <div v-show="tab === 'details'" class="drawer-body details">
      <label class="field">
        <span>Title</span>
        <input v-model="titleField" type="text" @input="commitTitle" />
      </label>
      <label class="field">
        <span>Section (book chapter)</span>
        <input
          v-model="sectionField"
          type="text"
          placeholder="e.g. Mains, Desserts"
          @input="commitSection"
        />
      </label>
      <label class="field">
        <span>Book</span>
        <select
          :value="currentBookId ?? ''"
          @change="emit('move-book', ($event.target as HTMLSelectElement).value || null)"
        >
          <option value="">Unfiled</option>
          <option v-for="book in books" :key="book.id" :value="book.id">{{ book.title }}</option>
        </select>
      </label>

      <div class="field">
        <span>Cover image</span>
        <div v-if="model.coverImage" class="cover-preview">
          <RecipeImage :image-ref="model.coverImage" :recipe-id="recipeId" alt="Cover" />
          <button class="cover-remove" title="Remove cover" @click="removeCover">
            <X :size="14" />
          </button>
        </div>
        <div class="cover-controls">
          <input
            v-model="coverUrl"
            type="url"
            placeholder="Paste image URL…"
            @change="commitCoverUrl"
            @keyup.enter="commitCoverUrl"
          />
          <label class="upload-btn" :class="{ busy: uploading }">
            <Loader2 v-if="uploading" :size="14" class="spin" />
            <ImagePlus v-else :size="14" />
            {{ uploading ? "Uploading…" : "Upload" }}
            <input
              type="file"
              accept="image/*"
              hidden
              :disabled="uploading"
              @change="onCoverFile"
            />
          </label>
        </div>
        <small class="hint">A web link for online recipes, or upload a photo.</small>
      </div>

      <div v-if="model.operations.length" class="field">
        <span>Step photos</span>
        <StepPhotos
          :source="source"
          :model="model"
          :recipe-id="recipeId"
          @update:source="emit('update:source', $event)"
        />
        <small class="hint">Attach a photo to any step — a web link, or upload one.</small>
      </div>

      <div class="field">
        <span>Quick insert</span>
        <div class="quick-insert">
          <button @click="emit('insert-ingredient')"><Plus :size="14" /> Ingredient</button>
          <button @click="emit('insert-operation')"><Route :size="14" /> Step</button>
        </div>
      </div>

      <button class="danger wide" @click="emit('delete')">
        <Trash2 :size="15" /> Delete recipe
      </button>
    </div>

    <!-- Source: raw .cg editor -->
    <div v-show="tab === 'source'" class="drawer-body source">
      <SourceEditor :model-value="source" @update:model-value="emit('update:source', $event)" />
      <p v-if="errorCount()" class="diag error">
        {{ errorCount() }} error{{ errorCount() === 1 ? "" : "s" }} — see Tools › Diagnostics
      </p>
      <p v-else-if="validation && !validation.valid" class="diag warn">Recipe has warnings.</p>
    </div>

    <!-- Tools: advanced inspector (timeline, nutrition, HACCP, export…) -->
    <div v-show="tab === 'tools'" class="drawer-body tools">
      <InspectorPanel
        :model="model"
        :validation="validation"
        :recipe-id="recipeId"
        :source="source"
        @update:source="emit('update:source', $event)"
      />
    </div>
  </aside>
</template>

<style scoped>
.edit-drawer {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  background: #f7f6f2;
  border-left: 1px solid #d3d8d1;
}
.drawer-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  background: #fff;
  border-bottom: 1px solid #d8ddd9;
}
.drawer-tabs {
  display: flex;
  gap: 3px;
}
.drawer-tabs button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 6px 10px;
  font-size: 12px;
  background: transparent;
  border: 0;
  border-radius: 7px;
  color: #55635b;
}
.drawer-tabs button.active {
  background: #e4efe6;
  color: #28643b;
}
.drawer-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}
.drawer-actions button {
  height: 32px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 11px;
  font-size: 13px;
}
.drawer-actions .icon {
  width: 32px;
  padding: 0;
  justify-content: center;
}
.drawer-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
}
.details {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.field > span {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #6d7972;
}
.field input,
.field select {
  height: 34px;
  padding: 0 10px;
  border: 1px solid #cbd3cd;
  border-radius: 7px;
  background: #fff;
}
.cover-preview {
  position: relative;
  aspect-ratio: 16 / 9;
  border-radius: 7px;
  overflow: hidden;
  border: 1px solid #d3d8d1;
}
.cover-remove {
  position: absolute;
  top: 6px;
  right: 6px;
  width: 26px;
  height: 26px;
  padding: 0;
  display: grid;
  place-items: center;
  border-radius: 6px;
  border: 0;
  background: rgba(0, 0, 0, 0.55);
  color: #fff;
}
.cover-controls {
  display: flex;
  gap: 8px;
}
.cover-controls input[type="url"] {
  flex: 1;
  height: 34px;
  padding: 0 10px;
  border: 1px solid #cbd3cd;
  border-radius: 7px;
}
.upload-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding: 0 12px;
  border: 1px solid #cbd3cd;
  border-radius: 7px;
  background: #fff;
  font-size: 13px;
  color: #27342d;
  cursor: pointer;
  white-space: nowrap;
}
.upload-btn.busy {
  opacity: 0.7;
}
.hint {
  color: #8a938c;
  font-size: 11px;
}
.spin {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
@media (prefers-reduced-motion: reduce) {
  .spin {
    animation: none;
  }
}
.quick-insert {
  display: flex;
  gap: 8px;
}
.quick-insert button {
  flex: 1;
  height: 34px;
}
.danger.wide {
  margin-top: 6px;
  height: 36px;
  justify-content: center;
}
.source {
  display: flex;
  flex-direction: column;
}
.source .source-editor {
  flex: 1;
  min-height: 0;
}
.diag {
  margin: 0;
  padding: 8px 12px;
  font-size: 12px;
  border-top: 1px solid #e2e6e1;
}
.diag.error {
  color: #a83737;
  background: #fbeceb;
}
.diag.warn {
  color: #8a6d1f;
  background: #fbf6e7;
}
</style>
