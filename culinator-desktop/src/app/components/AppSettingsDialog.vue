<script setup lang="ts">
/* global Event, HTMLSelectElement, HTMLInputElement, KeyboardEvent */
import { inject, onBeforeUnmount, onMounted } from "vue";
import { X } from "lucide-vue-next";
import { UNIT_DISPLAY_KEY } from "../../features/units/composables/useUnitDisplay";
import {
  DECIMAL_PLACES_OPTIONS,
  VIEW_SETTINGS_KEY,
  type DecimalPlaces,
  type IndexCardFormat,
  type MisePlacement,
  type NumberStyle,
  type TextSize,
} from "../../features/reading/composables/useViewSettings";
import { INDEX_CARD_PICKER_OPTIONS } from "../../features/reading/indexCardFormat";
import {
  INDEX_CARD_MARGIN_NAMES,
  type IndexCardMargin,
} from "../../features/reading/indexCardMargin";
import {
  RECIPE_TYPE_SCALE_LABELS,
  type RecipeTypeScale,
} from "../../features/reading/recipeTypeScale";

defineProps<{ open: boolean }>();
const emit = defineEmits<{ close: [] }>();

const units = inject(UNIT_DISPLAY_KEY, null);
const viewSettings = inject(VIEW_SETTINGS_KEY, null);

function onKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape") emit("close");
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));

function onPagerToggle(event: Event): void {
  viewSettings!.showIndexCardPager.value = (event.target as HTMLInputElement).checked;
}

function selectValue<T extends string>(
  event: Event,
  apply: ((value: T) => void) | undefined,
): void {
  apply?.((event.target as HTMLSelectElement).value as T);
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open && viewSettings && units" class="dialog-backdrop" @click.self="emit('close')">
      <section
        class="settings-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
      >
        <header class="dialog-header">
          <h2 id="settings-title">Settings</h2>
          <button type="button" class="icon-btn" title="Close" @click="emit('close')">
            <X :size="18" />
          </button>
        </header>

        <div class="settings-body">
          <fieldset>
            <legend>Units</legend>
            <label class="field">
              <span>Mass & volume</span>
              <select
                :value="units.unitSystem.value"
                @change="selectValue($event, units.setUnitSystem)"
              >
                <option value="metric">Metric</option>
                <option value="us_customary">US customary</option>
              </select>
            </label>
            <label class="field">
              <span>Temperature</span>
              <select
                :value="units.temperatureScale.value"
                @change="selectValue($event, units.setTemperatureScale)"
              >
                <option value="celsius">Celsius (°C)</option>
                <option value="fahrenheit">Fahrenheit (°F)</option>
              </select>
            </label>
          </fieldset>

          <fieldset>
            <legend>Numbers</legend>
            <label class="field">
              <span>Amount style</span>
              <select
                :value="viewSettings.numberStyle.value"
                @change="selectValue<NumberStyle>($event, viewSettings.setNumberStyle)"
              >
                <option value="fractions">Cooking fractions (½, 1¼)</option>
                <option value="decimals">Decimals (0.5, 1.25)</option>
              </select>
            </label>
            <label class="field">
              <span>Decimal precision</span>
              <select
                :value="viewSettings.decimalPlaces.value"
                @change="
                  viewSettings.setDecimalPlaces(
                    Number(($event.target as HTMLSelectElement).value) as DecimalPlaces,
                  )
                "
              >
                <option v-for="places in DECIMAL_PLACES_OPTIONS" :key="places" :value="places">
                  {{ places === 0 ? "Whole numbers" : `${places} decimal place${places > 1 ? "s" : ""}` }}
                </option>
              </select>
            </label>
            <p class="field-hint">
              Applies to decimal amounts and to fraction fallbacks (converted metric weights, etc.).
            </p>
          </fieldset>

          <fieldset>
            <legend>Reading layout</legend>
            <label class="field">
              <span>Ingredients & equipment</span>
              <select
                :value="viewSettings.misePlacement.value"
                @change="selectValue<MisePlacement>($event, viewSettings.setMisePlacement)"
              >
                <option value="top-matter">One list at the top</option>
                <option value="colocated">Beside each section</option>
              </select>
            </label>
            <label class="field">
              <span>Text size</span>
              <select
                :value="viewSettings.textSize.value"
                @change="selectValue<TextSize>($event, viewSettings.setTextSize)"
              >
                <option value="default">Default</option>
                <option value="large">Large</option>
                <option value="x-large">Extra large</option>
              </select>
            </label>
            <label class="field">
              <span>Open book layout</span>
              <select
                :value="viewSettings.bookLayout.value"
                @change="selectValue($event, viewSettings.setBookLayout)"
              >
                <option value="book">Page-flip folio</option>
                <option value="cards">Recipe card grid</option>
              </select>
            </label>
          </fieldset>

          <fieldset>
            <legend>Index cards</legend>
            <label class="field">
              <span>Card size</span>
              <select
                :value="viewSettings.indexCardFormat.value"
                @change="selectValue<IndexCardFormat>($event, viewSettings.setIndexCardFormat)"
              >
                <option
                  v-for="option in INDEX_CARD_PICKER_OPTIONS"
                  :key="option.id"
                  :value="option.id"
                >
                  {{ option.label }}
                </option>
              </select>
            </label>
            <label class="field">
              <span>Card margins</span>
              <select
                :value="viewSettings.indexCardMargin.value"
                @change="selectValue<IndexCardMargin>($event, viewSettings.setIndexCardMargin)"
              >
                <option
                  v-for="(name, id) in INDEX_CARD_MARGIN_NAMES"
                  :key="id"
                  :value="id"
                >
                  {{ name }}
                </option>
              </select>
            </label>
            <label class="field checkbox">
              <input
                type="checkbox"
                :checked="viewSettings.showIndexCardPager.value"
                @change="onPagerToggle"
              />
              <span>Show “Card N of M” on index cards</span>
            </label>
            <div class="field-group">
              <span class="group-label">Type sizes</span>
              <label class="field compact">
                <span>Title</span>
                <select
                  :value="viewSettings.recipeHeaderScale.value"
                  @change="selectValue<RecipeTypeScale>($event, viewSettings.setRecipeHeaderScale)"
                >
                  <option v-for="(label, id) in RECIPE_TYPE_SCALE_LABELS" :key="id" :value="id">
                    {{ label }}
                  </option>
                </select>
              </label>
              <label class="field compact">
                <span>Body</span>
                <select
                  :value="viewSettings.recipeBodyScale.value"
                  @change="selectValue<RecipeTypeScale>($event, viewSettings.setRecipeBodyScale)"
                >
                  <option v-for="(label, id) in RECIPE_TYPE_SCALE_LABELS" :key="id" :value="id">
                    {{ label }}
                  </option>
                </select>
              </label>
              <label class="field compact">
                <span>Notes</span>
                <select
                  :value="viewSettings.recipeAnnotationScale.value"
                  @change="
                    selectValue<RecipeTypeScale>($event, viewSettings.setRecipeAnnotationScale)
                  "
                >
                  <option v-for="(label, id) in RECIPE_TYPE_SCALE_LABELS" :key="id" :value="id">
                    {{ label }}
                  </option>
                </select>
              </label>
            </div>
          </fieldset>
        </div>

        <footer class="dialog-footer">
          <button type="button" class="primary" @click="emit('close')">Done</button>
        </footer>
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: grid;
  place-items: center;
  padding: 20px;
  background: rgba(20, 28, 24, 0.45);
}
.settings-dialog {
  width: min(100%, 480px);
  max-height: min(90vh, 640px);
  display: flex;
  flex-direction: column;
  border-radius: 12px;
  background: #fbfcfa;
  box-shadow: 0 20px 50px -20px rgba(0, 0, 0, 0.35);
  color: #23302a;
}
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 16px 18px 12px;
  border-bottom: 1px solid #e0e5e1;
}
.dialog-header h2 {
  margin: 0;
  font-size: 17px;
}
.icon-btn {
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: #55635b;
  cursor: pointer;
}
.icon-btn:hover {
  background: #e7efe6;
}
.settings-body {
  flex: 1;
  overflow: auto;
  padding: 8px 18px 16px;
}
fieldset {
  margin: 0 0 16px;
  padding: 0;
  border: 0;
}
legend {
  padding: 8px 0 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: #718078;
}
.field {
  display: grid;
  grid-template-columns: 1fr 1.1fr;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
  font-size: 13px;
}
.field.compact {
  grid-template-columns: 4.5rem 1fr;
}
.field.checkbox {
  grid-template-columns: auto 1fr;
  align-items: start;
  margin-top: 4px;
}
.field.checkbox input {
  margin-top: 2px;
}
.field select {
  height: 34px;
  padding: 0 10px;
  border: 1px solid #cbd3cd;
  border-radius: 7px;
  background: #fff;
  color: #23302a;
  font-size: 13px;
}
.field-hint {
  margin: -2px 0 0;
  grid-column: 1 / -1;
  font-size: 11px;
  line-height: 1.4;
  color: #718078;
}
.field-group {
  margin-top: 4px;
}
.group-label {
  display: block;
  margin-bottom: 6px;
  font-size: 12px;
  color: #55635b;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 18px 16px;
  border-top: 1px solid #e0e5e1;
}
.dialog-footer .primary {
  height: 34px;
  padding: 0 16px;
  border: 0;
  border-radius: 7px;
  background: #28643b;
  color: #fff;
  font-size: 13px;
  cursor: pointer;
}
.dialog-footer .primary:hover {
  background: #1f5230;
}
</style>
