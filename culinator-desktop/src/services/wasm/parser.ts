import init, { narrative, parse_ui_model } from "../../generated/wasm/culinator_wasm.js";

/**
 * Loader for the Rust parser compiled to WebAssembly.
 *
 * The desktop app used to carry a second, regex-based parser in TypeScript that
 * had to be kept in sync with `culinator-parser` by hand — and drifted. This
 * module is how the frontend reuses the real one instead. Parsing is ~0.2 ms,
 * so it stays synchronous and can run on every keystroke.
 *
 * `initParser()` must resolve before `parseUiModelWasm` is called; `main.ts`
 * awaits it before mounting the app.
 */

let ready = false;
let pending: Promise<void> | null = null;

/**
 * Load the parser. In the browser the module resolves itself from its bundled
 * URL; `moduleOrPath` exists so non-browser callers (SSR checks, scripts) can
 * hand over the bytes directly.
 */
export function initParser(moduleOrPath?: BufferSource): Promise<void> {
  if (ready) return Promise.resolve();
  pending ??= init(moduleOrPath ? { module_or_path: moduleOrPath } : undefined).then(() => {
    ready = true;
  });
  return pending;
}

export function isParserReady(): boolean {
  return ready;
}

/**
 * Parse `.cg` source into the editor's UI model. Never throws on malformed
 * input: the parser recovers, so you get whatever projected plus a
 * `diagnostics` array describing what it had to skip.
 */
export function parseUiModelWasm(source: string): unknown {
  if (!ready) {
    throw new Error("culinator parser wasm used before initParser() resolved");
  }
  return JSON.parse(parse_ui_model(source));
}

/** Unit system for displayed amounts; anything else keeps them as authored. */
export type WasmUnitSystem = "metric" | "us_customary" | "as_authored";
/** Whether amounts read as cooking fractions ("1/2") or decimals ("0.5"). */
export type WasmNumberStyle = "fractions" | "decimals";

/**
 * Build the reading-page narrative — ingredient groups, method sections with
 * rendered step prose, times, and per-section mise en place.
 *
 * This is the same generator the exporters use (`culinator-narrative`), so a
 * step reads identically in the app, the EPUB, and the printed page. Amounts
 * are converted and formatted here too, which is why the page no longer makes
 * a WebSocket round-trip per quantity.
 */
export function narrativeWasm(
  source: string,
  unitSystem: WasmUnitSystem,
  numberStyle: WasmNumberStyle,
): unknown {
  if (!ready) {
    throw new Error("culinator parser wasm used before initParser() resolved");
  }
  return JSON.parse(narrative(source, unitSystem, numberStyle));
}
