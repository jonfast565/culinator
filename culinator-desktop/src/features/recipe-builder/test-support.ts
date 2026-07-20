/// <reference types="node" />
// Node types are referenced here rather than added to `tsconfig.json`'s `types`
// array, which would make `fs` and `process` available to application code that
// runs in a browser. This file is test-only and never bundled.
import { readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { initParser } from "../../services/wasm/parser";

/**
 * Test helpers. The `web` wasm target normally resolves its own URL, but
 * `initParser` takes raw bytes for exactly this case — so the emission tests
 * run against the real Rust parser rather than a stand-in, which is the whole
 * point: a hand-written fake outline would drift from the grammar the same way
 * the deleted TypeScript parser did.
 */

const here = dirname(fileURLToPath(import.meta.url));
const SEED_DIR = resolve(here, "../../../../culinator-service/src/seed");

export async function loadParser(): Promise<void> {
  const bytes = readFileSync(resolve(here, "../../generated/wasm/culinator_wasm_bg.wasm"));
  await initParser(bytes);
}

export interface Seed {
  name: string;
  source: string;
}

export function seeds(): Seed[] {
  return readdirSync(SEED_DIR)
    .filter((name) => name.endsWith(".cg"))
    .sort()
    .map((name) => ({ name, source: readFileSync(join(SEED_DIR, name), "utf8") }));
}

export function seed(name: string): string {
  return readFileSync(join(SEED_DIR, name), "utf8");
}
