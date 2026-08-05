import { createApp } from "vue";
import Root from "./app/Root.vue";
import { initParser } from "./services/wasm/parser";
import "./styles/base.css";
import "./features/bookshelf/recipe-cards-print.css";
import "./features/reading/recipe-leaf.css";
import "./features/reading/index-card-view.css";
import "./features/reading/index-card-print.css";

// The recipe parser is Rust compiled to WebAssembly. Loading it before mount
// keeps `parseUiModel` synchronous for every consumer downstream.
await initParser();
createApp(Root).mount("#root");
