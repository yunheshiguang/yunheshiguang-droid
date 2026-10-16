import { useExtensionQuery } from "yunheshiguang/sdk";

export type FileState =
  | { state: "missing" }
  | { state: "loaded" }
  | { state: "broken"; detail: string };

export type ReferenceStatus = {
  policyFile: string;
  policy: FileState;
  policyRules: number;
  teamFile: string;
  team: FileState;
  teamMembers: number;
  licenseFile: string;
  license: FileState;
  auditFolder: string;
  syncFile: string;
  sync: FileState;
  syncFolder: string;
  assistantFile: string;
  assistant: FileState;
  redroidFile: string;
  redroid: FileState;
  vendorsFile: string;
  vendors: FileState;
  metricsFile: string;
  apiKeysFile: string;
  apiKeys: FileState;
};

export const REFERENCE_PLUGIN = "reference-extension";

export function useReferenceStatus() {
  return useExtensionQuery<ReferenceStatus>(REFERENCE_PLUGIN, "describe");
}
