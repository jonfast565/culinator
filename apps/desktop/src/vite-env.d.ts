/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_CULINOGRAPH_API_URL?: string;
  readonly VITE_CULINOGRAPH_WS_URL?: string;
  readonly VITE_CULINOGRAPH_API_TOKEN?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
