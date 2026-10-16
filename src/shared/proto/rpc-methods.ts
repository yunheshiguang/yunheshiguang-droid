import type { DescMethod, DescService } from "@bufbuild/protobuf";
import { WireType } from "@bufbuild/protobuf/wire";
import { MethodOptions_IdempotencyLevel } from "@bufbuild/protobuf/wkt";

import { compareIds } from "@/shared/lib/text";

import { local_api } from "./gen/yunheshiguang/rpc/v1/options_pb";

const generated = import.meta.glob<Record<string, unknown>>("./gen/**/*_pb.ts", { eager: true });

function isService(value: unknown): value is DescService {
  return typeof value === "object" && value !== null && "kind" in value && value.kind === "service";
}

function idOf(method: DescMethod): string {
  return method.name.replace(/(?<!^)[A-Z]/g, (letter) => `_${letter}`).toLowerCase();
}

const methods: readonly DescMethod[] = Object.values(generated)
  .flatMap((exports) => Object.values(exports).filter(isService))
  .flatMap((service) => service.methods);

function servedLocally(method: DescMethod): boolean {
  return (method.proto.options?.$unknown ?? []).some(
    (field) =>
      field.no === local_api.number &&
      field.wireType === WireType.Varint &&
      field.data.some((byte) => byte !== 0),
  );
}

export type RpcProcedure = {
  id: string;
  name: string;
  path: string;
  read: boolean;
  local: boolean;
};

export const rpcProcedures: readonly RpcProcedure[] = methods
  .map((method) => ({
    id: idOf(method),
    name: `${method.parent.name}.${method.name}`,
    path: `/${method.parent.typeName}/${method.name}`,
    read: method.idempotency === MethodOptions_IdempotencyLevel.NO_SIDE_EFFECTS,
    local: servedLocally(method),
  }))
  .sort((left, right) => compareIds(left.path, right.path));

export const localApiProcedures = rpcProcedures.filter((procedure) => procedure.local);
