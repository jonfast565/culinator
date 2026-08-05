<script setup lang="ts">
/* global HTMLElement, ResizeObserver */
import {
  computed,
  inject,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  shallowRef,
  watch,
} from "vue";
import type { UiOperation, UiRecipeModel } from "../../recipe-editor/model";
import type { IngredientGroup, MethodSection, NarrativeStep } from "../../recipe-editor/narrative";
import type { IndexCardFormat } from "../indexCardFormat";
import { CSS_PX_PER_INCH, INDEX_CARD_SPECS } from "../indexCardFormat";
import type {
  IndexCardMeasureContext,
  IndexCardMeasurements,
  IndexCardProbeHost,
} from "../measureIndexCard";
import {
  cacheIndexCardMeasurements,
  cachedIndexCardMeasurements,
  configureIndexCardProbeHost,
  createIndexCardProbeHost,
  destroyIndexCardProbeHost,
  indexCardMeasureSignature,
  indexCardProbeReady,
  readIndexCardMeasurements,
} from "../measureIndexCard";
import {
  indexCardIngredientsSectionLabel,
  indexCardSectionLabel,
  paginateIndexCards,
} from "../paginateIndexCards";
import { VIEW_SETTINGS_KEY } from "../composables/useViewSettings";
import IndexCardEquipmentList from "./IndexCardEquipmentList.vue";
import IndexCardLeafHead from "./IndexCardLeafHead.vue";
import IndexCardProbe from "./IndexCardProbe.vue";
import IngredientGroupList from "./IngredientGroupList.vue";
import MiseBlock from "./MiseBlock.vue";
import RecipeImage from "./RecipeImage.vue";
import RecipeStepRow from "./RecipeStepRow.vue";

const props = defineProps<{
  format: Exclude<IndexCardFormat, "full">;
  model: UiRecipeModel;
  recipeId?: string;
  editable?: boolean;
  highlightedSymbol?: string | null;
  summary: string;
  eyebrow: string;
  allergens: string[];
  ingredientGroups: IngredientGroup[];
  equipment: string[];
  sections: MethodSection[];
  operationFor: (step: NarrativeStep) => UiOperation | undefined;
}>();

const emit = defineEmits<{
  "select-symbol": [symbol: string];
  delete: [step: NarrativeStep];
}>();

const viewSettings = inject(VIEW_SETTINGS_KEY, null);

const showPager = computed(() => viewSettings?.showIndexCardPager.value ?? true);
const miseLayout = computed(() => viewSettings?.misePlacement.value ?? "top-matter");
const colocated = computed(() => miseLayout.value === "colocated");
const cardTitle = computed(() => props.model.title || "Untitled recipe");

/** Top-matter lists are dropped in the colocated (mise-by-section) layout. */
const deckIngredientGroups = computed(() => (colocated.value ? [] : props.ingredientGroups));
const deckEquipment = computed(() => (colocated.value ? [] : props.equipment));

const measureContext = computed<IndexCardMeasureContext>(() => ({
  format: props.format,
  margin: viewSettings?.indexCardMargin.value ?? "medium",
  headerScale: viewSettings?.recipeHeaderScale.value ?? "md",
  bodyScale: viewSettings?.recipeBodyScale.value ?? "md",
  annotationScale: viewSettings?.recipeAnnotationScale.value ?? "md",
  textSize: viewSettings?.textSize.value ?? "default",
}));

/** Everything the probe renders — also the cache key for one measurement pass. */
const probeContent = computed(() => ({
  title: cardTitle.value,
  eyebrow: props.eyebrow,
  summary: props.summary,
  allergens: props.allergens,
  coverImage: props.model.coverImage ?? "",
  ingredientGroups: deckIngredientGroups.value,
  equipment: deckEquipment.value,
  sections: props.sections,
  colocated: colocated.value,
}));

const signature = computed(() =>
  indexCardMeasureSignature(measureContext.value, probeContent.value),
);

const probeHost = shallowRef<IndexCardProbeHost | null>(null);
const probeTarget = computed(() => probeHost.value?.leaf ?? null);
const measurements = shallowRef<IndexCardMeasurements | null>(null);
/** Discards a pass whose fonts/images resolved after the inputs moved on. */
let measurePass = 0;

async function refreshMeasurements(): Promise<void> {
  const pending = signature.value;
  const cached = cachedIndexCardMeasurements(pending);
  measurements.value = cached;
  probeHost.value ??= createIndexCardProbeHost();
  const host = probeHost.value;
  if (!host) return;
  configureIndexCardProbeHost(host, measureContext.value);
  if (cached) return;

  const pass = ++measurePass;
  await nextTick();
  await indexCardProbeReady(host);
  if (pass !== measurePass) return;
  const measured = readIndexCardMeasurements(host);
  cacheIndexCardMeasurements(pending, measured);
  if (signature.value === pending) measurements.value = measured;
}

/**
 * Cards keep their true physical layout size and are scaled to fit a pane that
 * is narrower than the card, so pagination never depends on the pane width and
 * what the reader sees is the printed card in miniature.
 */
const deck = ref<HTMLElement | null>(null);
const paneWidth = ref(0);
let paneObserver: ResizeObserver | null = null;

const fitScale = computed(() => {
  const cardWidth = INDEX_CARD_SPECS[props.format].widthIn * CSS_PX_PER_INCH;
  if (paneWidth.value <= 0 || cardWidth <= 0) return 1;
  return Math.min(1, paneWidth.value / cardWidth);
});

const deckStyle = computed(() => ({ "--index-card-fit-scale": String(fitScale.value) }));

// The deck element is rebuilt when the mise layout changes, so follow the ref.
watch(deck, (element, previous) => {
  if (previous) paneObserver?.unobserve(previous);
  if (element) paneObserver?.observe(element);
});

onMounted(() => {
  void refreshMeasurements();
  if (typeof ResizeObserver === "undefined") return;
  paneObserver = new ResizeObserver((entries) => {
    paneWidth.value = entries[entries.length - 1].contentRect.width;
  });
  if (deck.value) paneObserver.observe(deck.value);
});

watch(signature, () => void refreshMeasurements(), { flush: "post" });

onBeforeUnmount(() => {
  measurePass += 1;
  paneObserver?.disconnect();
  paneObserver = null;
  destroyIndexCardProbeHost(probeHost.value);
  probeHost.value = null;
});

// Until the probe has been read (first paint, or a stalled web font) the packer
// falls back to font-metric estimates, which are deliberately conservative.
const pages = computed(() =>
  paginateIndexCards({
    format: props.format,
    ingredientGroups: deckIngredientGroups.value,
    equipment: deckEquipment.value,
    sections: props.sections,
    margin: measureContext.value.margin,
    headerScale: measureContext.value.headerScale,
    bodyScale: measureContext.value.bodyScale,
    annotationScale: measureContext.value.annotationScale,
    showPager: showPager.value,
    miseLayout: miseLayout.value,
    hasCover: Boolean(props.model.coverImage),
    measure: measurements.value ?? undefined,
  }),
);
</script>

<template>
  <div
    ref="deck"
    :key="miseLayout"
    class="index-card-deck"
    :data-format="format"
    :data-mise="miseLayout"
    :style="deckStyle"
  >
    <div v-for="page in pages" :key="`${miseLayout}-${page.index}`" class="index-card-fit">
      <div class="index-card-frame">
        <article class="leaf index-card-leaf">
          <figure v-if="page.showTitle && model.coverImage" class="leaf-cover">
            <RecipeImage :image-ref="model.coverImage" :recipe-id="recipeId" :alt="model.title" />
          </figure>

          <IndexCardLeafHead
            v-if="page.showTitle"
            :title="cardTitle"
            :eyebrow="eyebrow"
            :summary="summary"
            :allergens="allergens"
          />
          <IndexCardLeafHead v-else-if="page.continuation" continuation :title="cardTitle" />

          <section v-if="page.ingredientGroups.length" class="leaf-section ingredients">
            <h2 class="section-label">
              {{ indexCardIngredientsSectionLabel(page) }}
            </h2>
            <IngredientGroupList
              :groups="page.ingredientGroups"
              :selectable="editable"
              :highlighted-symbol="highlightedSymbol"
              @select="emit('select-symbol', $event)"
            />
          </section>

          <section v-if="page.equipment.length" class="leaf-section equipment">
            <h2 class="section-label">
              {{ indexCardSectionLabel("Equipment", page) }}
            </h2>
            <IndexCardEquipmentList :items="page.equipment" />
          </section>

          <section
            v-if="page.methodBlocks.length"
            class="leaf-section method"
            :class="{ colocated }"
          >
            <h2 v-if="!colocated" class="section-label">
              {{ indexCardSectionLabel("Method", page) }}
            </h2>
            <div class="steps">
              <template v-for="(block, blockIndex) in page.methodBlocks" :key="blockIndex">
                <h3 v-if="block.title" class="process-heading">{{ block.title }}</h3>
                <MiseBlock
                  v-if="block.mise"
                  :mise="block.mise"
                  :selectable="editable"
                  :highlighted-symbol="highlightedSymbol"
                  :ingredients-continued="block.miseIngredientsContinued"
                  :equipment-continued="block.miseEquipmentContinued"
                  @select="emit('select-symbol', $event)"
                />
                <p v-if="block.note" class="section-note">{{ block.note }}</p>
                <RecipeStepRow
                  v-for="step in block.steps"
                  :key="step.symbol"
                  :number="step.number"
                  :operation="operationFor(step)"
                  :text="step.text"
                  :meta="step.meta"
                  :time="step.time"
                  :recipe-id="recipeId"
                  :editable="editable"
                  :highlighted="highlightedSymbol === step.symbol"
                  @delete="emit('delete', step)"
                  @select="emit('select-symbol', $event)"
                />
              </template>
            </div>
          </section>

          <footer v-if="showPager" class="card-pager">
            Card {{ page.index + 1 }} of {{ page.total }}
          </footer>
        </article>
      </div>
    </div>
  </div>

  <Teleport v-if="probeTarget" :to="probeTarget">
    <IndexCardProbe
      :title="cardTitle"
      :eyebrow="eyebrow"
      :summary="summary"
      :allergens="allergens"
      :cover-image="model.coverImage"
      :recipe-id="recipeId"
      :ingredient-groups="deckIngredientGroups"
      :equipment="deckEquipment"
      :sections="sections"
      :colocated="colocated"
      :editable="editable"
    />
  </Teleport>
</template>
