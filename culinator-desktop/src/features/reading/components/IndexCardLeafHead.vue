<script setup lang="ts">
import AllergenBadges from "./AllergenBadges.vue";

// Shared by the card deck and the measurement probe so a measured header height
// can never drift from the rendered one.
defineProps<{
  title: string;
  eyebrow?: string;
  summary?: string;
  allergens?: string[];
  /** Short “Continued” header used on every card after the first. */
  continuation?: boolean;
}>();
</script>

<template>
  <header v-if="continuation" class="leaf-head continuation-head">
    <h1 class="leaf-title">{{ title }}</h1>
    <p class="leaf-summary">Continued</p>
  </header>
  <header v-else class="leaf-head">
    <p class="eyebrow">{{ eyebrow }}</p>
    <h1 class="leaf-title">{{ title }}</h1>
    <p class="leaf-summary">{{ summary }}</p>
    <AllergenBadges v-if="allergens?.length" :allergens="allergens" />
  </header>
</template>
