import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

import { BatchActionKind, enumKey } from "@/shared/proto";

import { type BatchActionInput, listBatchActions } from "./batch-actions";

const handlerDir = path.join(process.cwd(), "src-tauri/src/workspace/batch/actions");

const inputForRequirement: Record<string, BatchActionInput> = {
  Nothing: "none",
  Text: "text",
  PackageName: "packageName",
  SourceFile: "sourcePath",
  SavePath: "savePath",
  Script: "script",
};

function contractName(rustVariant: string): string {
  return rustVariant.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toUpperCase();
}

function backendRequirements(): Map<string, string> {
  const sources = readdirSync(handlerDir)
    .filter((file) => file.endsWith(".rs"))
    .map((file) => readFileSync(path.join(handlerDir, file), "utf8"));
  const entries = sources.flatMap((source) =>
    Array.from(
      source.matchAll(
        /BatchActionSpec\s*\{\s*kind:\s*BatchActionKind::(\w+),[^}]*?requirement:\s*ActionRequirement::(\w+)/g,
      ),
    ),
  );

  return new Map(entries.map(([, kind, requirement]) => [contractName(kind), requirement]));
}

describe("batch action inputs", () => {
  const requirements = backendRequirements();

  it("reads a requirement for every action the backend describes", () => {
    expect(requirements.size).toBeGreaterThanOrEqual(listBatchActions().length);
  });

  it("asks for the same field the backend validates", () => {
    const drifted = listBatchActions()
      .map((action) => {
        const name = enumKey(BatchActionKind, action.kind);
        const requirement = requirements.get(name);
        const expected = requirement ? inputForRequirement[requirement] : undefined;

        return expected === action.input
          ? null
          : `${name}: frontend asks for ${action.input}, backend requires ${requirement ?? "nothing it declares"}`;
      })
      .filter(Boolean);

    expect(drifted).toEqual([]);
  });
});
