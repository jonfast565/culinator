<script setup lang="ts">
import { ref, watch } from "vue";
import { ImageOff } from "lucide-vue-next";
import { getRecipeImage } from "../../../services/api";

// Renders a recipe image from a `.cg` reference that is either an external URL
// (used directly) or a stored asset handle (resolved to bytes via the image
// API / offline store). Tolerates missing assets with a quiet placeholder.
const props = defineProps<{
  imageRef?: string;
  recipeId?: string;
  alt?: string;
}>();

const src = ref<string | null>(null);
const failed = ref(false);

function isExternal(reference: string): boolean {
  return /^(https?:|data:)/i.test(reference);
}

async function load(): Promise<void> {
  src.value = null;
  failed.value = false;
  const reference = props.imageRef?.trim();
  if (!reference) return;
  if (isExternal(reference)) {
    src.value = reference;
    return;
  }
  if (!props.recipeId) return;
  const data = await getRecipeImage(props.recipeId, reference);
  if (data) src.value = `data:${data.asset.mediaType};base64,${data.dataBase64}`;
  else failed.value = true;
}

watch(() => [props.imageRef, props.recipeId], load, { immediate: true });
</script>

<template>
  <img v-if="src" :src="src" :alt="alt ?? ''" class="recipe-image" @error="failed = true" />
  <div v-else-if="failed" class="recipe-image-missing" :title="imageRef">
    <ImageOff :size="18" />
  </div>
</template>

<style scoped>
.recipe-image {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.recipe-image-missing {
  display: grid;
  place-items: center;
  width: 100%;
  min-height: 120px;
  height: 100%;
  color: #9aa39c;
  background: repeating-linear-gradient(45deg, #efeee7, #efeee7 8px, #e8e6dd 8px, #e8e6dd 16px);
}
</style>
