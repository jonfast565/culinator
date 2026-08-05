<script setup lang="ts">
import { computed, inject } from "vue";
import type { IndexCardFormat } from "../indexCardFormat";
import { indexCardSpec } from "../indexCardFormat";
import { DEFAULT_INDEX_CARD_MARGIN, INDEX_CARD_MARGIN_IN } from "../indexCardMargin";
import { recipeTypeScaleFactor } from "../recipeTypeScale";
import { VIEW_SETTINGS_KEY } from "../composables/useViewSettings";

const viewSettings = inject(VIEW_SETTINGS_KEY, null);

const format = computed<IndexCardFormat>(() => viewSettings?.indexCardFormat.value ?? "full");
const margin = computed(() => viewSettings?.indexCardMargin.value ?? DEFAULT_INDEX_CARD_MARGIN);
const spec = computed(() => indexCardSpec(format.value));

/** CSS custom properties for card frame, type roles, and margins. */
const stageStyle = computed(() => {
  const marginIn =
    format.value === "full"
      ? INDEX_CARD_MARGIN_IN[margin.value].full
      : INDEX_CARD_MARGIN_IN[margin.value].card;
  const card = spec.value;
  return {
    "--recipe-header-scale": String(
      recipeTypeScaleFactor(viewSettings?.recipeHeaderScale.value ?? "md"),
    ),
    "--recipe-body-scale": String(recipeTypeScaleFactor(viewSettings?.recipeBodyScale.value ?? "md")),
    "--recipe-annotation-scale": String(
      recipeTypeScaleFactor(viewSettings?.recipeAnnotationScale.value ?? "md"),
    ),
    "--index-card-margin": `${marginIn}in`,
    ...(card
      ? {
          "--index-card-width": `${card.widthIn}in`,
          "--index-card-aspect-ratio": `${card.widthIn} / ${card.heightIn}`,
        }
      : {}),
  };
});
</script>

<template>
  <div
    class="index-card-stage"
    :data-format="format"
    :data-type-tier="spec?.typeTier"
    :data-margin="margin"
    :style="stageStyle"
  >
    <slot />
  </div>
</template>
