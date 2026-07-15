<script setup lang="ts">
import type { ConnectionStatus } from "../../services/transport/websocket-client";
defineProps<{ status: ConnectionStatus }>();
</script>
<template>
  <span class="connection-badge" :class="status"><span></span>{{ status }}</span>
</template>
