import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeAll, beforeEach, describe, expect, it } from "vitest";

import { type LogGateway, setLogGateway, setWindowGateway } from "@/shared/api";
import { installMockHost } from "@/shared/mock-host";
import { idleWindowGateway } from "@/shared/testing";
import { preloadModulePage } from "@/shared/workspace-module";

import { reportUnhandledErrors } from "./error-reporting";
import { checkoutExtensions, fullAssembly } from "./registry/full-assembly";
import { WorkspaceShell } from "./workspace-shell";

let errors: string[] = [];
let warnings: string[] = [];
let stopReporting: () => void = () => {};

beforeAll(async () => {
  await Promise.all(
    fullAssembly().registry.modules.map((module) => preloadModulePage(module.component)),
  );
}, 120_000);

beforeEach(() => {
  errors = [];
  warnings = [];
  installMockHost({ latencyMs: 0 });
  const log: LogGateway = {
    error: (message) => errors.push(message),
    warn: (message) => warnings.push(message),
  };
  setLogGateway(log);
  setWindowGateway(idleWindowGateway);
  stopReporting = reportUnhandledErrors(window);
});

afterEach(() => {
  stopReporting();
});

async function openEveryModule() {
  const user = userEvent.setup();
  const main = screen.getByRole("main");
  const modules = within(await screen.findByRole("navigation", { name: "Modules" })).getAllByRole(
    "button",
  );
  await waitFor(() => expect(main).not.toBeEmptyDOMElement());

  for (const entry of modules) {
    await user.click(entry);
    await waitFor(() => expect(entry).toHaveAttribute("aria-current", "page"));
    expect(within(main).queryByText("This module encountered an error")).not.toBeInTheDocument();
  }

  return modules;
}

describe("WorkspaceShell", () => {
  const sweep = { timeout: 30_000 };

  it("opens every registered module without a crash or missing copy", sweep, async () => {
    render(<WorkspaceShell />);

    expect((await openEveryModule()).length).toBeGreaterThan(1);
    expect(warnings).toEqual([]);
    expect(errors).toEqual([]);
  });

  it(
    "opens every module the extensions in the checkout contribute, capabilities granted",
    sweep,
    async () => {
      const extensions = checkoutExtensions().map(({ extension }) => extension);
      const { registry } = fullAssembly();
      render(<WorkspaceShell registry={registry} extensions={extensions} />);

      const entries = await openEveryModule();

      expect(entries).toHaveLength(
        registry.navigation.reduce((count, group) => count + group.entries.length, 0),
      );
      expect(warnings).toEqual([]);
      expect(errors).toEqual([]);
    },
  );
});
