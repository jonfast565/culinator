<script setup lang="ts">
import { Beaker, Plus, Trash2, X } from "lucide-vue-next";
import type { BuilderFormula } from "../composables/useRecipeBuilder";
import BuilderTextField from "./BuilderTextField.vue";

/**
 * Compact baker's-percentage editor. The full calculator (preferment, pan/piece
 * scaling, apply-to-recipe) lives in the Formulas tool — this section keeps
 * the DSL block editable inline and offers a door into that tool.
 */
defineProps<{ formulas: BuilderFormula[]; disabled?: boolean }>();

const emit = defineEmits<{
  target: [symbol: string, value: string];
  ingredientBaker: [formula: string, ingredient: string, value: string];
  add: [];
  remove: [symbol: string];
  addIngredient: [formula: string];
  removeIngredient: [formula: string, ingredient: string];
  openTool: [];
}>();
</script>

<template>
  <section id="builder-formulas" class="panel builder-section">
    <div class="panel-header">
      <h3>Formulas</h3>
      <button class="ghost open-tool" type="button" :disabled="disabled" @click="emit('openTool')">
        <Beaker :size="14" /> Open formula tool
      </button>
    </div>

    <div v-if="!formulas.length" class="empty-cta">
      <p class="empty">
        Baker's percentages scale a dough or batter from flour. Add a formula here, or open the
        formula tool to seed one from the recipe and apply scaled amounts back.
      </p>
      <div class="empty-actions">
        <button class="primary" :disabled="disabled" @click="emit('add')">
          <Plus :size="14" /> Add formula
        </button>
        <button :disabled="disabled" @click="emit('openTool')">
          <Beaker :size="14" /> Open formula tool
        </button>
      </div>
    </div>

    <div class="formulas">
      <article v-for="formula in formulas" :key="formula.symbol" class="card formula-card">
        <header class="formula-head">
          <strong>{{ formula.symbol.replace(/_/g, " ") }}</strong>
          <span v-if="formula.basis" class="basis">{{ formula.basis }}</span>
          <button
            class="icon danger"
            title="Remove formula"
            :disabled="disabled"
            @click="emit('remove', formula.symbol)"
          >
            <Trash2 :size="15" />
          </button>
        </header>

        <BuilderTextField
          label="Target weight"
          :model-value="formula.target"
          placeholder="e.g. 1800 g"
          :disabled="disabled"
          @commit="emit('target', formula.symbol, $event)"
        />

        <div class="ingredients">
          <div class="ingredients-head">
            <span>Ingredient</span>
            <span>Baker's %</span>
            <span></span>
          </div>
          <div
            v-for="ingredient in formula.ingredients"
            :key="ingredient.symbol"
            class="ingredient-row"
          >
            <span class="ingredient-name">{{ ingredient.symbol.replace(/_/g, " ") }}</span>
            <BuilderTextField
              label=""
              :model-value="ingredient.baker"
              placeholder="80%"
              :disabled="disabled"
              @commit="emit('ingredientBaker', formula.symbol, ingredient.symbol, $event)"
            />
            <button
              class="icon"
              title="Remove ingredient"
              :disabled="disabled"
              @click="emit('removeIngredient', formula.symbol, ingredient.symbol)"
            >
              <X :size="14" />
            </button>
          </div>
          <button
            class="add-ingredient"
            :disabled="disabled"
            @click="emit('addIngredient', formula.symbol)"
          >
            <Plus :size="14" /> Add ingredient
          </button>
        </div>
      </article>
    </div>

    <div v-if="formulas.length" class="add-row">
      <button :disabled="disabled" @click="emit('add')"><Plus :size="14" /> Formula</button>
    </div>
  </section>
</template>

<style scoped>
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.open-tool {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
}
.empty-cta {
  display: grid;
  gap: 10px;
  justify-items: start;
  margin-bottom: 4px;
  padding: 14px 16px;
  border: 1px dashed #cfd6d0;
  border-radius: 10px;
  background: #fbfcfa;
}
.empty-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.empty {
  color: #6d7972;
  font-size: 13px;
  margin: 0;
}
.empty-cta .primary {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.formulas {
  display: grid;
  gap: 12px;
}
.formula-card {
  display: grid;
  gap: 12px;
  padding: 14px;
}
.formula-head {
  display: flex;
  align-items: center;
  gap: 10px;
}
.formula-head strong {
  text-transform: capitalize;
  font-size: 15px;
}
.basis {
  flex: 1;
  font-size: 12px;
  color: #8a938c;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}
.ingredients {
  display: grid;
  gap: 6px;
}
.ingredients-head,
.ingredient-row {
  display: grid;
  grid-template-columns: 1fr 7rem 2rem;
  gap: 8px;
  align-items: center;
}
.ingredients-head {
  font-size: 11px;
  color: #8a938c;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.ingredient-name {
  text-transform: capitalize;
  font-size: 13px;
}
.add-ingredient,
.add-row button {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.add-row {
  margin-top: 8px;
}
</style>
