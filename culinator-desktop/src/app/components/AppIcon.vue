<script setup lang="ts">
defineProps<{
  size?: number;
  alt?: string;
}>();
</script>

<template>
  <img
    class="app-icon"
    src="/app-icon.png"
    :alt="alt ?? 'Culinator'"
    :width="size ?? 32"
    :height="size ?? 32"
    decoding="async"
  />
</template>

<style scoped>
.app-icon {
  display: block;
  object-fit: contain;
}
</style>
