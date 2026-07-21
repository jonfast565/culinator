<script setup lang="ts">
import { ref, watch } from "vue";
import { FileCode2, Save, X } from "lucide-vue-next";
import type { Diagnostic, ValidationResult } from "../../../domain/types";
import type { SaveStatus } from "../composables/useRecipeEditor";
import SourceEditor from "./SourceEditor.vue";

const props = defineProps<{
  source: string;
  validation: ValidationResult | null;
  dirty: boolean;
  saving: boolean;
  saveStatus?: SaveStatus;
  initialDiagnostic?: Diagnostic | null;
}>();

const emit = defineEmits<{
  (event: "update:source", value: string): void;
  (event: "save"): void;
  (event: "close"): void;
}>();

const sourceEditor = ref<InstanceType<typeof SourceEditor>>();

function saveStatusLabel(): string {
  switch (props.saveStatus) {
    case "saving":
      return "Saving…";
    case "saved":
      return "Saved";
    case "error":
      return "Save failed";
    default:
      return "";
  }
}

function jumpToDiagnostic(diagnostic: Diagnostic): void {
  if (diagnostic.start != null) {
    window.requestAnimationFrame(() => sourceEditor.value?.jumpToOffset(diagnostic.start!));
  }
}

watch(
  () => props.initialDiagnostic,
  (next) => {
    if (next) jumpToDiagnostic(next);
  },
  { immediate: true },
);
</script>

<template>
  <aside class="edit-drawer">
    <header class="drawer-head">
      <div class="drawer-title"><FileCode2 :size="15" /> Recipe source</div>
      <div class="drawer-actions">
        <span v-if="saveStatusLabel()" class="save-status" :class="saveStatus">{{
          saveStatusLabel()
        }}</span>
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

    <div class="drawer-body source">
      <SourceEditor
        ref="sourceEditor"
        :model-value="source"
        :diagnostics="validation?.diagnostics"
        @update:model-value="emit('update:source', $event)"
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
.drawer-title {
  display: flex;
  align-items: center;
  gap: 7px;
  color: #45524b;
  font-size: 13px;
  font-weight: 600;
}
.drawer-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}
.save-status {
  font-size: 12px;
  color: #6d7972;
}
.save-status.saved {
  color: #28643b;
}
.save-status.error {
  color: #a83737;
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
.source {
  display: flex;
  flex-direction: column;
}
.source .source-editor {
  flex: 1;
  min-height: 0;
}
</style>
