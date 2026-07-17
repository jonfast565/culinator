<script setup lang="ts">
/* global HTMLDialogElement */
import { reactive, ref } from "vue";
import { Download, X } from "lucide-vue-next";
import type { BookExportFormat, BookExportOptions } from "../../../domain/types";
import { downloadExport, exportBook } from "../../../services/api/export-api";

const props = defineProps<{ bookId: string; bookTitle: string }>();

const dialogEl = ref<HTMLDialogElement | null>(null);
const busy = ref(false);
const error = ref("");
const options = reactive<BookExportOptions>({
  formats: ["epub", "print_html"],
  title: props.bookTitle,
  author: "",
  description: "",
  unitSystem: "metric",
  includeNutrition: true,
  toc: true,
  sectionDividers: true,
});

const formats: { value: BookExportFormat; label: string }[] = [
  { value: "epub", label: "EPUB" },
  { value: "print_html", label: "Print HTML" },
  { value: "web", label: "Static site" },
];

function open(): void {
  options.title = props.bookTitle;
  error.value = "";
  dialogEl.value?.showModal();
}

function close(): void {
  dialogEl.value?.close();
}

function toggle(format: BookExportFormat): void {
  const index = options.formats.indexOf(format);
  if (index >= 0) options.formats.splice(index, 1);
  else options.formats.push(format);
}

async function runExport(): Promise<void> {
  if (!options.formats.length) {
    error.value = "Choose at least one format.";
    return;
  }
  busy.value = true;
  error.value = "";
  try {
    const bundle = await exportBook(props.bookId, options);
    await downloadExport(bundle);
    close();
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <button class="export-trigger" title="Export book" @click="open">
    <Download :size="15" />
    <span>Export</span>
  </button>

  <dialog ref="dialogEl" class="export-dialog" @click.self="close">
    <form class="export-form" @submit.prevent="runExport">
      <header class="export-head">
        <h2>Export book</h2>
        <button type="button" class="icon-btn" aria-label="Close" @click="close">
          <X :size="18" />
        </button>
      </header>

      <p class="export-lead">Download {{ bookTitle }} as a zip bundle.</p>

      <fieldset class="formats">
        <legend>Formats</legend>
        <label v-for="format in formats" :key="format.value" class="format">
          <input
            type="checkbox"
            :checked="options.formats.includes(format.value)"
            @change="toggle(format.value)"
          />
          {{ format.label }}
        </label>
      </fieldset>

      <label class="field">
        Unit system
        <select v-model="options.unitSystem">
          <option value="metric">Metric</option>
          <option value="us_customary">US customary</option>
        </select>
      </label>

      <footer class="export-actions">
        <button type="button" class="ghost" @click="close">Cancel</button>
        <button type="submit" class="primary" :disabled="busy">
          <Download :size="15" /> {{ busy ? "Exporting…" : "Download zip" }}
        </button>
      </footer>

      <p v-if="error" class="error">{{ error }}</p>
    </form>
  </dialog>
</template>

<style scoped>
.export-trigger {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 34px;
  padding: 0 12px;
  border-radius: 8px;
  border: 1px solid #cbd3cd;
  background: rgba(255, 255, 255, 0.75);
  color: #23302a;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.export-trigger:hover {
  background: #fff;
  color: #28643b;
}

.export-dialog {
  padding: 0;
  border: 0;
  border-radius: 14px;
  max-width: min(92vw, 440px);
  box-shadow: 0 24px 60px -20px rgba(20, 30, 25, 0.45);
  background: #fbf9f3;
}
.export-dialog::backdrop {
  background: rgba(20, 30, 25, 0.45);
}
.export-form {
  display: grid;
  gap: 16px;
  padding: 20px 22px 22px;
}
.export-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.export-head h2 {
  margin: 0;
  font-family: "Iowan Old Style", "Palatino Linotype", Palatino, Georgia, serif;
  font-size: 20px;
  font-weight: 600;
  color: #23302a;
}
.icon-btn {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: #55635b;
  cursor: pointer;
}
.icon-btn:hover {
  background: rgba(40, 100, 59, 0.08);
  color: #28643b;
}
.export-lead {
  margin: 0;
  font-size: 14px;
  color: #6d7972;
}
.formats {
  margin: 0;
  padding: 0;
  border: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}
.formats legend {
  width: 100%;
  margin-bottom: 4px;
  font-size: 13px;
  font-weight: 600;
  color: #23302a;
}
.format {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}
.field {
  display: grid;
  gap: 6px;
  font-size: 13px;
  font-weight: 600;
  color: #23302a;
}
.field select {
  height: 36px;
  border-radius: 8px;
  border: 1px solid #cbd3cd;
  padding: 0 10px;
  font-weight: 400;
  background: #fff;
}
.export-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
.ghost,
.primary {
  height: 36px;
  padding: 0 14px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.ghost {
  border: 1px solid #cbd3cd;
  background: transparent;
  color: #55635b;
}
.primary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 0;
  background: #28643b;
  color: #fff;
}
.primary:disabled {
  opacity: 0.6;
  cursor: wait;
}
.error {
  margin: 0;
  color: #b42318;
  font-size: 13px;
}
</style>
