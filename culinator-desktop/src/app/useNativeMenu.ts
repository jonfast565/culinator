import { onBeforeUnmount, onMounted, watch } from "vue";
import type { Ref } from "vue";
import type { AppMenuAction, AppMenuSection } from "./appMenuModel";
import { isTauri } from "../services/platform";

/**
 * Mirrors the app menu onto the Tauri shell's native menu (the macOS menu bar,
 * or the window menu on Windows/Linux), so the in-app menu bar can be hidden
 * without losing any commands.
 *
 * The menu is pushed to Rust as a flat spec and rebuilt there on every change —
 * labels here are state-dependent ("Use US units" / "Use metric units"), so
 * there is no static menu to declare once.
 */

/** Rust → webview event carrying the id of the chosen menu item. */
export const MENU_ACTION_EVENT = "culinator://menu-action";

export interface NativeMenuItemSpec {
  /** The `AppMenuAction`, used as the native menu item id. */
  id: string;
  label: string;
  enabled: boolean;
  separatorBefore: boolean;
  accelerator?: string;
}

export interface NativeMenuSectionSpec {
  label: string;
  enabled: boolean;
  items: NativeMenuItemSpec[];
}

/**
 * Flatten the menu model into what the shell needs. Accelerators are dropped
 * from disabled items: a native key equivalent is registered whether or not the
 * item can run, and two enabled items must never claim the same key.
 */
export function toNativeMenuSpec(sections: AppMenuSection[]): NativeMenuSectionSpec[] {
  return sections.map((section) => {
    const sectionEnabled = section.disabled !== true;
    return {
      label: section.label,
      enabled: sectionEnabled,
      items: section.items.map((item) => {
        const enabled = sectionEnabled && item.disabled !== true;
        return {
          id: item.action,
          label: item.label,
          enabled,
          separatorBefore: item.divider === true,
          ...(enabled && item.accelerator ? { accelerator: item.accelerator } : {}),
        };
      }),
    };
  });
}

/**
 * Keeps the native menu in step with `menus` and routes its clicks back into
 * the same action handler the in-app menu bar uses. Outside Tauri this is inert
 * and reports `active: false`, which is also what tells the app it still owns
 * the menu accelerators itself.
 */
export function useNativeMenu(
  menus: Ref<AppMenuSection[]>,
  onAction: (action: AppMenuAction) => void,
): { active: boolean } {
  const active = isTauri();
  if (!active) return { active };

  let queued: NativeMenuSectionSpec[] | null = null;
  let flushing = false;

  async function flush(): Promise<void> {
    if (flushing) return;
    flushing = true;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      while (queued) {
        const next = queued;
        queued = null;
        await invoke("set_app_menu", { menu: next });
      }
    } catch (error) {
      console.error("Could not update the native menu", error);
    } finally {
      flushing = false;
    }
  }

  watch(
    menus,
    (sections) => {
      queued = toNativeMenuSpec(sections);
      void flush();
    },
    { immediate: true, flush: "post" },
  );

  let unlisten: (() => void) | null = null;
  let disposed = false;
  onMounted(async () => {
    const { listen } = await import("@tauri-apps/api/event");
    const stop = await listen<string>(MENU_ACTION_EVENT, (event) => {
      onAction(event.payload as AppMenuAction);
    });
    // The component may have been torn down while `listen` was in flight.
    if (disposed) stop();
    else unlisten = stop;
  });
  onBeforeUnmount(() => {
    disposed = true;
    unlisten?.();
    unlisten = null;
  });

  return { active };
}
