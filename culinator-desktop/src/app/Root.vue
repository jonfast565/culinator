<script setup lang="ts">
import { ref } from "vue";
import App from "./App.vue";
import InitProgress from "./components/InitProgress.vue";
import AppDialogHost from "../shared/components/AppDialogHost.vue";

const ready = ref(false);
</script>

<template>
  <InitProgress v-if="!ready" @ready="ready = true" />
  <template v-else>
    <App />
    <AppDialogHost />
  </template>
</template>
