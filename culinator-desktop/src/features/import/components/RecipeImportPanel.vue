<script setup lang="ts">
/* global File, Event, HTMLInputElement, btoa */
import { computed, onMounted, ref } from "vue";
import {
  Camera,
  ChevronLeft,
  ChevronRight,
  FileImage,
  KeyRound,
  LoaderCircle,
  X,
} from "lucide-vue-next";
import type {
  PublicImportSettings,
  RecipeImageInput,
  RecipeImportResult,
  ValidationResult,
} from "../../../domain/types";
import {
  getImportSettings,
  importStructured,
  clearStoredApiKey,
  translateRecipeImages,
  updateImportSettings,
  validateRecipe,
} from "../../../services/api";

export interface ImportAcceptPayload {
  source: string;
  title: string;
  hasDiagnostics: boolean;
}

const emit = defineEmits<{ close: []; accept: [payload: ImportAcceptPayload] }>();

const step = ref(0);
const sourceKind = ref<"photos" | "structured">("photos");
const settings = ref<PublicImportSettings>({
  apiKeyConfigured: false,
  openaiModel: "gpt-4.1-mini",
  useLocalOcr: true,
  tesseractCommand: "tesseract",
});
const advancedOpen = ref(false);
const apiKey = ref("");
const targetLanguage = ref("English");
const files = ref<File[]>([]);
const previews = ref<string[]>([]);
const result = ref<RecipeImportResult>();
const structuredDraft = ref<{ title: string; sourceText: string; warnings: string[] }>();
const structuredFormat = ref<"json_ld" | "json" | "yaml">("json_ld");
const structuredContent = ref("");
const previewText = ref("");
const previewValidation = ref<ValidationResult | null>(null);
const busy = ref(false);
const error = ref("");

const previewTitle = computed(
  () => structuredDraft.value?.title ?? result.value?.title ?? "Imported recipe",
);
const previewWarnings = computed(
  () => structuredDraft.value?.warnings ?? result.value?.warnings ?? [],
);

const steps = ["Choose source", "Import", "Preview", "Confirm"];

onMounted(async () => {
  try {
    settings.value = await getImportSettings();
  } catch (e) {
    error.value = String(e);
  }
});

function chooseSource(kind: "photos" | "structured"): void {
  sourceKind.value = kind;
  step.value = 1;
}

function selected(event: Event): void {
  const input = event.target as HTMLInputElement;
  files.value = [...(input.files ?? [])];
  previews.value.forEach(URL.revokeObjectURL);
  previews.value = files.value.map(URL.createObjectURL);
  result.value = undefined;
}

async function saveSettings(): Promise<void> {
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

async function data(file: File): Promise<RecipeImageInput> {
  const buffer = await file.arrayBuffer();
  let binary = "";
  for (const byte of new Uint8Array(buffer)) binary += String.fromCharCode(byte);
  return { fileName: file.name, mediaType: file.type || "image/jpeg", dataBase64: btoa(binary) };
}

async function runImport(): Promise<void> {
  busy.value = true;
  error.value = "";
  try {
    if (sourceKind.value === "structured") {
      structuredDraft.value = await importStructured({
        format: structuredFormat.value,
        content: structuredContent.value,
      });
    } else {
      await saveSettings();
      result.value = await translateRecipeImages(
        await Promise.all(files.value.map(data)),
        targetLanguage.value,
      );
    }
    previewText.value =
      sourceKind.value === "structured"
        ? (structuredDraft.value?.sourceText ?? "")
        : (result.value?.sourceText ?? "");
    previewValidation.value = await validateRecipe(previewText.value);
    step.value = 2;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

function goToConfirm(): void {
  step.value = 3;
}

function confirmImport(): void {
  const source = previewText.value;
  if (!source.trim()) return;
  const diagnostics = previewValidation.value?.diagnostics ?? [];
  const hasDiagnostics =
    !previewValidation.value?.valid ||
    diagnostics.some((item) => item.severity === "error" || item.severity === "warning") ||
    previewWarnings.value.length > 0;
  emit("accept", {
    source,
    title: previewTitle.value,
    hasDiagnostics,
  });
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
    <section class="import-modal">
      <header class="import-head">
        <div>
          <h2>Import recipe</h2>
          <p class="import-sub">Step {{ step + 1 }} of {{ steps.length }} — {{ steps[step] }}</p>
        </div>
        <button type="button" aria-label="Close" @click="emit('close')"><X /></button>
      </header>

      <div class="step-track">
        <span
          v-for="(label, index) in steps"
          :key="label"
          class="step-dot"
          :class="{ active: index === step, done: index < step }"
        >
          {{ label }}
        </span>
      </div>

      <div v-if="step === 0" class="step-body">
        <p class="step-intro">How would you like to bring a recipe in?</p>
        <div class="source-cards">
          <button type="button" class="source-card" @click="chooseSource('photos')">
            <Camera :size="28" />
            <strong>Photos / OCR</strong>
            <span>Scan a cookbook page or photo with AI translation</span>
          </button>
          <button type="button" class="source-card" @click="chooseSource('structured')">
            <FileImage :size="28" />
            <strong>Paste structured data</strong>
            <span>JSON-LD, JSON, or YAML from the web</span>
          </button>
        </div>
      </div>

      <div v-else-if="step === 1" class="step-body">
        <button type="button" class="back-link" @click="step = 0">
          <ChevronLeft :size="15" /> Change source
        </button>

        <details class="advanced" :open="advancedOpen">
          <summary @click.prevent="advancedOpen = !advancedOpen">
            <KeyRound :size="16" /> Advanced settings (API key &amp; OCR)
          </summary>
          <div class="advanced-body">
            <p v-if="settings.apiKeyConfigured" class="hint ok">
              API key stored.
              <button type="button" class="link" @click="clearKey">Clear stored key</button>
            </p>
            <label class="field"
              >OpenAI API key
              <input
                v-model="apiKey"
                type="password"
                :placeholder="settings.apiKeyConfigured ? 'Configured — enter to replace' : 'sk-…'"
              />
            </label>
            <label class="field">Model <input v-model="settings.openaiModel" /> </label>
            <label class="checkbox"
              ><input v-model="settings.useLocalOcr" type="checkbox" /> Try local Tesseract OCR
              before AI</label
            >
            <label v-if="settings.useLocalOcr" class="field"
              >Tesseract command <input v-model="settings.tesseractCommand"
            /></label>
          </div>
        </details>

        <template v-if="sourceKind === 'photos'">
          <div class="drop-zone">
            <Camera class="mx-auto mb-2" />
            <p>Choose one or more recipe images.</p>
            <label class="upload-btn"
              ><FileImage :size="17" /> Select images<input
                type="file"
                accept="image/*"
                capture="environment"
                multiple
                hidden
                @change="selected"
            /></label>
            <div v-if="previews.length" class="preview-grid">
              <img v-for="src in previews" :key="src" :src="src" alt="" />
            </div>
          </div>
          <label class="field">Output language <input v-model="targetLanguage" /> </label>
        </template>

        <template v-else>
          <label class="field"
            >Format
            <select v-model="structuredFormat">
              <option value="json_ld">JSON-LD (schema.org)</option>
              <option value="json">JSON</option>
              <option value="yaml">YAML</option>
            </select>
          </label>
          <textarea
            v-model="structuredContent"
            class="paste-area"
            placeholder="Paste recipe JSON-LD, JSON, or YAML…"
          />
        </template>

        <p v-if="error" class="error">{{ error }}</p>
        <button
          type="button"
          class="primary wide"
          :disabled="busy || (sourceKind === 'photos' ? !files.length : !structuredContent.trim())"
          @click="runImport"
        >
          <LoaderCircle v-if="busy" class="spin" :size="18" />
          {{ busy ? "Importing…" : "Continue to preview" }}
          <ChevronRight v-if="!busy" :size="16" />
        </button>
      </div>

      <div v-else-if="step === 2" class="step-body">
        <div class="preview-card">
          <h3>{{ previewTitle }}</h3>
          <p :class="previewValidation?.valid ? 'ok' : 'warn'">
            {{
              previewValidation?.valid
                ? "DSL validated successfully"
                : "Review validation diagnostics before saving"
            }}
          </p>
          <ul v-if="previewWarnings.length" class="warnings">
            <li v-for="warning in previewWarnings" :key="warning">{{ warning }}</li>
          </ul>
          <ul v-if="previewValidation?.diagnostics.length" class="warnings">
            <li
              v-for="(item, index) in previewValidation.diagnostics"
              :key="`${item.message}-${index}`"
            >
              {{ item.severity }}: {{ item.message }}
            </li>
          </ul>
        </div>
        <textarea v-model="previewText" class="source-preview" />
        <div class="step-actions">
          <button type="button" class="ghost" @click="step = 1">
            <ChevronLeft :size="15" /> Back
          </button>
          <button type="button" class="primary" @click="goToConfirm">
            Continue <ChevronRight :size="15" />
          </button>
        </div>
      </div>

      <div v-else class="step-body">
        <div class="confirm-card">
          <h3>Ready to add “{{ previewTitle }}”?</h3>
          <p>
            The recipe will be saved to your library
            <template v-if="previewValidation && !previewValidation.valid">
              and opened in the editor to review diagnostics.
            </template>
            <template v-else> and opened for reading.</template>
          </p>
        </div>
        <div class="step-actions">
          <button type="button" class="ghost" @click="step = 2">
            <ChevronLeft :size="15" /> Back
          </button>
          <button type="button" class="primary" @click="confirmImport">Add recipe</button>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.import-modal {
  max-height: 92vh;
  width: min(100%, 720px);
  overflow: auto;
  border-radius: 16px;
  background: #fff;
  padding: 24px;
  box-shadow: 0 24px 60px -24px rgba(0, 0, 0, 0.45);
}
.import-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 16px;
}
.import-head h2 {
  margin: 0;
  font-size: 20px;
}
.import-sub {
  margin: 4px 0 0;
  font-size: 13px;
  color: #6d7972;
}
.step-track {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 20px;
}
.step-dot {
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 11px;
  background: #eef1ed;
  color: #6d7972;
}
.step-dot.active {
  background: #28643b;
  color: #fff;
}
.step-dot.done {
  background: #d9f0df;
  color: #28643b;
}
.step-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.step-intro {
  margin: 0;
  color: #4a5a52;
}
.source-cards {
  display: grid;
  gap: 12px;
}
@media (min-width: 540px) {
  .source-cards {
    grid-template-columns: 1fr 1fr;
  }
}
.source-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
  padding: 18px;
  border: 1px solid #cbd3cd;
  border-radius: 12px;
  background: #fbf9f3;
  text-align: left;
  cursor: pointer;
}
.source-card:hover {
  border-color: #28643b;
  background: #f3faf4;
}
.source-card strong {
  font-size: 15px;
}
.source-card span {
  font-size: 13px;
  color: #6d7972;
  line-height: 1.4;
}
.back-link {
  align-self: flex-start;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: 0;
  background: transparent;
  color: #28643b;
  font-size: 13px;
  cursor: pointer;
}
.advanced {
  border: 1px solid #e2e6e1;
  border-radius: 10px;
  padding: 10px 12px;
}
.advanced summary {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  color: #4a5a52;
}
.advanced-body {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}
.field input,
.field select,
.paste-area,
.source-preview {
  padding: 8px 10px;
  border: 1px solid #cbd3cd;
  border-radius: 8px;
  font-size: 13px;
}
.paste-area,
.source-preview {
  min-height: 180px;
  font-family: ui-monospace, monospace;
  font-size: 12px;
}
.checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}
.drop-zone {
  border: 1px dashed #cbd3cd;
  border-radius: 12px;
  padding: 20px;
  text-align: center;
}
.upload-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  padding: 8px 14px;
  border-radius: 8px;
  background: #23302a;
  color: #fff;
  cursor: pointer;
}
.preview-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
  margin-top: 12px;
}
.preview-grid img {
  height: 88px;
  width: 100%;
  object-fit: cover;
  border-radius: 8px;
}
.preview-card,
.confirm-card {
  padding: 14px;
  border: 1px solid #e2e6e1;
  border-radius: 10px;
  background: #fbf9f3;
}
.preview-card h3,
.confirm-card h3 {
  margin: 0 0 6px;
}
.ok {
  color: #28643b;
  font-size: 13px;
}
.warn {
  color: #8a6d1f;
  font-size: 13px;
}
.warnings {
  margin: 8px 0 0;
  padding-left: 18px;
  font-size: 12px;
  color: #8a6d1f;
}
.hint {
  font-size: 12px;
}
.link {
  border: 0;
  background: transparent;
  text-decoration: underline;
  cursor: pointer;
}
.error {
  margin: 0;
  padding: 10px;
  border-radius: 8px;
  background: #fbeceb;
  color: #a83737;
  font-size: 13px;
}
.primary.wide {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 42px;
}
.step-actions {
  display: flex;
  justify-content: space-between;
  gap: 10px;
}
.spin {
  animation: spin 1s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
