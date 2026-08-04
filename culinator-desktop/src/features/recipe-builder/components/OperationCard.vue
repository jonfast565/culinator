<script setup lang="ts">
/* global Event, HTMLInputElement, HTMLSelectElement */
import { computed, ref, watch } from "vue";
import {
  ChevronDown,
  ChevronRight,
  ChevronUp,
  Copy,
  ImagePlus,
  Loader2,
  Trash2,
  X,
} from "lucide-vue-next";
import { fileToBase64, uploadRecipeImage } from "../../../services/api";
import RecipeImage from "../../reading/components/RecipeImage.vue";
import type {
  BuilderBinding,
  BuilderDoneness,
  BuilderEquipment,
  BuilderOperation,
} from "../composables/useRecipeBuilder";
import BuilderTextField from "./BuilderTextField.vue";
import BindingEditor from "./BindingEditor.vue";
import AfterEditor from "./AfterEditor.vue";
import DurationField from "./DurationField.vue";
import EquipmentEditor from "./EquipmentEditor.vue";
import DonenessEditor from "./DonenessEditor.vue";
import NoteList from "./NoteList.vue";

const props = defineProps<{
  operation: BuilderOperation;
  resourceSymbols: string[];
  operationSymbols: string[];
  previousStep?: string;
  recipeId?: string;
  disabled?: boolean;
  canMoveUp?: boolean;
  canMoveDown?: boolean;
  /** Expand when the live preview (or diagnostics) focuses this step. */
  forceExpanded?: boolean;
  /** Bumped by "Expand all" / "Collapse all" in the Method section. */
  expandGeneration?: number;
  collapseGeneration?: number;
  /** Initial open state (e.g. the only step in a new recipe). */
  startExpanded?: boolean;
  /** Visually mark the card as the current preview selection. */
  focused?: boolean;
  createIngredient?: (name: string) => string | undefined;
  createResource?: (name: string, role: string) => string | undefined;
}>();

const emit = defineEmits<{
  verb: [value: string];
  inputs: [bindings: BuilderBinding[]];
  produces: [value: string];
  after: [predecessors: string[]];
  field: [key: string, value: string];
  flag: [key: string, value: boolean];
  notes: [value: string[]];
  doneness: [cues: BuilderDoneness[]];
  equipment: [bindings: BuilderEquipment[]];
  photo: [value: string];
  rename: [value: string];
  duplicate: [];
  remove: [];
  move: [direction: "up" | "down"];
  focus: [];
}>();

const LABORS = ["", "active", "passive", "monitor", "automated"];
const HEATS = ["", "low", "medium_low", "medium", "medium_high", "high"];
const VERBS = [
  "mix",
  "combine",
  "whisk",
  "blend",
  "fold",
  "knead",
  "heat",
  "cook",
  "bake",
  "simmer",
  "boil",
  "fry",
  "saute",
  "roast",
  "rest",
  "cool",
  "chill",
  "cut",
  "chop",
  "dice",
  "mince",
  "slice",
  "mash",
  "grate",
  "drain",
  "strain",
  "grease",
  "coat",
  "spread",
  "season",
  "transfer",
  "prepare",
];

const uid = Math.random().toString(36).slice(2, 8);
const expanded = ref(!!props.forceExpanded || !!props.startExpanded);
watch(
  () => props.forceExpanded,
  (next) => {
    if (next) expanded.value = true;
  },
);
watch(
  () => props.expandGeneration,
  (next, prev) => {
    if (next && next !== prev) expanded.value = true;
  },
);
watch(
  () => props.collapseGeneration,
  (next, prev) => {
    if (next && next !== prev) expanded.value = false;
  },
);
const otherSteps = computed(() =>
  props.operationSymbols.filter((symbol) => symbol !== props.operation.symbol),
);

const summary = computed(() => {
  const verb = props.operation.action || (props.operation.isPrep ? "prep" : "step");
  const inputs = props.operation.inputs
    .map((binding) => binding.symbol.replace(/_/g, " "))
    .filter(Boolean)
    .slice(0, 3)
    .join(", ");
  const duration = props.operation.durationText;
  const parts = [verb];
  if (inputs) parts.push(inputs);
  if (duration) parts.push(duration);
  if (props.operation.labor) parts.push(props.operation.labor);
  return parts.join(" · ");
});

function onLabor(event: Event): void {
  emit("field", "labor", (event.target as HTMLSelectElement).value);
}
function onHeat(event: Event): void {
  emit("field", "heat", (event.target as HTMLSelectElement).value);
}

const uploading = ref(false);
const photoUrl = ref("");
function commitPhotoUrl(): void {
  const value = photoUrl.value.trim();
  if (value) {
    emit("photo", value);
    photoUrl.value = "";
  }
}
async function onPhotoFile(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file || !props.recipeId) return;
  uploading.value = true;
  try {
    const dataBase64 = await fileToBase64(file);
    const asset = await uploadRecipeImage(props.recipeId, {
      role: "step",
      mediaType: file.type || "image/jpeg",
      fileName: file.name,
      dataBase64,
    });
    emit("photo", asset.handle);
  } finally {
    uploading.value = false;
  }
}
</script>

<template>
  <article
    class="card operation-card"
    :class="{ disabled, collapsed: !expanded, focused }"
    :data-builder-symbol="operation.symbol"
  >
    <header class="card-head">
      <button
        type="button"
        class="expand-toggle"
        :aria-expanded="expanded"
        :title="expanded ? 'Collapse step' : 'Expand step'"
        @click="
          expanded = !expanded;
          emit('focus');
        "
      >
        <ChevronDown v-if="expanded" :size="16" />
        <ChevronRight v-else :size="16" />
      </button>
      <div class="titles">
        <strong class="card-title">{{ operation.symbol.replace(/_/g, " ") }}</strong>
        <span v-if="!expanded" class="summary">{{ summary }}</span>
        <span v-if="operation.isPrep" class="prep-badge">prep</span>
        <span v-if="!expanded && operation.labor" class="meta-chip labor">{{
          operation.labor
        }}</span>
        <span v-if="!expanded && operation.durationText" class="meta-chip time">{{
          operation.durationText
        }}</span>
      </div>
      <div class="card-tools">
        <button
          class="icon"
          title="Move up"
          :disabled="disabled || !canMoveUp"
          @click="emit('move', 'up')"
        >
          <ChevronUp :size="15" />
        </button>
        <button
          class="icon"
          title="Move down"
          :disabled="disabled || !canMoveDown"
          @click="emit('move', 'down')"
        >
          <ChevronDown :size="15" />
        </button>
        <button class="icon" title="Duplicate" :disabled="disabled" @click="emit('duplicate')">
          <Copy :size="14" />
        </button>
        <button
          class="icon danger"
          title="Remove step"
          :disabled="disabled"
          @click="emit('remove')"
        >
          <Trash2 :size="15" />
        </button>
      </div>
    </header>

    <template v-if="expanded">
      <p v-if="operation.readOnly" class="readonly-note">
        This is a shorthand <code>prep</code> step. Edit it in the source view to change the verb or
        ingredient; open a block in source to add duration and dependencies here.
      </p>

      <template v-else>
        <datalist :id="`verbs-${uid}`">
          <option v-for="verb in VERBS" :key="verb" :value="verb" />
        </datalist>
        <datalist :id="`res-${uid}`">
          <option v-for="symbol in resourceSymbols" :key="symbol" :value="symbol" />
        </datalist>

        <div v-if="!operation.isPrep" class="two">
          <BuilderTextField
            label="Action (verb)"
            :model-value="operation.action"
            :list="`verbs-${uid}`"
            placeholder="e.g. mix, bake, rest"
            :disabled="disabled"
            :focus-priority="!operation.action || operation.action === 'prepare'"
            @commit="emit('verb', $event)"
          />
          <BuilderTextField
            label="Produces"
            :model-value="operation.produces"
            :list="`res-${uid}`"
            placeholder="resulting material"
            :disabled="disabled"
            @commit="emit('produces', $event)"
          />
        </div>
        <p v-else class="prep-note">
          Prep of
          <strong>{{ operation.inputs[0]?.symbol?.replace(/_/g, " ") || "ingredient" }}</strong>
          →
          <strong>{{ operation.produces.replace(/_/g, " ") || "result" }}</strong>
        </p>

        <BindingEditor
          v-if="!operation.isPrep"
          :bindings="operation.inputs"
          :options="resourceSymbols"
          :list-id="`inputs-${uid}`"
          :disabled="disabled"
          :create-ingredient="createIngredient"
          @commit="emit('inputs', $event)"
        />

        <AfterEditor
          :predecessors="operation.after"
          :options="otherSteps"
          :previous-step="previousStep"
          :disabled="disabled"
          @commit="emit('after', $event)"
        />

        <DurationField
          :model-value="operation.durationText"
          :disabled="disabled"
          @commit="emit('field', 'duration', $event)"
        />

        <div class="two">
          <label class="select-field">
            <span>Labor</span>
            <select :value="operation.labor" :disabled="disabled" @change="onLabor">
              <option v-for="labor in LABORS" :key="labor" :value="labor">
                {{ labor || "—" }}
              </option>
            </select>
          </label>
          <label class="check-field">
            <input
              type="checkbox"
              :checked="operation.optional"
              :disabled="disabled"
              @change="emit('flag', 'optional', ($event.target as HTMLInputElement).checked)"
            />
            Optional step
          </label>
        </div>

        <details class="more">
          <summary>More options</summary>
          <div class="more-fields">
            <div class="two">
              <BuilderTextField
                label="Temperature"
                :model-value="operation.temperature"
                placeholder="e.g. 350 fahrenheit"
                :disabled="disabled"
                @commit="emit('field', 'temperature', $event)"
              />
              <label class="select-field">
                <span>Heat level</span>
                <select :value="operation.heat" :disabled="disabled" @change="onHeat">
                  <option v-for="heat in HEATS" :key="heat" :value="heat">
                    {{ heat ? heat.replace("_", " ") : "—" }}
                  </option>
                </select>
              </label>
            </div>

            <BuilderTextField
              label="Repeat (times)"
              :model-value="operation.repeat"
              type="number"
              placeholder="e.g. 3"
              :disabled="disabled"
              @commit="emit('field', 'repeat', $event)"
            />

            <EquipmentEditor
              :bindings="operation.equipment"
              :options="resourceSymbols"
              :list-id="`equip-${uid}`"
              :disabled="disabled"
              :create-resource="createResource"
              @commit="emit('equipment', $event)"
            />

            <DonenessEditor
              :cues="operation.doneness"
              :disabled="disabled"
              @commit="emit('doneness', $event)"
            />

            <NoteList
              label="Notes"
              :notes="operation.notes"
              :disabled="disabled"
              @commit="emit('notes', $event)"
            />

            <BuilderTextField
              v-if="!operation.isPrep"
              label="Identifier (step name)"
              :model-value="operation.symbol"
              :disabled="disabled"
              @commit="emit('rename', $event)"
            />

            <div class="photo">
              <span class="editor-label">Step photo</span>
              <div v-if="operation.photo" class="photo-preview">
                <RecipeImage :image-ref="operation.photo" :recipe-id="recipeId" alt="Step" />
                <button class="photo-remove" title="Remove photo" @click="emit('photo', '')">
                  <X :size="14" />
                </button>
              </div>
              <div class="photo-controls">
                <input
                  v-model="photoUrl"
                  type="url"
                  placeholder="Paste image URL…"
                  :disabled="disabled"
                  @change="commitPhotoUrl"
                  @keyup.enter="commitPhotoUrl"
                />
                <label class="upload-btn" :class="{ busy: uploading }">
                  <Loader2 v-if="uploading" :size="14" class="spin" />
                  <ImagePlus v-else :size="14" />
                  {{ uploading ? "Uploading…" : "Upload" }}
                  <input
                    type="file"
                    accept="image/*"
                    hidden
                    :disabled="uploading || disabled"
                    @change="onPhotoFile"
                  />
                </label>
              </div>
            </div>
          </div>
        </details>
      </template>

      <div v-if="operation.unknown.length" class="chips">
        <span class="chips-label">Kept as written:</span>
        <span v-for="(item, index) in operation.unknown" :key="index" class="chip">
          {{ item.keyword }} {{ item.text }}
        </span>
      </div>
    </template>
  </article>
</template>

<style scoped>
.operation-card {
  display: grid;
  gap: 12px;
  padding: 14px;
  scroll-margin-top: 12px;
  transition:
    box-shadow 0.18s ease,
    border-color 0.18s ease,
    background 0.18s ease;
}
.operation-card.collapsed {
  gap: 0;
  padding: 10px 12px;
}
.operation-card.focused {
  border-color: #8fb897;
  box-shadow: 0 0 0 3px rgb(40 100 59 / 14%);
  background: #fbfefb;
}
.operation-card.disabled {
  opacity: 0.6;
}
.card-head {
  display: flex;
  align-items: center;
  gap: 8px;
}
.expand-toggle {
  width: 28px;
  height: 28px;
  padding: 0;
  display: grid;
  place-items: center;
  border: 0;
  background: transparent;
  color: #55635b;
  cursor: pointer;
}
.titles {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px 10px;
}
.card-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 15px;
  text-transform: capitalize;
}
.summary {
  font-size: 12px;
  color: #6d7972;
  text-transform: capitalize;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.prep-badge,
.meta-chip {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  padding: 2px 6px;
  border-radius: 999px;
}
.prep-badge {
  color: #5d4a8a;
  background: #efeaf8;
}
.meta-chip.labor {
  color: #28643b;
  background: #e4efe6;
}
.meta-chip.time {
  color: #1a5f8a;
  background: #e7f1f7;
  text-transform: none;
  letter-spacing: 0;
  font-variant-numeric: tabular-nums;
}
.card-tools {
  display: flex;
  gap: 4px;
}
.icon {
  width: 30px;
  height: 30px;
  padding: 0;
  display: grid;
  place-items: center;
}
.icon.danger {
  color: #a83737;
}
.readonly-note,
.prep-note {
  margin: 0;
  font-size: 12px;
  color: #8a938c;
}
.prep-note strong {
  color: #45524b;
  text-transform: capitalize;
}
.two {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  align-items: end;
}
.select-field {
  display: grid;
  gap: 5px;
  margin: 0;
  font-size: 12px;
  color: #657169;
}
.check-field {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin: 0;
  align-self: center;
  font-size: 13px;
  color: #3d4842;
}
.check-field input {
  width: auto;
}
.more summary {
  cursor: pointer;
  font-size: 12px;
  color: #55635b;
  user-select: none;
}
.more-fields {
  display: grid;
  gap: 14px;
  margin-top: 10px;
}
.editor-label {
  font-size: 12px;
  color: #657169;
}
.photo {
  display: grid;
  gap: 6px;
}
.photo-preview {
  position: relative;
  aspect-ratio: 16 / 9;
  border-radius: 7px;
  overflow: hidden;
  border: 1px solid #d3d8d1;
}
.photo-remove {
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
.photo-controls {
  display: flex;
  gap: 8px;
}
.photo-controls input[type="url"] {
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
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
  padding-top: 4px;
  border-top: 1px dashed #dfe3de;
}
.chips-label {
  font-size: 11px;
  color: #8a938c;
}
.chip {
  font-size: 11px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  padding: 2px 8px;
  border-radius: 10px;
  background: #eef1ec;
  color: #55635b;
}
@media (max-width: 620px) {
  .two {
    grid-template-columns: 1fr;
  }
}
</style>
