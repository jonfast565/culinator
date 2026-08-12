#!/usr/bin/env node
// Compile culinator-parser to WebAssembly and generate the JS/TS bindings the
// app imports. The frontend has no parser of its own — it calls into this.
//
// Prerequisites (one-time):
//   rustup target add wasm32-unknown-unknown
//   cargo install wasm-bindgen-cli --version 0.2.126
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const workspace = resolve(here, "..", "..");
const outDir = resolve(here, "..", "src", "generated", "wasm");
const run = (cmd, args) => execFileSync(cmd, args, { cwd: workspace, stdio: "inherit" });

run("cargo", ["build", "-p", "culinator-wasm", "--target", "wasm32-unknown-unknown", "--release"]);

// Honor CARGO_TARGET_DIR (Cursor sandboxes redirect it); fall back to ./target.
const metadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--format-version", "1", "--no-deps"], {
    cwd: workspace,
    encoding: "utf8",
  }),
);
const wasmArtifact = resolve(
  metadata.target_directory,
  "wasm32-unknown-unknown",
  "release",
  "culinator_wasm.wasm",
);

run("wasm-bindgen", [wasmArtifact, "--out-dir", outDir, "--target", "web"]);
console.log(`\nwasm bindings written to ${outDir}`);
