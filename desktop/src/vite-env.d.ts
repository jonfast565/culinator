/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_CULINATOR_API_URL?: string;
  readonly VITE_CULINATOR_WS_URL?: string;
  readonly VITE_CULINATOR_API_TOKEN?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
