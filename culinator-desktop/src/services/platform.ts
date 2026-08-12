/* global navigator */
/**
 * Host-environment checks shared by the API layer and the app shell.
 *
 * The same bundle runs in a browser (`npm run dev`) and inside the Tauri
 * desktop shell, so anything that depends on the shell — native dialogs, the
 * system menu bar — has to ask first.
 */

/** True when running inside the Tauri desktop shell rather than a browser tab. */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** True on macOS, where modifier keys are drawn as symbols (⌘, ⇧, ⌥). */
export function isMacPlatform(): boolean {
  return (
    typeof navigator !== "undefined" &&
    typeof navigator.platform === "string" &&
    navigator.platform.toLowerCase().includes("mac")
  );
}
