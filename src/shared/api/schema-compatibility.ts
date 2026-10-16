export const EXPECTED_SCHEMA_VERSION = "2026.06.p0";

import type { MessageKey } from "@/shared/i18n";

export type SchemaCompatibility = "compatible" | "backend-newer" | "backend-older" | "unknown";

type Release = { year: number; month: number };

function parseRelease(version: string): Release | undefined {
  const match = /^(\d{4})\.(\d{2})(?:\.|$)/.exec(version.trim());

  if (!match) {
    return undefined;
  }

  return { year: Number(match[1]), month: Number(match[2]) };
}

export function assessSchemaCompatibility(
  schemaVersion: string | undefined,
  expected: string = EXPECTED_SCHEMA_VERSION,
): SchemaCompatibility {
  if (!schemaVersion) {
    return "unknown";
  }

  if (schemaVersion.trim() === expected.trim()) {
    return "compatible";
  }

  const actualRelease = parseRelease(schemaVersion);
  const expectedRelease = parseRelease(expected);

  if (!actualRelease || !expectedRelease) {
    return "unknown";
  }

  const actualOrder = actualRelease.year * 12 + actualRelease.month;
  const expectedOrder = expectedRelease.year * 12 + expectedRelease.month;

  if (actualOrder === expectedOrder) {
    return "compatible";
  }

  return actualOrder > expectedOrder ? "backend-newer" : "backend-older";
}

export function isSchemaCompatible(schemaVersion: string | undefined): boolean {
  return assessSchemaCompatibility(schemaVersion) === "compatible";
}

export const schemaCompatibilityMessageKeys = {
  compatible: "schemaCompatibility.compatible",
  "backend-newer": "schemaCompatibility.backendNewer",
  "backend-older": "schemaCompatibility.backendOlder",
  unknown: "schemaCompatibility.unknown",
} as const satisfies Record<SchemaCompatibility, MessageKey>;
