<script setup lang="ts">
/* global Event, HTMLElement, KeyboardEvent */
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import {
  BookOpen,
  ChevronDown,
  Circle,
  FilePlus2,
  Home,
  Import,
  PanelTopClose,
  Save,
  Settings,
} from "lucide-vue-next";
import AppIcon from "./AppIcon.vue";
import type { AppView } from "../useNavigation";
import type { AppMenuAction, AppMenuItem, AppMenuSection } from "../appMenuModel";
import { MENU_BAR_ACCELERATOR, formatAccelerator } from "../appMenuModel";

const props = defineProps<{
  /** Built by `buildAppMenus` — shared with the native menu inside Tauri. */
  menus: AppMenuSection[];
  view: AppView;
  recipeTitle?: string;
  dirty?: boolean;
}>();

const emit = defineEmits<{ action: [action: AppMenuAction] }>();
const openMenu = ref<string | null>(null);

const menus = computed(() => props.menus);
const isEditing = computed(() => props.view === "editing" || props.view === "building");
const hideMenuBarHint = computed(() => formatAccelerator(MENU_BAR_ACCELERATOR));

function choose(item: AppMenuItem): void {
  if (item.disabled) return;
  openMenu.value = null;
  emit("action", item.action);
}

function toggle(label: string): void {
  openMenu.value = openMenu.value === label ? null : label;
}

function closeMenus(event?: KeyboardEvent): void {
  if (!event || event.key === "Escape") openMenu.value = null;
}

function closeFromDocument(event: Event): void {
  if (!(event.target as HTMLElement).closest(".app-menu-bar")) openMenu.value = null;
}

onMounted(() => {
  window.addEventListener("keydown", closeMenus);
  document.addEventListener("pointerdown", closeFromDocument);
});
onBeforeUnmount(() => {
  window.removeEventListener("keydown", closeMenus);
  document.removeEventListener("pointerdown", closeFromDocument);
});
</script>

<template>
  <header class="app-menu-bar">
    <span class="app-brand" title="Culinator" aria-hidden="true">
      <AppIcon :size="28" alt="" />
    </span>
    <nav class="menu-strip" aria-label="Application menu">
      <div
        v-for="menu in menus"
        :key="menu.label"
        class="menu-root"
        :class="{ open: openMenu === menu.label }"
      >
        <button
          class="menu-trigger"
          :disabled="menu.disabled"
          :aria-expanded="openMenu === menu.label"
          aria-haspopup="menu"
          @click="toggle(menu.label)"
        >
          {{ menu.label }} <ChevronDown :size="12" />
        </button>
        <div v-if="openMenu === menu.label" class="menu-popover" role="menu">
          <button
            v-for="item in menu.items"
            :key="item.action"
            role="menuitem"
            :class="{ divider: item.divider, destructive: item.action === 'delete' }"
            :disabled="item.disabled"
            @click="choose(item)"
          >
            <span class="item-main">
              <span>{{ item.label }}</span>
              <kbd v-if="item.accelerator">{{ formatAccelerator(item.accelerator) }}</kbd>
            </span>
            <small v-if="item.hint">{{ item.hint }}</small>
          </button>
        </div>
      </div>
    </nav>

    <div class="menu-context">
      <template v-if="recipeTitle">
        <span class="context-title">{{ recipeTitle }}</span>
        <span v-if="dirty" class="context-state"
          ><Circle :size="7" fill="currentColor" /> Edited</span
        >
        <Save v-else-if="isEditing" :size="13" aria-label="Saved" />
      </template>
      <span v-else class="context-title">
        <Home v-if="view === 'shelf'" :size="13" />
        <BookOpen v-else :size="13" />
        {{ view === "shelf" ? "Recipe shelf" : "Library" }}
      </span>
      <button class="quick-new" title="New recipe" @click="emit('action', 'new-recipe')">
        <FilePlus2 :size="14" /> New
      </button>
      <button class="quick-import" title="Import recipe" @click="emit('action', 'import-recipe')">
        <Import :size="14" />
      </button>
      <button class="quick-settings" title="Settings" @click="emit('action', 'open-settings')">
        <Settings :size="14" />
      </button>
      <button
        class="quick-hide"
        :title="`Hide menu bar (${hideMenuBarHint})`"
        @click="emit('action', 'toggle-menu-bar')"
      >
        <PanelTopClose :size="14" />
      </button>
    </div>
  </header>
</template>

<style scoped>
.app-menu-bar {
  position: relative;
  z-index: 30;
  flex: 0 0 44px;
  width: 100%;
  min-width: 0;
  display: flex;
  align-items: stretch;
  padding: 0 10px;
  color: #eef5ef;
  background: #17251e;
  border-bottom: 1px solid #0e1913;
  box-shadow: 0 1px 0 rgba(255, 255, 255, 0.06) inset;
}
.app-brand {
  flex: 0 0 auto;
  display: grid;
  place-items: center;
  align-self: center;
  width: 28px;
  height: 28px;
  margin-right: 4px;
  border-radius: 8px;
  overflow: hidden;
}
.menu-trigger,
.quick-new,
.quick-import,
.quick-settings,
.quick-hide {
  border: 0;
  background: transparent;
  color: inherit;
}
.menu-trigger:hover,
.menu-root.open .menu-trigger,
.quick-new:hover,
.quick-import:hover,
.quick-settings:hover,
.quick-hide:hover {
  background: rgba(255, 255, 255, 0.09);
}
.menu-strip {
  display: flex;
  align-items: stretch;
  min-width: 0;
}
.menu-root {
  position: relative;
  display: flex;
}
.menu-trigger {
  gap: 3px;
  height: 100%;
  padding: 0 9px;
  border-radius: 0;
  font-size: 12.5px;
}
.menu-trigger:disabled {
  opacity: 0.38;
}
.menu-popover {
  position: absolute;
  top: calc(100% + 1px);
  left: 0;
  width: 224px;
  padding: 6px;
  border: 1px solid #c8d0ca;
  border-radius: 0 0 9px 9px;
  background: #fbfcfa;
  color: #243129;
  box-shadow: 0 18px 38px -18px rgba(11, 24, 16, 0.55);
}
.menu-popover button {
  width: 100%;
  min-height: 32px;
  display: flex;
  align-items: flex-start;
  flex-direction: column;
  gap: 1px;
  padding: 7px 9px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  text-align: left;
  font-size: 13px;
}
.menu-popover button:hover:not(:disabled) {
  background: #e7efe6;
}
.menu-popover button.divider {
  margin-top: 5px;
  border-top: 1px solid #e0e5e1;
  border-radius: 0 0 6px 6px;
}
.menu-popover button.destructive {
  color: #9e3232;
}
.menu-popover small {
  font-size: 10px;
  color: #6d7972;
}
.item-main {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.item-main kbd {
  flex: 0 0 auto;
  padding: 1px 5px;
  border: 1px solid #d3d8d1;
  border-radius: 4px;
  background: #f0f3f0;
  color: #55635b;
  font-size: 10px;
  font-family: inherit;
  font-weight: 600;
}
.menu-context {
  min-width: 0;
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 8px;
  padding-left: 12px;
}
.context-title {
  min-width: 0;
  max-width: 220px;
  display: flex;
  align-items: center;
  gap: 5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #bdcbbf;
  font-size: 11px;
}
.context-state {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #e9c56c;
  font-size: 10px;
}
.quick-new,
.quick-import,
.quick-settings,
.quick-hide {
  height: 30px;
  padding: 0 9px;
  border-radius: 6px;
  font-size: 11px;
}
.quick-import,
.quick-settings,
.quick-hide {
  width: 30px;
  padding: 0;
}
@media (max-width: 980px) {
  .context-title,
  .context-state,
  .quick-new {
    display: none;
  }
  .menu-trigger {
    padding: 0 6px;
  }
}
@media (max-width: 720px) {
  .app-menu-bar {
    overflow-x: auto;
  }
  .menu-context {
    position: sticky;
    right: 0;
    background: #17251e;
  }
}
</style>
