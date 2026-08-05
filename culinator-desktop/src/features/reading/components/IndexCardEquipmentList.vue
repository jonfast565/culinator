<script setup lang="ts">
// Shared by the card deck and the measurement probe: the probe measures this
// exact list for every slice the packer can place.
defineProps<{ items: string[] }>();
</script>

<template>
  <ul class="equipment-list">
    <li v-for="item in items" :key="item">{{ item }}</li>
  </ul>
</template>
