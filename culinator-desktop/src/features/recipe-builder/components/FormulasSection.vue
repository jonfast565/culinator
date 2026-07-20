<script setup lang="ts">
import { Plus, Trash2, X } from "lucide-vue-next";
import type { BuilderFormula } from "../composables/useRecipeBuilder";
import BuilderTextField from "./BuilderTextField.vue";

/**
 * Baker's-percentage formulas — flour-relative ratios that scale to a target
 * weight. A niche construct (no seed uses it), so the editor stays to the
 * essentials: a target and per-ingredient baker's percentages.
 */
defineProps<{ formulas: BuilderFormula[]; disabled?: boolean }>();

const emit = defineEmits<{
  target: [symbol: string, value: string];
  ingredientBaker: [formula: string, ingredient: string, value: string];
  add: [];
  remove: [symbol: string];
  addIngredient: [formula: string];
  removeIngredient: [formula: string, ingredient: string];
}>();
</script>

<template>
  <section id="builder-formulas" class="panel builder-section">
    <div class="panel-header">
      <h3>Formulas</h3>
    </div>

    <p v-if="!formulas.length" class="empty">
      No formulas. Add one to express ingredients as baker's percentages.
    </p>

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

    <div class="add-row">
      <button :disabled="disabled" @click="emit('add')"><Plus :size="14" /> Formula</button>
    </div>
  </section>
</template>

<style scoped>
.empty {
  color: #8a938c;
  font-size: 13px;
  margin: 0 0 12px;
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
.ingredients-head {
  display: grid;
  grid-template-columns: 1fr 120px auto;
  gap: 8px;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: #8a938c;
}
.ingredient-row {
  display: grid;
  grid-template-columns: 1fr 120px auto;
  gap: 8px;
  align-items: center;
}
.ingredient-name {
  text-transform: capitalize;
  font-size: 13px;
}
.icon {
  width: 32px;
  height: 32px;
  padding: 0;
  display: grid;
  place-items: center;
}
.icon.danger {
  color: #a83737;
}
.add-ingredient,
.add-row button {
  justify-self: start;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 13px;
  padding: 5px 10px;
}
.add-row {
  margin-top: 12px;
}
</style>
