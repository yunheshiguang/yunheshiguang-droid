import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

import { BOOT_SNAPSHOT_KEY, type BootSnapshot } from "./boot-snapshot";

const indexHtml = readFileSync(new URL("../../../index.html", import.meta.url), "utf8");

const fields: Record<keyof BootSnapshot, true> = {
  lang: true,
  loadingLabel: true,
  themeClass: true,
  colorScheme: true,
};

describe("boot snapshot contract with index.html", () => {
  it("reads the key the shell writes", () => {
    expect(indexHtml).toContain(`localStorage.getItem("${BOOT_SNAPSHOT_KEY}")`);
  });

  it.each(Object.keys(fields))("replays the %s field", (field) => {
    expect(indexHtml).toContain(`snapshot.${field}`);
  });
});
