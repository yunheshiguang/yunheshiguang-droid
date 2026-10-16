import { create } from "@bufbuild/protobuf";
import { describe, expect, it } from "vitest";

import {
  type AdbDeviceActionResult,
  AdbDeviceActionResultSchema,
  AdbErrorKind,
} from "@/shared/proto";

import { orderDeviceResults, summarizeDeviceResults } from "./run-summary";

function result(deviceId: string, success: boolean): AdbDeviceActionResult {
  return create(AdbDeviceActionResultSchema, {
    deviceId,
    success,
    errorKind: success ? AdbErrorKind.UNSPECIFIED : AdbErrorKind.TIMEOUT,
    message: success ? "" : "timed out",
  });
}

const results = [result("a", true), result("b", false), result("c", true), result("d", false)];

describe("summarizeDeviceResults", () => {
  it("counts successes and failures across the batch", () => {
    expect(summarizeDeviceResults(results)).toEqual({ total: 4, succeeded: 2, failed: 2 });
  });

  it("treats a batch that never reached a device as empty", () => {
    expect(summarizeDeviceResults([])).toEqual({ total: 0, succeeded: 0, failed: 0 });
  });
});

describe("orderDeviceResults", () => {
  it("lists failures first and keeps the backend order inside each group", () => {
    expect(orderDeviceResults(results).map((item) => item.deviceId)).toEqual(["b", "d", "a", "c"]);
  });
});
