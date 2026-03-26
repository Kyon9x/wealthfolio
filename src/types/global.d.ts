// Global ambient type declarations to avoid `any` for globals
declare global {
  interface Window {
    // Tauri global injected by the desktop runtime
    __TAURI__?: unknown;
  }
}

export {};

