import { create, toJsonString } from "@bufbuild/protobuf";

import { BatchActionKind, BatchActionKindSchema, FlowSchema } from "@/shared/proto";

export type FlowBody = {
  name: string;
  steps: readonly { id: string; kind: BatchActionKind; value: string }[];
};

const KIND_PREFIX = "BATCH_ACTION_KIND_";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function kindNamed(name: string): BatchActionKind {
  const full = name.startsWith(KIND_PREFIX) ? name : `${KIND_PREFIX}${name}`;

  return (
    BatchActionKindSchema.values.find((value) => value.name === full)?.number ??
    BatchActionKind.UNSPECIFIED
  );
}

function kindOf(kind: unknown): BatchActionKind | undefined {
  if (typeof kind === "string") {
    return kindNamed(kind);
  }
  if (typeof kind === "number") {
    return BatchActionKindSchema.values.some((value) => value.number === kind)
      ? kind
      : BatchActionKind.UNSPECIFIED;
  }
  return undefined;
}

export function encodeFlowDocument(flow: FlowBody): string {
  return toJsonString(FlowSchema, create(FlowSchema, { name: flow.name, steps: [...flow.steps] }), {
    alwaysEmitImplicit: true,
  });
}

export function decodeFlowDocument(body: string): FlowBody | undefined {
  let value: unknown;

  try {
    value = JSON.parse(body);
  } catch {
    return undefined;
  }

  const listed = isRecord(value) ? (value.steps ?? []) : undefined;

  if (!isRecord(value) || typeof value.name !== "string" || !Array.isArray(listed)) {
    return undefined;
  }

  const steps = listed.map((step: unknown, index) => {
    const kind = isRecord(step) ? kindOf(step.kind) : undefined;

    return isRecord(step) && kind !== undefined
      ? {
          id: typeof step.id === "string" && step.id ? step.id : `step-${index + 1}`,
          kind,
          value: typeof step.value === "string" ? step.value : "",
        }
      : undefined;
  });

  if (steps.some((step) => step === undefined)) {
    return undefined;
  }

  return { name: value.name, steps: steps.filter((step) => step !== undefined) };
}
