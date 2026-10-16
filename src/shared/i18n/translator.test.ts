import { describe, expect, it, vi } from "vitest";

import { createTranslator } from "./translator";

const translator = createTranslator({
  "en-US": {
    devices: { count_one: "{{count}} device", count_other: "{{count}} devices" },
    greeting: "Hello {{name}}",
  },
  "zh-CN": {
    devices: { count: "{{count}} 台设备" },
  },
});

describe("createTranslator", () => {
  it("picks the plural form from a count the backend sends as a string", () => {
    expect(translator.translate("en-US", "devices.count", { count: "1" })).toBe("1 device");
    expect(translator.translate("en-US", "devices.count", { count: "3" })).toBe("3 devices");
  });

  it("uses the plain key for a language without plural forms", () => {
    expect(translator.translate("zh-CN", "devices.count", { count: "1" })).toBe("1 台设备");
  });

  it("returns the key itself for missing copy instead of another language", () => {
    expect(translator.translate("zh-CN", "greeting", { name: "x" })).toBe("greeting");
  });

  it("leaves an unsupplied placeholder visible", () => {
    expect(translator.translate("en-US", "greeting")).toBe("Hello {{name}}");
  });

  it("keeps a count that is not a number as plain text", () => {
    expect(translator.translate("en-US", "greeting", { name: "n/a", count: "n/a" })).toBe(
      "Hello n/a",
    );
  });

  it("does not escape values, because React escapes on render", () => {
    expect(translator.translate("en-US", "greeting", { name: "<b>" })).toBe("Hello <b>");
  });

  it("reports copy it cannot find, including keys built at runtime", () => {
    const onMissingKey = vi.fn();
    const reporting = createTranslator({ "en-US": { greeting: "Hello" } }, onMissingKey);
    const command = "reveal_log_directory";

    expect(reporting.translate("en-US", "greeting")).toBe("Hello");
    expect(reporting.translate("en-US", `commands.${command}`)).toBe(
      "commands.reveal_log_directory",
    );

    expect(onMissingKey).toHaveBeenCalledTimes(1);
    expect(onMissingKey).toHaveBeenCalledWith("commands.reveal_log_directory", "en-US");
  });
});
