<script setup lang="ts">
/* global File, Event, HTMLInputElement, btoa */
import { onMounted, ref } from "vue";
import { Camera, FileImage, KeyRound, LoaderCircle, X } from "lucide-vue-next";
import type {
  PublicImportSettings,
  RecipeImageInput,
  RecipeImportResult,
} from "../../../domain/types";
import {
  getImportSettings,
  importStructured,
  clearStoredApiKey,
  translateRecipeImages,
  updateImportSettings,
} from "../../../services/api";
const emit = defineEmits<{ close: []; accept: [source: string, title: string] }>();
const tab = ref<"photos" | "structured">("photos");
const settings = ref<PublicImportSettings>({
  apiKeyConfigured: false,
  openaiModel: "gpt-4.1-mini",
  useLocalOcr: true,
  tesseractCommand: "tesseract",
});
const apiKey = ref("");
const targetLanguage = ref("English");
const files = ref<File[]>([]);
const previews = ref<string[]>([]);
const result = ref<RecipeImportResult>();
const structuredDraft = ref<{ title: string; sourceText: string; warnings: string[] }>();
const structuredFormat = ref<"json_ld" | "json" | "yaml">("json_ld");
const structuredContent = ref("");
const busy = ref(false);
const error = ref("");
onMounted(async () => {
  try {
    settings.value = await getImportSettings();
  } catch (e) {
    error.value = String(e);
  }
});
function selected(event: Event) {
  const input = event.target as HTMLInputElement;
  files.value = [...(input.files ?? [])];
  previews.value.forEach(URL.revokeObjectURL);
  previews.value = files.value.map(URL.createObjectURL);
  result.value = undefined;
}
async function saveSettings() {
  settings.value = await updateImportSettings({
    openaiApiKey: apiKey.value || undefined,
    openaiModel: settings.value.openaiModel,
    useLocalOcr: settings.value.useLocalOcr,
    tesseractCommand: settings.value.tesseractCommand,
  });
  apiKey.value = "";
}
async function clearKey(): Promise<void> {
  settings.value = await clearStoredApiKey();
}
async function importStructuredRecipe(): Promise<void> {
  busy.value = true;
  error.value = "";
  try {
    structuredDraft.value = await importStructured({
      format: structuredFormat.value,
      content: structuredContent.value,
    });
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}
async function data(file: File): Promise<RecipeImageInput> {
  const buffer = await file.arrayBuffer();
  let binary = "";
  for (const byte of new Uint8Array(buffer)) binary += String.fromCharCode(byte);
  return { fileName: file.name, mediaType: file.type || "image/jpeg", dataBase64: btoa(binary) };
}
async function translate() {
  busy.value = true;
  error.value = "";
  try {
    await saveSettings();
    result.value = await translateRecipeImages(
      await Promise.all(files.value.map(data)),
      targetLanguage.value,
    );
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
    <section
      class="max-h-[92vh] w-full max-w-5xl overflow-auto rounded-2xl bg-white p-6 shadow-2xl"
    >
      <header class="mb-5 flex items-center justify-between">
        <div>
          <h2 class="text-xl font-semibold">Import recipe</h2>
          <p class="text-sm text-slate-500">
            Photos, JSON-LD, JSON, or YAML → validated Culinator DSL.
          </p>
        </div>
        <button @click="emit('close')"><X /></button>
      </header>
      <div class="mb-4 flex gap-2">
        <button :class="{ active: tab === 'photos' }" @click="tab = 'photos'">Photos / OCR</button>
        <button :class="{ active: tab === 'structured' }" @click="tab = 'structured'">
          Paste JSON-LD / JSON / YAML
        </button>
      </div>
      <div class="grid gap-6 lg:grid-cols-2">
        <div class="space-y-4">
          <div class="rounded-xl border p-4">
            <h3 class="mb-3 flex items-center gap-2 font-medium">
              <KeyRound :size="18" />AI settings
            </h3>
            <p v-if="settings.apiKeyConfigured" class="mb-2 text-xs text-green-700">
              API key stored in
              {{ settings.secretStoreBackend ?? "secure storage" }}.
              <button class="underline" @click="clearKey">Clear stored key</button>
            </p>
            <label class="block text-sm"
              >OpenAI API key
              <input
                v-model="apiKey"
                type="password"
                class="mt-1 w-full rounded border p-2"
                :placeholder="
                  settings.apiKeyConfigured ? 'Configured — enter to replace' : 'sk-…'
                " /></label
            ><label class="mt-3 block text-sm"
              >Model
              <input v-model="settings.openaiModel" class="mt-1 w-full rounded border p-2" /></label
            ><label class="mt-3 flex items-center gap-2 text-sm"
              ><input v-model="settings.useLocalOcr" type="checkbox" />Try local Tesseract OCR
              before AI</label
            ><label v-if="settings.useLocalOcr" class="mt-3 block text-sm"
              >Tesseract command
              <input v-model="settings.tesseractCommand" class="mt-1 w-full rounded border p-2"
            /></label>
          </div>
          <template v-if="tab === 'photos'">
            <div class="rounded-xl border border-dashed p-5 text-center">
              <Camera class="mx-auto mb-2" />
              <p class="mb-3 text-sm text-slate-600">
                On phones, use the camera. On desktop, choose one or more images.
              </p>
              <label
                class="inline-flex cursor-pointer items-center gap-2 rounded-lg bg-slate-900 px-4 py-2 text-white"
                ><FileImage :size="17" />Camera or images<input
                  type="file"
                  accept="image/*"
                  capture="environment"
                  multiple
                  class="hidden"
                  @change="selected"
              /></label>
              <div v-if="previews.length" class="mt-4 grid grid-cols-3 gap-2">
                <img
                  v-for="src in previews"
                  :key="src"
                  :src="src"
                  class="h-28 w-full rounded object-cover"
                />
              </div>
            </div>
            <label class="block text-sm"
              >Output language
              <input v-model="targetLanguage" class="mt-1 w-full rounded border p-2" /></label
            ><button
              class="flex w-full items-center justify-center gap-2 rounded-lg bg-herb px-4 py-3 text-white disabled:opacity-50"
              :disabled="busy || !files.length"
              @click="translate"
            >
              <LoaderCircle v-if="busy" class="animate-spin" :size="18" />{{
                busy ? "Reading and translating…" : "Convert to Culinator"
              }}
            </button>
          </template>
          <template v-else>
            <label class="block text-sm"
              >Format
              <select v-model="structuredFormat" class="mt-1 w-full rounded border p-2">
                <option value="json_ld">JSON-LD (schema.org)</option>
                <option value="json">JSON</option>
                <option value="yaml">YAML</option>
              </select>
            </label>
            <textarea
              v-model="structuredContent"
              class="h-48 w-full rounded-xl border p-3 font-mono text-xs"
              placeholder="Paste recipe JSON-LD, JSON, or YAML…"
            />
            <button
              class="flex w-full items-center justify-center gap-2 rounded-lg bg-herb px-4 py-3 text-white disabled:opacity-50"
              :disabled="busy || !structuredContent.trim()"
              @click="importStructuredRecipe"
            >
              <LoaderCircle v-if="busy" class="animate-spin" :size="18" />Parse structured recipe
            </button>
          </template>
          <p v-if="error" class="rounded bg-red-50 p-3 text-sm text-red-700">{{ error }}</p>
        </div>
        <div class="space-y-4">
          <template v-if="tab === 'structured' && structuredDraft"
            ><div class="rounded-xl border p-4">
              <h3 class="font-semibold">{{ structuredDraft.title }}</h3>
              <ul
                v-if="structuredDraft.warnings.length"
                class="mt-2 list-disc pl-5 text-sm text-amber-700"
              >
                <li v-for="warning in structuredDraft.warnings" :key="warning">{{ warning }}</li>
              </ul>
            </div>
            <textarea
              v-model="structuredDraft.sourceText"
              class="h-96 w-full rounded-xl border p-3 font-mono text-xs"
            ></textarea
            ><button
              class="w-full rounded-lg bg-slate-900 px-4 py-3 text-white"
              @click="emit('accept', structuredDraft.sourceText, structuredDraft.title)"
            >
              Use this recipe
            </button></template
          >
          <template v-else-if="tab === 'photos' && result"
            ><div class="rounded-xl border p-4">
              <h3 class="font-semibold">{{ result.title }}</h3>
              <p
                :class="result.validation.valid ? 'text-green-700' : 'text-amber-700'"
                class="text-sm"
              >
                {{
                  result.validation.valid
                    ? "DSL validated successfully"
                    : "Review validation diagnostics before saving"
                }}
              </p>
              <ul v-if="result.warnings.length" class="mt-2 list-disc pl-5 text-sm text-amber-700">
                <li v-for="warning in result.warnings" :key="warning">{{ warning }}</li>
              </ul>
            </div>
            <textarea
              v-model="result.sourceText"
              class="h-96 w-full rounded-xl border p-3 font-mono text-xs"
            ></textarea
            ><button
              class="w-full rounded-lg bg-slate-900 px-4 py-3 text-white"
              @click="emit('accept', result.sourceText, result.title)"
            >
              Use this recipe
            </button>
            <details class="rounded border p-3">
              <summary class="cursor-pointer text-sm font-medium">OCR transcript</summary>
              <pre class="mt-2 whitespace-pre-wrap text-xs">{{
                result.extractedText || "AI read the images directly."
              }}</pre>
            </details></template
          >
          <div
            v-else
            class="flex h-full min-h-80 items-center justify-center rounded-xl bg-slate-50 p-8 text-center text-slate-500"
          >
            {{
              tab === "structured"
                ? "Parsed DSL draft will appear here for review."
                : "The translated DSL and validation report will appear here."
            }}
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
button.active {
  background: #28643b;
  color: white;
  border-radius: 8px;
  padding: 6px 12px;
  border: 0;
}
</style>
