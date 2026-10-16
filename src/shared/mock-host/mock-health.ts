import type {
  BatchActionKindJson,
  DeviceRecordJson,
  GovernedActionJson,
  OperationHealthJson,
  ProbeHealthJson,
  WorkspaceHealthJson,
} from "@/shared/proto";

const PROBE_CAPACITY = 60;

type Stats = { runs: number; failures: number; totalMs: number; slowestMs: number; last: string };

export function createMockHealth(latencyMs: number) {
  const probes: ProbeHealthJson[] = [];
  const operations = new Map<
    string,
    { key: [GovernedActionJson, BatchActionKindJson]; stats: Stats }
  >();

  return {
    probe(devices: readonly DeviceRecordJson[]) {
      const online = devices.filter((device) => device.status === "DEVICE_STATUS_ONLINE").length;

      probes.unshift({
        probedAt: new Date().toISOString(),
        durationMs: latencyMs,
        devices: devices.length,
        online,
        unauthorized: 0,
        offline: devices.length - online,
      });
      probes.splice(PROBE_CAPACITY);
    },

    operation(
      action: GovernedActionJson,
      succeeded: boolean,
      batchAction: BatchActionKindJson = "BATCH_ACTION_KIND_UNSPECIFIED",
    ) {
      const id = `${action}/${batchAction}`;
      const entry = operations.get(id) ?? {
        key: [action, batchAction] as [GovernedActionJson, BatchActionKindJson],
        stats: { runs: 0, failures: 0, totalMs: 0, slowestMs: 0, last: "" },
      };

      entry.stats.runs += 1;
      entry.stats.failures += succeeded ? 0 : 1;
      entry.stats.totalMs += latencyMs;
      entry.stats.slowestMs = Math.max(entry.stats.slowestMs, latencyMs);
      entry.stats.last = new Date().toISOString();
      operations.set(id, entry);
    },

    health(): WorkspaceHealthJson {
      return {
        probes: [...probes],
        probeCapacity: PROBE_CAPACITY,
        operations: [...operations.values()].map(
          ({ key: [action, batchAction], stats }): OperationHealthJson => ({
            action,
            batchAction,
            runs: stats.runs,
            failures: stats.failures,
            averageMs: Math.round(stats.totalMs / stats.runs),
            slowestMs: stats.slowestMs,
            lastFinishedAt: stats.last,
          }),
        ),
        sinks: [
          {
            id: "core.health",
            label: { key: "workspace.telemetry.health.label" },
            description: { key: "workspace.telemetry.health.description" },
          },
        ],
      };
    },
  };
}
