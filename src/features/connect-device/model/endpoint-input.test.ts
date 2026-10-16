import { describe, expect, it } from "vitest";

import { DEFAULT_DEBUG_PORT, parseEndpointInput } from "./endpoint-input";

describe("parseEndpointInput", () => {
  it("splits an address into host and port", () => {
    expect(parseEndpointInput(" 192.168.1.20:37123 ")).toEqual({
      host: "192.168.1.20",
      port: 37123,
    });
    expect(parseEndpointInput("pixel-7.local:5555")).toEqual({ host: "pixel-7.local", port: 5555 });
    expect(parseEndpointInput("[fe80::1]:5555")).toEqual({ host: "[fe80::1]", port: 5555 });
  });

  it("falls back to the default port only when one is given", () => {
    expect(parseEndpointInput("192.168.1.20", DEFAULT_DEBUG_PORT)).toEqual({
      host: "192.168.1.20",
      port: 5555,
    });
    expect(parseEndpointInput("fe80::1", DEFAULT_DEBUG_PORT)).toEqual({
      host: "fe80::1",
      port: 5555,
    });
    expect(parseEndpointInput("192.168.1.20")).toBeUndefined();
    expect(parseEndpointInput("fe80::1")).toBeUndefined();
  });

  it("rejects input that is not a bare address", () => {
    for (const input of [
      "",
      "   ",
      "192.168.1.20:99999",
      "192.168.1.20:port",
      "host/path",
      "user@host:5555",
      "host:5555?x=1",
    ]) {
      expect(parseEndpointInput(input, DEFAULT_DEBUG_PORT), input).toBeUndefined();
    }
  });
});
