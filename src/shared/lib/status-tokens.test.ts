import { describe, expect, it } from "vitest";

import {
  connectionHealthTokens,
  deviceStatusTokens,
  resolveStatusToken,
  statusTextTones,
  taskStatusTokens,
} from "./status-tokens";

const sets = [
  ["connection health", connectionHealthTokens],
  ["device status", deviceStatusTokens],
  ["task status", taskStatusTokens],
] as const;

describe("resolveStatusToken", () => {
  it("returns the token declared for a known status", () => {
    expect(resolveStatusToken(deviceStatusTokens, "ONLINE")).toBe(deviceStatusTokens.tokens.ONLINE);
  });

  it.each(sets)("falls back for unknown and missing statuses in %s", (_name, set) => {
    expect(resolveStatusToken(set, "STATUS_FROM_A_NEWER_BACKEND")).toBe(set.fallback);
    expect(resolveStatusToken(set, undefined)).toBe(set.fallback);
    expect(resolveStatusToken(set, "")).toBe(set.fallback);
  });

  it.each(sets)("never resolves to an empty class list in %s", (_name, set) => {
    for (const token of [...Object.values(set.tokens), set.fallback]) {
      expect(token.trim()).not.toBe("");
    }
  });

  it("does not inherit tokens from Object.prototype", () => {
    expect(resolveStatusToken(deviceStatusTokens, "constructor")).toBe(deviceStatusTokens.fallback);
  });
});

describe("statusTextTones", () => {
  it("gives every tone its own non-empty colour", () => {
    const classes = Object.values(statusTextTones);

    expect(classes.every((token) => token.trim() !== "")).toBe(true);
    expect(new Set(classes).size).toBe(classes.length);
  });

  it("is the palette the task status tokens are built from", () => {
    expect(taskStatusTokens.tokens.DONE).toBe(statusTextTones.positive);
    expect(taskStatusTokens.fallback).toBe(statusTextTones.neutral);
  });
});
