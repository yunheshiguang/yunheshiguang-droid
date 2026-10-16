import { describe, expect, expectTypeOf, it } from "vitest";

import { type MessageKey, type MessageParams, runtimeKey, type Translate } from "./message-types";

const translate = ((key: string, params?: Readonly<Record<string, string>>) =>
  params ? `${key}:${Object.values(params).join(",")}` : key) as Translate;

describe("Translate", () => {
  it("checks literal keys and their params at compile time", () => {
    expect(translate("schemaCompatibility.compatible")).toBe("schemaCompatibility.compatible");
    expect(translate("schemaCompatibility.unknown", { expected: "1" })).toBe(
      "schemaCompatibility.unknown:1",
    );

    expectTypeOf<"schemaCompatibility.missing">().not.toExtend<MessageKey>();
    expectTypeOf<MessageParams<"schemaCompatibility.compatible">>().toEqualTypeOf<[]>();
    expectTypeOf<MessageParams<"schemaCompatibility.unknown">>().toEqualTypeOf<
      [Readonly<Record<"expected", string>>]
    >();
    expectTypeOf<{ version: string }>().not.toExtend<
      MessageParams<"schemaCompatibility.unknown">[0]
    >();
  });

  it("passes keys known only at runtime through unchecked", () => {
    expect(translate(runtimeKey("enums.deviceStatus.ONLINE"))).toBe("enums.deviceStatus.ONLINE");
  });
});
