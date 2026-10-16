import { create } from "@bufbuild/protobuf";
import {
  CircleStop,
  ClipboardPaste,
  Eraser,
  FileCode2,
  FileUp,
  Keyboard,
  type LucideIcon,
  PackageOpen,
  PackageX,
  RefreshCcw,
  Rocket,
  ScanLine,
  Video,
} from "lucide-react";

import type { PathPickRequest } from "@/shared/api";
import type { PlainMessageKey } from "@/shared/i18n";
import { BatchActionKind, type BatchActionRequest, BatchActionRequestSchema } from "@/shared/proto";

export type BatchActionInput =
  | "none"
  | "text"
  | "packageName"
  | "sourcePath"
  | "savePath"
  | "script";

export interface BatchActionDescriptor {
  kind: BatchActionKind;
  icon: LucideIcon;
  labelKey: PlainMessageKey;
  input: BatchActionInput;
  accepts?: readonly string[];
  destructive?: boolean;
  order: number;
}

export const batchActionDescriptors: readonly BatchActionDescriptor[] = [
  {
    kind: BatchActionKind.SCREENSHOT,
    icon: ScanLine,
    labelKey: "task.actions.screenshot",
    input: "savePath",
    order: 10,
  },
  {
    kind: BatchActionKind.SCREEN_RECORD,
    icon: Video,
    labelKey: "task.actions.screenRecord",
    input: "savePath",
    order: 20,
  },
  {
    kind: BatchActionKind.INSTALL_APK,
    icon: PackageOpen,
    labelKey: "task.actions.installApk",
    input: "sourcePath",
    accepts: ["apk"],
    order: 30,
  },
  {
    kind: BatchActionKind.UPLOAD_FILE,
    icon: FileUp,
    labelKey: "task.actions.uploadFile",
    input: "sourcePath",
    order: 40,
  },
  {
    kind: BatchActionKind.INPUT_TEXT,
    icon: Keyboard,
    labelKey: "task.actions.inputText",
    input: "text",
    order: 50,
  },
  {
    kind: BatchActionKind.CLIPBOARD_WRITE,
    icon: ClipboardPaste,
    labelKey: "task.actions.clipboardWrite",
    input: "text",
    order: 55,
  },
  {
    kind: BatchActionKind.LAUNCH_APP,
    icon: Rocket,
    labelKey: "task.actions.launchApp",
    input: "packageName",
    order: 60,
  },
  {
    kind: BatchActionKind.STOP_APP,
    icon: CircleStop,
    labelKey: "task.actions.stopApp",
    input: "packageName",
    order: 61,
  },
  {
    kind: BatchActionKind.CLEAR_APP_DATA,
    icon: Eraser,
    labelKey: "task.actions.clearAppData",
    input: "packageName",
    destructive: true,
    order: 62,
  },
  {
    kind: BatchActionKind.UNINSTALL_APP,
    icon: PackageX,
    labelKey: "task.actions.uninstallApp",
    input: "packageName",
    destructive: true,
    order: 63,
  },
  {
    kind: BatchActionKind.RUN_SCRIPT,
    icon: FileCode2,
    labelKey: "task.actions.runScript",
    input: "script",
    order: 65,
  },
  {
    kind: BatchActionKind.RECOVER_CONNECTION,
    icon: RefreshCcw,
    labelKey: "task.actions.recoverConnection",
    input: "none",
    order: 70,
  },
];

export const batchActionInputPlaceholderKeys: Record<
  Exclude<BatchActionInput, "none">,
  PlainMessageKey
> = {
  text: "task.inputs.text",
  packageName: "task.inputs.packageName",
  sourcePath: "task.inputs.sourcePath",
  savePath: "task.inputs.savePath",
  script: "task.inputs.script",
};

export function listBatchActions(): readonly BatchActionDescriptor[] {
  return [...batchActionDescriptors].sort((left, right) => left.order - right.order);
}

export function resolveBatchAction(kind: BatchActionKind): BatchActionDescriptor | undefined {
  return batchActionDescriptors.find((descriptor) => descriptor.kind === kind);
}

export function destructiveActions(
  kinds: readonly BatchActionKind[],
): readonly BatchActionDescriptor[] {
  const present = new Set(kinds);

  return batchActionDescriptors.filter(
    (descriptor) => descriptor.destructive === true && present.has(descriptor.kind),
  );
}

export function pathPickerFor(descriptor: BatchActionDescriptor): PathPickRequest | undefined {
  switch (descriptor.input) {
    case "sourcePath":
      return { target: "file", extensions: descriptor.accepts };
    case "savePath":
      return { target: "directory" };
    default:
      return undefined;
  }
}

const requestFieldFor: Record<
  Exclude<BatchActionInput, "none">,
  "text" | "packageName" | "path" | "scriptId"
> = {
  text: "text",
  packageName: "packageName",
  sourcePath: "path",
  savePath: "path",
  script: "scriptId",
};

export function buildBatchActionRequest(
  kind: BatchActionKind,
  deviceIds: readonly string[],
  value: string,
): BatchActionRequest {
  const input = batchActionDescriptors.find((descriptor) => descriptor.kind === kind)?.input;
  const request = create(BatchActionRequestSchema, { kind, deviceIds: [...deviceIds] });

  if (input && input !== "none" && value) {
    request[requestFieldFor[input]] = value;
  }

  return request;
}
