<script setup lang="ts">
/* global Event, HTMLInputElement */
import { ref, toRef } from "vue";
import { ImagePlus, Loader2, X } from "lucide-vue-next";
import type { UiOperation, UiRecipeModel } from "../model";
import { useRecipeNarrative } from "../narrative";
import { setOperationPhoto } from "../sourcePatch";
import { fileToBase64, uploadRecipeImage } from "../../../services/api";
import RecipeImage from "../../reading/components/RecipeImage.vue";

// Per-step photos: attach an image to any operation, either an external URL
// (stored as-is in the `.cg`) or an uploaded file (persisted, referenced by a
// generated asset handle). Mirrors the cover-image control in EditDrawer, but
// patches each operation's own `photo "…";` property via its source span.
const props = defineProps<{
  source: string;
  model: UiRecipeModel;
  recipeId?: string;
}>();
const emit = defineEmits<{ (event: "update:source", value: string): void }>();

// Reuse the reading-view narrative so step numbers, headings, and prose match
// exactly what the recipe page shows.
const { rows, describe } = useRecipeNarrative(toRef(props, "model"));

// Which step is mid-upload, keyed by operation symbol.
const uploadingSymbol = ref<string | null>(null);

function setPhoto(operation: UiOperation, reference: string): void {
  if (!operation.range) return;
  emit("update:source", setOperationPhoto(props.source, operation.range, reference));
}

function commitUrl(operation: UiOperation, event: Event): void {
  const input = event.target as HTMLInputElement;
  const value = input.value.trim();
  if (!value) return;
  input.value = "";
  setPhoto(operation, value);
}

async function onFile(operation: UiOperation, event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file || !props.recipeId) return;
  uploadingSymbol.value = operation.symbol;
  try {
    const dataBase64 = await fileToBase64(file);
    const asset = await uploadRecipeImage(props.recipeId, {
      role: "step",
      operationSymbol: operation.symbol,
      mediaType: file.type || "image/jpeg",
      fileName: file.name,
      dataBase64,
    });
    setPhoto(operation, asset.handle);
  } finally {
    uploadingSymbol.value = null;
  }
}
</script>

<template>
  <div class="step-photos">
    <template v-for="row in rows" :key="row.key">
      <p v-if="row.kind === 'heading'" class="group">{{ row.label }}</p>
      <div v-else class="step-row">
        <span class="num">{{ row.number }}</span>
        <div class="body">
          <p class="desc">{{ describe(row.operation!) }}</p>
          <div v-if="row.operation!.photo" class="thumb">
            <RecipeImage :image-ref="row.operation!.photo" :recipe-id="recipeId" />
            <button class="thumb-remove" title="Remove photo" @click="setPhoto(row.operation!, '')">
              <X :size="13" />
            </button>
          </div>
          <div class="controls">
            <input
              type="url"
              placeholder="Image URL…"
              @change="commitUrl(row.operation!, $event)"
              @keyup.enter="commitUrl(row.operation!, $event)"
            />
            <label
              class="upload"
              :class="{ busy: uploadingSymbol === row.operation!.symbol }"
              title="Upload a photo for this step"
            >
              <Loader2 v-if="uploadingSymbol === row.operation!.symbol" :size="13" class="spin" />
              <ImagePlus v-else :size="13" />
              <input
                type="file"
                accept="image/*"
                hidden
                :disabled="uploadingSymbol === row.operation!.symbol"
                @change="onFile(row.operation!, $event)"
              />
            </label>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.step-photos {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.group {
  margin: 6px 0 0;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #28643b;
}
.step-row {
  display: grid;
  grid-template-columns: 22px 1fr;
  gap: 10px;
  align-items: start;
}
.num {
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: #28643b;
  text-align: right;
  line-height: 1.5;
}
.body {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}
.desc {
  margin: 0;
  font-size: 13px;
  line-height: 1.4;
  color: #3a463f;
}
.thumb {
  position: relative;
  max-width: 220px;
  aspect-ratio: 4 / 3;
  overflow: hidden;
  border-radius: 6px;
  border: 1px solid #d3d8d1;
}
.thumb-remove {
  position: absolute;
  top: 5px;
  right: 5px;
  width: 24px;
  height: 24px;
  padding: 0;
  display: grid;
  place-items: center;
  border-radius: 6px;
  border: 0;
  background: rgba(0, 0, 0, 0.55);
  color: #fff;
}
.controls {
  display: flex;
  gap: 6px;
}
.controls input[type="url"] {
  flex: 1;
  min-width: 0;
  height: 30px;
  padding: 0 9px;
  border: 1px solid #cbd3cd;
  border-radius: 6px;
  background: #fff;
  font-size: 12px;
}
.upload {
  display: inline-grid;
  place-items: center;
  width: 34px;
  height: 30px;
  border: 1px solid #cbd3cd;
  border-radius: 6px;
  background: #fff;
  color: #27342d;
  cursor: pointer;
  flex: none;
}
.upload.busy {
  opacity: 0.7;
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
</style>
