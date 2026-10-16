import { describe, expect, it } from "vitest";

import {
  assessSchemaCompatibility,
  EXPECTED_SCHEMA_VERSION,
  isSchemaCompatible,
  type SchemaCompatibility,
  schemaCompatibilityMessageKeys,
} from "./schema-compatibility";

describe("assessSchemaCompatibility", () => {
  it("accepts the version the frontend was built against", () => {
    expect(assessSchemaCompatibility(EXPECTED_SCHEMA_VERSION)).toBe("compatible");
    expect(isSchemaCompatible(EXPECTED_SCHEMA_VERSION)).toBe(true);
  });

  it("treats a different revision of the same release as compatible", () => {
    expect(assessSchemaCompatibility("2026.06.p1", "2026.06.p0")).toBe("compatible");
    expect(assessSchemaCompatibility("2026.06", "2026.06.p0")).toBe("compatible");
  });

  it("orders releases across year boundaries", () => {
    expect(assessSchemaCompatibility("2027.01.p0", "2026.12.p0")).toBe("backend-newer");
    expect(assessSchemaCompatibility("2026.12.p0", "2027.01.p0")).toBe("backend-older");
  });

  it("reports an unparseable or missing version as unknown", () => {
    for (const version of [undefined, "", "   ", "next", "26.6"]) {
      expect(assessSchemaCompatibility(version)).toBe("unknown");
    }
  });

  it("ignores surrounding whitespace", () => {
    expect(assessSchemaCompatibility(` ${EXPECTED_SCHEMA_VERSION} `)).toBe("compatible");
  });

  it("has a message key for every outcome", () => {
    const outcomes: SchemaCompatibility[] = [
      "compatible",
      "backend-newer",
      "backend-older",
      "unknown",
    ];

    for (const outcome of outcomes) {
      expect(schemaCompatibilityMessageKeys[outcome]).toBeTruthy();
    }
  });
});
