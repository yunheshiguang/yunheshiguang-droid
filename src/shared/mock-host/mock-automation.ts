import { toJson } from "@bufbuild/protobuf";
import { Code, ConnectError, type ServiceImpl } from "@connectrpc/connect";

import {
  type AutomationService,
  type BatchActionKindJson,
  type BatchActionRequestJson,
  type FlowEntryJson,
  type FlowRunJson,
  type FlowRunResultJson,
  FlowRunResultSchema,
  FlowRunSchema,
  FlowRunsSchema,
  FlowsSchema,
  type RunOptionsJson,
  type RunTargetJson,
  type SaveScheduleRequestJson,
  SaveScheduleRequestSchema,
  type ScheduleJson,
  type ScheduleResultJson,
  SchedulesSchema,
  type StartFlowRunRequestJson,
  StartFlowRunRequestSchema,
  type UiTextJson,
} from "@/shared/proto";

import { fromMockJson, type MockContext } from "./mock-context";
import { MOCK_OPERATOR } from "./mock-governance";

const KIND_PREFIX = "BATCH_ACTION_KIND_";
const ONLINE = new Set(["DEVICE_STATUS_ONLINE", "DEVICE_STATUS_BUSY", "DEVICE_STATUS_IDLE"]);

type MockFlow = { name: string; steps: { id: string; kind: BatchActionKindJson; value: string }[] };

function readFlow(body: string | undefined): MockFlow | undefined {
  try {
    const flow = JSON.parse(body ?? "") as {
      name?: string;
      steps?: { id?: string; kind?: string; value?: string }[];
    };

    return typeof flow.name === "string" && Array.isArray(flow.steps)
      ? {
          name: flow.name,
          steps: flow.steps.map((step) => ({
            id: step.id ?? "",
            kind: (step.kind?.startsWith(KIND_PREFIX)
              ? step.kind
              : `${KIND_PREFIX}${step.kind}`) as BatchActionKindJson,
            value: step.value ?? "",
          })),
        }
      : undefined;
  } catch {
    return undefined;
  }
}

function stepRequest(kind: BatchActionKindJson, value: string, deviceIds: string[]) {
  const request: BatchActionRequestJson = { kind, deviceIds };
  const field = {
    BATCH_ACTION_KIND_INPUT_TEXT: "text",
    BATCH_ACTION_KIND_CLIPBOARD_WRITE: "text",
    BATCH_ACTION_KIND_LAUNCH_APP: "packageName",
    BATCH_ACTION_KIND_SCREENSHOT: "path",
    BATCH_ACTION_KIND_SCREEN_RECORD: "path",
    BATCH_ACTION_KIND_INSTALL_APK: "path",
    BATCH_ACTION_KIND_UPLOAD_FILE: "path",
    BATCH_ACTION_KIND_RUN_SCRIPT: "scriptId",
  }[kind as string] as "text" | "packageName" | "path" | "scriptId" | undefined;

  if (field) {
    request[field] = value;
  }
  return request;
}

const refusedRun = (key: string): FlowRunResultJson => ({ accepted: false, detail: { key } });
const refusedSchedule = (key: string): ScheduleResultJson => ({ accepted: false, detail: { key } });

export function createMockAutomation(context: MockContext): ServiceImpl<typeof AutomationService> {
  const runs: FlowRunJson[] = [];
  const stops = new Set<string>();
  const schedules = new Map<string, ScheduleJson>();
  let sequence = 0;

  const resolve = (target: RunTargetJson | undefined): string[] | UiTextJson => {
    const online = context.devices().filter((device) => ONLINE.has(device.status ?? ""));
    const devices = target?.devices
      ? [...new Set(target.devices.deviceIds ?? [])]
      : target?.groupId !== undefined
        ? online
            .filter((device) => device.groupId === target.groupId)
            .map((device) => device.id ?? "")
        : target?.tag !== undefined
          ? online
              .filter((device) => device.labels?.tags?.includes(target.tag ?? ""))
              .map((device) => device.id ?? "")
          : undefined;

    if (!devices) {
      return { key: "workspace.automation.targetMissing" };
    }
    return devices.length > 0 ? devices : { key: "workspace.automation.noDevices" };
  };

  const execute = (run: FlowRunJson, flow: MockFlow, options: RunOptionsJson) => {
    let failedStep = 0;

    for (let iteration = 1; iteration <= (run.iterations ?? 1) && !failedStep; iteration += 1) {
      run.iteration = iteration;
      for (const [index, step] of flow.steps.entries()) {
        if (stops.has(run.id ?? "")) {
          break;
        }
        let result = context.runBatch(stepRequest(step.kind, step.value, run.deviceIds ?? []));
        let attempts = 1;

        while (
          result.task?.status !== "TASK_STATUS_DONE" &&
          attempts <= (options.retryAttempts ?? 0)
        ) {
          attempts += 1;
          result = context.runBatch(stepRequest(step.kind, step.value, run.deviceIds ?? []));
        }
        const done = result.task?.status === "TASK_STATUS_DONE";
        run.steps = (run.steps ?? []).map((candidate, position) =>
          position === index
            ? {
                ...candidate,
                status: done ? "STEP_STATUS_DONE" : "STEP_STATUS_FAILED",
                attempts,
                result,
              }
            : candidate,
        );
        if (!done) {
          failedStep = index + 1;
          break;
        }
      }
    }

    const stopped = stops.has(run.id ?? "");
    run.steps = (run.steps ?? []).map((step) =>
      step.status === "STEP_STATUS_PENDING" ? { ...step, status: "STEP_STATUS_SKIPPED" } : step,
    );
    run.status = stopped
      ? "RUN_STATUS_STOPPED"
      : failedStep
        ? "RUN_STATUS_FAILED"
        : "RUN_STATUS_SUCCEEDED";
    run.detail = stopped
      ? { key: "workspace.automation.stopped" }
      : failedStep
        ? { key: "workspace.automation.stepFailed", params: { step: String(failedStep) } }
        : undefined;
    run.finishedAt = new Date().toISOString();
    context.audit("GOVERNED_ACTION_RUN_AUTOMATION", !failedStep && !stopped, run.deviceIds ?? [], {
      key: "workspace.automation.audit.run",
      params: { name: run.flowName ?? "" },
    });
    context.measure("GOVERNED_ACTION_RUN_AUTOMATION", !failedStep && !stopped);
  };

  const start = ({
    flowId = "",
    target,
    options = {},
  }: StartFlowRunRequestJson): FlowRunResultJson => {
    const document = context.documents
      .list({ collection: "automation-flows" })
      .documents?.find((candidate) => candidate.id === flowId);
    const flow = readFlow(document?.body);

    if (!document) {
      return refusedRun("workspace.automation.flowMissing");
    }
    if (!flow) {
      return refusedRun("workspace.automation.flowUnreadable");
    }
    if (flow.steps.length === 0) {
      return refusedRun("workspace.automation.flowEmpty");
    }
    const deviceIds = resolve(target);
    if (!Array.isArray(deviceIds)) {
      return { accepted: false, detail: deviceIds };
    }

    sequence += 1;
    const run: FlowRunJson = {
      id: `run-${String(Date.now()).padStart(15, "0")}-${String(sequence).padStart(6, "0")}`,
      flowId,
      flowName: flow.name,
      trigger: "RUN_TRIGGER_MANUAL",
      actor: "main",
      status: "RUN_STATUS_RUNNING",
      deviceIds,
      iteration: 1,
      iterations: Math.max(1, options.repeat ?? 1),
      steps: flow.steps.map((step) => ({
        stepId: step.id,
        kind: step.kind,
        status: "STEP_STATUS_PENDING",
        attempts: 0,
      })),
      startedAt: new Date().toISOString(),
      scheduleId: "",
      target,
      options,
      operator: MOCK_OPERATOR,
    };
    runs.unshift(run);
    setTimeout(() => execute(run, flow, options), context.latencyMs);
    return { accepted: true, run: structuredClone(run) };
  };

  const save = ({ schedule }: SaveScheduleRequestJson): ScheduleResultJson => {
    const name = schedule?.name?.trim() ?? "";
    const cron = schedule?.cron?.trim() ?? "";

    if (!schedule?.id || !name) {
      return refusedSchedule("workspace.schedules.invalid.nameRequired");
    }
    if (!schedule.flowId) {
      return refusedSchedule("workspace.schedules.invalid.flowRequired");
    }
    if (!cron.startsWith("@") && cron.split(/\s+/).length !== 5) {
      return {
        accepted: false,
        detail: { key: "workspace.schedules.invalid.cron", params: { message: cron } },
      };
    }
    schedules.set(schedule.id, { ...schedule, name, cron, owner: MOCK_OPERATOR });
    context.audit("GOVERNED_ACTION_MANAGE_SCHEDULES", true, [], {
      key: "workspace.schedules.audit.saved",
      params: { name },
    });
    return { accepted: true };
  };

  const runResult = async (result: FlowRunResultJson) =>
    fromMockJson(FlowRunResultSchema, await context.settle(result));
  const scheduleResult = async (result: ScheduleResultJson) => context.settle(result);

  return {
    listFlows: async () =>
      fromMockJson(
        FlowsSchema,
        await context.settle({
          flows: (
            context.documents.list({ collection: "automation-flows" }).documents ?? []
          ).flatMap((document): FlowEntryJson[] => {
            const flow = readFlow(document.body);
            return flow ? [{ id: document.id, flow }] : [];
          }),
        }),
      ),
    startFlowRun: (request) => runResult(start(toJson(StartFlowRunRequestSchema, request))),
    stopFlowRun: ({ runId }) => {
      const run = runs.find((candidate) => candidate.id === runId);

      if (run?.status !== "RUN_STATUS_RUNNING") {
        return runResult(refusedRun("workspace.automation.runNotRunning"));
      }
      stops.add(runId);
      return runResult({ accepted: true, run: structuredClone(run) });
    },
    listFlowRuns: async () =>
      fromMockJson(FlowRunsSchema, await context.settle({ runs: structuredClone(runs) })),
    getFlowRun: async ({ runId }) => {
      const run = runs.find((candidate) => candidate.id === runId);

      if (!run) {
        throw new ConnectError("no run has this id", Code.NotFound);
      }
      return fromMockJson(FlowRunSchema, await context.settle(structuredClone(run)));
    },
    listSchedules: async () =>
      fromMockJson(
        SchedulesSchema,
        await context.settle({
          schedules: [...schedules.values()].map((schedule) => {
            const last = runs.find((run) => run.scheduleId === schedule.id);

            return {
              schedule,
              nextRunAt: schedule.enabled
                ? new Date(Date.now() + 3_600_000).toISOString()
                : undefined,
              lastRunId: last?.id ?? "",
              lastStatus: last?.status ?? "RUN_STATUS_UNSPECIFIED",
              lastRunAt: last?.startedAt,
            };
          }),
        }),
      ),
    saveSchedule: (request) => scheduleResult(save(toJson(SaveScheduleRequestSchema, request))),
    removeSchedule: ({ id }) => {
      const removed = schedules.get(id);

      if (!removed) {
        return scheduleResult({
          accepted: false,
          detail: { key: "workspace.schedules.invalid.missing" },
        });
      }
      const name = removed.name ?? "";
      schedules.delete(id);
      context.audit("GOVERNED_ACTION_MANAGE_SCHEDULES", true, [], {
        key: "workspace.schedules.audit.removed",
        params: { name },
      });
      return scheduleResult({ accepted: true });
    },
  };
}
