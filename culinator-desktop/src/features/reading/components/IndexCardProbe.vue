<script setup lang="ts">
/**
 * Offscreen measurement card: every block the deck can place, rendered once
 * with the real card components and tagged with `data-measure-key`.
 *
 * It is teleported into the probe host from `measureIndexCard.ts`, so the CSS
 * context (stage attributes, custom properties, physical width) matches a real
 * card and each block reports the height it will actually occupy. Labels that
 * can carry over are rendered in both forms — plain and “(cont.)” — because the
 * suffix can change the wrap.
 */
import { computed } from "vue";
import type { IngredientGroup, MethodSection } from "../../recipe-editor/narrative";
import { indexCardBlockKey } from "../measureIndexCard";
import { indexCardContinuationLabel } from "../paginateIndexCards";
import IndexCardEquipmentList from "./IndexCardEquipmentList.vue";
import IndexCardLeafHead from "./IndexCardLeafHead.vue";
import IngredientListRow from "./IngredientListRow.vue";
import MiseBlock from "./MiseBlock.vue";
import RecipeImage from "./RecipeImage.vue";
import RecipeStepRow from "./RecipeStepRow.vue";

const props = defineProps<{
  title: string;
  eyebrow: string;
  summary: string;
  allergens: string[];
  coverImage?: string;
  recipeId?: string;
  ingredientGroups: IngredientGroup[];
  equipment: string[];
  sections: MethodSection[];
  colocated: boolean;
  editable?: boolean;
}>();

const INGREDIENT_LABELS = ["Ingredients", "Ingredients (cont.)"];
const EQUIPMENT_LABELS = ["Equipment", "Equipment (cont.)"];
const METHOD_LABELS = ["Method", "Method (cont.)"];

function labelVariants(label: string | undefined): string[] {
  if (!label) return [];
  const continued = indexCardContinuationLabel(label);
  return continued && continued !== label ? [label, continued] : [label];
}

/**
 * Slices the packer can place: the whole remaining tail from any position, and
 * a single item when only one fits. Column balancing makes a list's height
 * depend on which items it holds, so each slice is measured as a whole list.
 */
const equipmentSlices = computed(() => {
  const slices: string[][] = [];
  const seen = new Set<string>();
  const add = (items: string[]): void => {
    if (!items.length) return;
    const key = indexCardBlockKey.equipmentList(items);
    if (seen.has(key)) return;
    seen.add(key);
    slices.push(items);
  };
  props.equipment.forEach((item, index) => {
    add(props.equipment.slice(index));
    add([item]);
  });
  return slices;
});

function hasMise(section: MethodSection): boolean {
  return section.mise.ingredients.length > 0 || section.mise.equipment.length > 0;
}
</script>

<template>
  <figure v-if="coverImage" class="leaf-cover" :data-measure-key="indexCardBlockKey.titleCover">
    <RecipeImage :image-ref="coverImage" :recipe-id="recipeId" :alt="title" />
  </figure>

  <IndexCardLeafHead
    :data-measure-key="indexCardBlockKey.titleHead"
    :title="title"
    :eyebrow="eyebrow"
    :summary="summary"
    :allergens="allergens"
  />
  <IndexCardLeafHead
    :data-measure-key="indexCardBlockKey.continuationHead"
    continuation
    :title="title"
  />

  <section v-if="ingredientGroups.length" class="leaf-section ingredients">
    <h2
      v-for="label in INGREDIENT_LABELS"
      :key="label"
      class="section-label"
      :data-measure-key="indexCardBlockKey.sectionLabel(label)"
    >
      {{ label }}
    </h2>
    <div class="ingredient-groups">
      <div
        v-for="(group, groupIndex) in ingredientGroups"
        :key="groupIndex"
        class="ingredient-group"
      >
        <h3
          v-for="variant in labelVariants(group.label)"
          :key="variant"
          class="variant-heading"
          :data-measure-key="indexCardBlockKey.groupLabel(variant)"
        >
          {{ variant }}
        </h3>
        <ul class="ingredient-list">
          <IngredientListRow
            v-for="(item, itemIndex) in group.items"
            :key="itemIndex"
            :parts="item"
            :selectable="editable"
            :data-measure-key="indexCardBlockKey.ingredientRow(item)"
          />
        </ul>
      </div>
    </div>
  </section>

  <section v-if="equipment.length" class="leaf-section equipment">
    <h2
      v-for="label in EQUIPMENT_LABELS"
      :key="label"
      class="section-label"
      :data-measure-key="indexCardBlockKey.sectionLabel(label)"
    >
      {{ label }}
    </h2>
    <IndexCardEquipmentList
      v-for="slice in equipmentSlices"
      :key="indexCardBlockKey.equipmentList(slice)"
      :items="slice"
      :data-measure-key="indexCardBlockKey.equipmentList(slice)"
    />
  </section>

  <section v-if="sections.length" class="leaf-section method" :class="{ colocated }">
    <h2
      v-for="label in METHOD_LABELS"
      :key="label"
      class="section-label"
      :data-measure-key="indexCardBlockKey.sectionLabel(label)"
    >
      {{ label }}
    </h2>
    <div class="steps" data-measure-role="steps">
      <template v-for="section in sections" :key="section.process">
        <h3
          v-for="variant in labelVariants(section.title)"
          :key="variant"
          class="process-heading"
          :data-measure-key="indexCardBlockKey.processHeading(variant)"
        >
          {{ variant }}
        </h3>
        <!-- Both label forms: a mise the packer slices resumes with “(cont.)”. -->
        <MiseBlock
          v-for="continued in hasMise(section) ? [false, true] : []"
          :key="String(continued)"
          measured
          :mise="section.mise"
          :selectable="editable"
          :ingredients-continued="continued"
          :equipment-continued="continued"
        />
        <p
          v-if="section.note"
          class="section-note"
          :data-measure-key="indexCardBlockKey.sectionNote(section.note)"
        >
          {{ section.note }}
        </p>
        <RecipeStepRow
          v-for="step in section.steps"
          :key="step.symbol"
          :number="step.number"
          :text="step.text"
          :meta="step.meta"
          :time="step.time"
          :editable="editable"
          :data-measure-key="indexCardBlockKey.step(step)"
        />
      </template>
    </div>
  </section>

  <!-- `.leaf-section` margin-top, read for both the first-of-type and later gaps. -->
  <section class="leaf-section" data-measure-role="section-gap"></section>
  <section class="leaf-section" data-measure-role="section-gap"></section>
  <footer class="card-pager" data-measure-role="pager">Card 1 of 1</footer>
</template>
