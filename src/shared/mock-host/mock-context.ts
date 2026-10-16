import { type DescMessage, fromJson, type JsonValue, type MessageShape } from "@bufbuild/protobuf";

import type { CommandPayload } from "@/shared/api";
import type {
  BatchActionKindJson,
  BatchActionRequestJson,
  BatchActionResultJson,
  DeviceRecordJson,
  GovernedActionJson,
  UiTextJson,
} from "@/shared/proto";

import type { createMockDocuments } from "./mock-documents";

export type MockCommandHandler = (payload: CommandPayload | undefined) => Promise<unknown>;

export type MockContext = {
  latencyMs: number;
  settle: <T>(value: T) => Promise<T>;
  audit: (
    action: GovernedActionJson,
    succeeded: boolean,
    targets: readonly string[],
    detail?: UiTextJson,
  ) => void;
  measure: (action: GovernedActionJson, succeeded: boolean, kind?: BatchActionKindJson) => void;
  documents: ReturnType<typeof createMockDocuments>;
  runBatch: (request: BatchActionRequestJson) => BatchActionResultJson;
  devices: () => DeviceRecordJson[];
};

export function fromMockJson<Desc extends DescMessage>(
  schema: Desc,
  value: unknown,
): MessageShape<Desc> {
  return fromJson(schema, JSON.parse(JSON.stringify(value)) as JsonValue);
}
