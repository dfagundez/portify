import { defineConfig } from "vite";

// Tauri serves the dev server on a fixed port and bundles the built output
// from ../dist, so both are pinned here rather than left to Vite's defaults.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // src-tauri is Rust; Vite reloading on it just adds noise.
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    // WebView2 on Windows 10+ and WebKit on macOS/Linux all support ES2022.
    target: "es2022",
    minify: true,
    sourcemap: false,
  },
});
