import { fileURLToPath, URL } from "node:url";
import babel from "@rolldown/plugin-babel";
import tailwindcss from "@tailwindcss/vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import { defineConfig, loadEnv } from "vite";

import { getAppBuildInfo } from "./scripts/build-info.mjs";
import {
  parseWorkspaceExtensions,
  workspaceExtensions,
} from "./scripts/vite-plugin-workspace-extensions.ts";

const root = fileURLToPath(new URL(".", import.meta.url));
const host = process.env.TAURI_DEV_HOST;
const appBuildInfo = getAppBuildInfo();

export default defineConfig(async ({ mode }) => ({
  define: {
    __APP_BUILD_INFO__: JSON.stringify(appBuildInfo),
  },
  plugins: [
    react(),
    babel({ presets: [reactCompilerPreset()] }),
    tailwindcss(),
    workspaceExtensions(
      root,
      parseWorkspaceExtensions(loadEnv(mode, root, "").WORKSPACE_EXTENSIONS),
    ),
  ],
  clearScreen: false,
  build: {
    rolldownOptions: { checks: { bundlerTimings: false } },
  },
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
    dedupe: ["react", "react-dom"],
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/target/**"],
    },
  },
}));
