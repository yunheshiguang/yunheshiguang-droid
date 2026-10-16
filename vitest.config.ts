import { configDefaults, defineConfig, mergeConfig } from "vitest/config";

import viteConfig from "./vite.config.ts";

const tests = [
  "src/**/*.test.{ts,tsx}",
  "extensions/*/web/src/**/*.test.{ts,tsx}",
  "scripts/**/*.test.ts",
];
const domTests = ["src/**/*.dom.test.{ts,tsx}", "extensions/*/web/src/**/*.dom.test.{ts,tsx}"];

export default defineConfig(async (env) =>
  mergeConfig(await viteConfig(env), {
    test: {
      projects: [
        {
          extends: true,
          test: {
            name: "node",
            environment: "node",
            include: tests,
            exclude: [...configDefaults.exclude, ...domTests],
          },
        },
        {
          extends: true,
          test: {
            name: "dom",
            environment: "happy-dom",
            include: domTests,
            setupFiles: ["./src/shared/testing/setup-dom.ts"],
          },
        },
        {
          extends: true,
          test: {
            name: "frozen-prototype",
            environment: "happy-dom",
            include: ["src/app/workspace-shell.dom.test.tsx"],
            setupFiles: [
              "./src/shared/testing/freeze-prototype.ts",
              "./src/shared/testing/setup-dom.ts",
            ],
          },
        },
      ],
    },
  }),
);
