<script setup lang="ts">
import { reactive, ref } from "vue";
import { Download } from "lucide-vue-next";
import type { BookExportFormat, BookExportOptions } from "../../../domain/types";
import { downloadExport, exportBook } from "../../../services/api/export-api";

const props = defineProps<{ bookId: string; bookTitle: string }>();

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
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="book-export">
    <h3>Export book</h3>
    <div class="formats">
      <label v-for="format in formats" :key="format.value" class="format">
        <input
          type="checkbox"
          :checked="options.formats.includes(format.value)"
          @change="toggle(format.value)"
        />
        {{ format.label }}
      </label>
    </div>
    <label
      >Unit system
      <select v-model="options.unitSystem">
        <option value="metric">Metric</option>
        <option value="us_customary">US customary</option>
      </select>
    </label>
    <button class="primary" :disabled="busy" @click="runExport">
      <Download :size="15" /> {{ busy ? "Exporting…" : "Download zip" }}
    </button>
    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<style scoped>
.book-export {
  display: grid;
  gap: 10px;
  padding: 12px;
  border: 1px solid #cbd3cd;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.75);
}
.book-export h3 {
  margin: 0;
  font-size: 14px;
}
.formats {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.format {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}
label {
  display: grid;
  gap: 4px;
  font-size: 13px;
}
select {
  height: 34px;
  border-radius: 8px;
  border: 1px solid #cbd3cd;
  padding: 0 8px;
}
button.primary {
  height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
.error {
  margin: 0;
  color: #b42318;
  font-size: 13px;
}
</style>
