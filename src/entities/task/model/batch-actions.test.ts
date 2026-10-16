import { create } from "@bufbuild/protobuf";
import { describe, expect, it } from "vitest";

import { BatchActionKind, BatchActionRequestSchema } from "@/shared/proto";

import { buildBatchActionRequest, pathPickerFor, resolveBatchAction } from "./batch-actions";

function pickerFor(kind: Parameters<typeof resolveBatchAction>[0]) {
  const descriptor = resolveBatchAction(kind);

  return descriptor ? pathPickerFor(descriptor) : undefined;
}

describe("pathPickerFor", () => {
  it("picks a file for a source path, limited to what the action accepts", () => {
    expect(pickerFor(BatchActionKind.INSTALL_APK)).toEqual({ target: "file", extensions: ["apk"] });
    expect(pickerFor(BatchActionKind.UPLOAD_FILE)).toEqual({
      target: "file",
      extensions: undefined,
    });
  });

  it("picks the directory that captures are written into", () => {
    expect(pickerFor(BatchActionKind.SCREENSHOT)).toEqual({ target: "directory" });
    expect(pickerFor(BatchActionKind.SCREEN_RECORD)).toEqual({ target: "directory" });
  });

  it("offers no picker for inputs that are not paths", () => {
    expect(pickerFor(BatchActionKind.INPUT_TEXT)).toBeUndefined();
    expect(pickerFor(BatchActionKind.LAUNCH_APP)).toBeUndefined();
    expect(pickerFor(BatchActionKind.RECOVER_CONNECTION)).toBeUndefined();
  });
});

describe("buildBatchActionRequest", () => {
  it("sends only the field the action declares", () => {
    expect(buildBatchActionRequest(BatchActionKind.LAUNCH_APP, ["a"], "com.example.app")).toEqual(
      create(BatchActionRequestSchema, {
        kind: BatchActionKind.LAUNCH_APP,
        deviceIds: ["a"],
        packageName: "com.example.app",
      }),
    );
    expect(buildBatchActionRequest(BatchActionKind.INSTALL_APK, ["a"], "/tmp/app.apk")).toEqual(
      create(BatchActionRequestSchema, {
        kind: BatchActionKind.INSTALL_APK,
        deviceIds: ["a"],
        path: "/tmp/app.apk",
      }),
    );
  });

  it("sends no input for an action that takes none", () => {
    expect(
      buildBatchActionRequest(BatchActionKind.RECOVER_CONNECTION, ["a", "b"], "leftover"),
    ).toEqual(
      create(BatchActionRequestSchema, {
        kind: BatchActionKind.RECOVER_CONNECTION,
        deviceIds: ["a", "b"],
      }),
    );
  });

  it("leaves an empty value out so the backend reports what is missing", () => {
    expect(buildBatchActionRequest(BatchActionKind.INPUT_TEXT, ["a"], "")).toEqual(
      create(BatchActionRequestSchema, {
        kind: BatchActionKind.INPUT_TEXT,
        deviceIds: ["a"],
      }),
    );
  });

  it("sends the clipboard text in the text field", () => {
    expect(buildBatchActionRequest(BatchActionKind.CLIPBOARD_WRITE, ["a"], "你好")).toEqual(
      create(BatchActionRequestSchema, {
        kind: BatchActionKind.CLIPBOARD_WRITE,
        deviceIds: ["a"],
        text: "你好",
      }),
    );
  });

  it("does not borrow another action's field for an action it does not know", () => {
    expect(buildBatchActionRequest(BatchActionKind.UNSPECIFIED, ["a"], "text")).toEqual(
      create(BatchActionRequestSchema, {
        kind: BatchActionKind.UNSPECIFIED,
        deviceIds: ["a"],
      }),
    );
  });
});
