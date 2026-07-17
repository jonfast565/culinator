<script setup lang="ts">
import { computed, inject } from "vue";
import { ChevronLeft, Scale, UtensilsCrossed } from "lucide-vue-next";
import { UNIT_DISPLAY_KEY } from "../composables/useUnitDisplay";
import UnitConverter from "./UnitConverter.vue";
import { UNIT_GROUPS } from "../unitsCatalog";

const emit = defineEmits<{ (event: "back"): void }>();

const units = inject(UNIT_DISPLAY_KEY, null);
const unitSystem = computed(() => units?.unitSystem.value ?? "metric");

function toggleSystem(): void {
  units?.toggleUnitSystem();
}
</script>

<template>
  <div class="measures-view">
    <header class="measures-head">
      <button class="ghost" @click="emit('back')"><ChevronLeft :size="16" /> Shelf</button>
      <div class="brand">
        <span class="brand-mark"><UtensilsCrossed :size="18" /></span>
        <span>
          <strong>Measures</strong>
          <small>Kitchen unit conversion</small>
        </span>
      </div>
      <button class="ghost unit-toggle" @click="toggleSystem">
        <Scale :size="15" />
        {{ unitSystem === "metric" ? "Metric" : "US customary" }}
      </button>
    </header>

    <main class="measures-stage">
      <section class="intro card">
        <h1>Convert cooking measures</h1>
        <p>
          Translate mass, volume, temperature, time, and length using the same conversion engine
          that powers recipe reading and formulas.
        </p>
        <ul class="supported">
          <li v-for="group in UNIT_GROUPS" :key="group.id">{{ group.label }}</li>
        </ul>
      </section>

      <UnitConverter :unit-system="unitSystem" />
    </main>
  </div>
</template>

<style scoped>
.measures-view {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: radial-gradient(120% 80% at 50% -10%, #efece2 0%, #e7e3d6 55%, #e0dbcb 100%);
}
.measures-head {
  display: flex;
  align-items: center;
  gap: 16px;
  min-height: 72px;
  padding: 12px 18px;
  background: white;
  border-bottom: 1px solid #d8ddd9;
}
.measures-head .ghost {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 34px;
  padding: 0 12px;
  font-size: 13px;
}
.brand {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}
.brand-mark {
  display: grid;
  place-items: center;
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: #d9f0df;
  color: #1f5130;
}
.brand strong {
  display: block;
  font-size: 17px;
}
.brand small {
  color: #718078;
}
.unit-toggle {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.measures-stage {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: clamp(24px, 4vw, 56px) 20px;
  display: grid;
  gap: 18px;
  max-width: 760px;
  margin: 0 auto;
  width: 100%;
}
.intro {
  padding: 22px 24px;
}
.intro h1 {
  margin: 0 0 8px;
  font-family: "Iowan Old Style", "Palatino Linotype", Palatino, Georgia, serif;
  font-size: clamp(28px, 4vw, 36px);
}
.intro p {
  margin: 0;
  color: #536059;
  line-height: 1.5;
}
.supported {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 16px 0 0;
  padding: 0;
  list-style: none;
}
.supported li {
  padding: 4px 10px;
  border-radius: 999px;
  background: #eef3ef;
  color: #28643b;
  font-size: 12px;
  font-weight: 600;
}
</style>
