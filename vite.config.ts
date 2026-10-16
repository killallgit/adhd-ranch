import { dirname, resolve } from "path";
import { fileURLToPath } from "url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const __dirname = dirname(fileURLToPath(import.meta.url));

const uiRoot = resolve(__dirname, "src");

export default defineConfig({
  root: uiRoot,
  envDir: __dirname,
  cacheDir: resolve(__dirname, "node_modules/.vite"),
  publicDir: false,
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    outDir: resolve(__dirname, "dist"),
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(uiRoot, "index.html"),
        newFocus: resolve(uiRoot, "new-focus.html"),
        settings: resolve(uiRoot, "settings.html"),
        agentDebug: resolve(uiRoot, "agent-debug.html"),
        installHooks: resolve(uiRoot, "install-hooks.html"),
      },
    },
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: [resolve(__dirname, "vitest.setup.ts")],
    include: ["**/*.test.{ts,tsx}", "../scripts/**/*.test.mjs"],
  },
});
