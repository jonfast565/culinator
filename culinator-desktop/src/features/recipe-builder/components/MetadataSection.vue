<script setup lang="ts">
/* global Event, HTMLInputElement */
import { ref } from "vue";
import { ImagePlus, Loader2, X } from "lucide-vue-next";
import { fileToBase64, uploadRecipeImage } from "../../../services/api";
import RecipeImage from "../../reading/components/RecipeImage.vue";
import type { BuilderMetadata } from "../composables/useRecipeBuilder";
import BuilderTextField from "./BuilderTextField.vue";

const props = defineProps<{
  metadata: BuilderMetadata;
  recipeId?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{ commit: [key: string, value: string] }>();

function commit(key: string, value: string): void {
  emit("commit", key, value);
}

const coverUrl = ref("");
const uploading = ref(false);

function commitCoverUrl(): void {
  const value = coverUrl.value.trim();
  if (value) {
    emit("commit", "image", value);
    coverUrl.value = "";
  }
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
    emit("commit", "image", asset.handle);
  } finally {
    uploading.value = false;
  }
}
function removeCover(): void {
  emit("commit", "image", "");
}
</script>

<template>
  <section id="builder-details" class="panel builder-section">
    <div class="panel-header">
      <h3>Details</h3>
    </div>

    <fieldset :disabled="disabled" class="fields">
      <BuilderTextField
        label="Title"
        :model-value="metadata.title"
        placeholder="e.g. Pizza Dough"
        @commit="commit('title', $event)"
      />
      <div class="field-row">
        <BuilderTextField
          label="Section (book chapter)"
          :model-value="metadata.section"
          placeholder="e.g. Mains"
          @commit="commit('section', $event)"
        />
        <BuilderTextField
          label="Publisher"
          :model-value="metadata.publisher"
          @commit="commit('publisher', $event)"
        />
      </div>
      <BuilderTextField
        label="Description"
        :model-value="metadata.description"
        multiline
        placeholder="A short introduction to the recipe"
        @commit="commit('description', $event)"
      />
      <div class="field-row">
        <BuilderTextField
          label="Active time"
          :model-value="metadata.activeTime"
          placeholder="e.g. 20 min"
          @commit="commit('active_time', $event)"
        />
        <BuilderTextField
          label="Total time"
          :model-value="metadata.totalTime"
          placeholder="e.g. 45 min"
          @commit="commit('total_time', $event)"
        />
      </div>
      <BuilderTextField
        label="Source"
        :model-value="metadata.source"
        placeholder="Where the recipe came from"
        @commit="commit('source', $event)"
      />
      <BuilderTextField
        label="Source URL"
        :model-value="metadata.sourceUrl"
        type="url"
        placeholder="https://…"
        @commit="commit('source_url', $event)"
      />
      <BuilderTextField
        label="Attribution"
        :model-value="metadata.attribution"
        multiline
        placeholder="Credit line, licence, etc."
        @commit="commit('attribution', $event)"
      />

      <div class="cover">
        <span class="cover-label">Cover image</span>
        <div v-if="metadata.coverImage" class="cover-preview">
          <RecipeImage :image-ref="metadata.coverImage" :recipe-id="recipeId" alt="Cover" />
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
    </fieldset>
  </section>
</template>

<style scoped>
.fields {
  border: 0;
  padding: 0;
  margin: 0;
  display: grid;
  gap: 14px;
}
.fields:disabled {
  opacity: 0.55;
}
.field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
.cover {
  display: grid;
  gap: 6px;
}
.cover-label {
  font-size: 12px;
  color: #657169;
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
}
.upload-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
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
@media (max-width: 620px) {
  .field-row {
    grid-template-columns: 1fr;
  }
}
</style>
