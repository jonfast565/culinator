import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import pluginVue from "eslint-plugin-vue";
import tseslint from "typescript-eslint";

export default tseslint.config(
  // `src/generated` is wasm-bindgen output — machine-written, not ours to lint.
  { ignores: ["dist", "node_modules", "src-tauri/target", "src/generated"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...pluginVue.configs["flat/recommended"],
  {
    files: ["**/*.vue"],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
        extraFileExtensions: [".vue"],
      },
    },
  },
  {
    files: ["**/*.{ts,vue}"],
    languageOptions: {
      globals: {
        window: "readonly",
        document: "readonly",
        localStorage: "readonly",
        crypto: "readonly",
        WebSocket: "readonly",
        URL: "readonly",
        fetch: "readonly",
        setTimeout: "readonly",
        clearTimeout: "readonly",
        console: "readonly",
      },
    },
    rules: {
      "@typescript-eslint/consistent-type-imports": "error",
      "vue/multi-word-component-names": "off",
      "vue/attribute-hyphenation": ["error", "always"],
    },
  },
  {
    files: ["scripts/**/*.mjs"],
    languageOptions: { globals: { console: "readonly", process: "readonly" } },
  },
  prettier,
);
