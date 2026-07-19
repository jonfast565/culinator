import { createApp } from "vue";
import Root from "./app/Root.vue";
import { initParser } from "./services/wasm/parser";
import "./styles/base.css";

// The recipe parser is Rust compiled to WebAssembly. Loading it before mount
// keeps `parseUiModel` synchronous for every consumer downstream.
await initParser();
createApp(Root).mount("#root");
