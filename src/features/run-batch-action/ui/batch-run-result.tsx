import { CheckCircle2, LoaderCircle, RotateCcw, XCircle } from "lucide-react";

import type { DeviceRecord } from "@/entities/device";
import { ScriptOutputList } from "@/entities/script";
import { resolveBatchAction, type TaskRun } from "@/entities/task";
import { runtimeKey, useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { cn } from "@/shared/lib/utils";
import { deviceTitle, taskClass, textValue } from "@/shared/lib/workspace-view";
import type { BatchActionRequest, DeviceInputRoute } from "@/shared/proto";
import { AdbErrorKind, BatchActionKind, enumKey, TaskStatus } from "@/shared/proto";
import { ConfirmButton, ErrorState } from "@/shared/ui";

import type { BatchRunView } from "../model/batch-run-view";
import { orderDeviceResults, summarizeDeviceResults } from "../model/run-summary";

type BatchRunResultProps = {
  view: BatchRunView;
  devices: readonly DeviceRecord[];
  onReplay?: (request: BatchActionRequest) => void;
};

function InputRouteNote({ route }: { route: DeviceInputRoute | undefined }) {
  const { t } = useI18n();

  if (!route?.channelId) {
    return null;
  }

  const channel = textValue(route.channel, true, t) || route.channelId;

  return (
    <p className="mt-0.5 break-words text-xs leading-5 text-muted-foreground">
      {t("runBatchAction.via", { channel })}
      {route.failure ? ` · ${textValue(route.failure, true, t)}` : ""}
    </p>
  );
}

export function BatchRunResult({ view, devices, onReplay }: BatchRunResultProps) {
  const { t } = useI18n();

  switch (view.status) {
    case "idle":
      return null;
    case "running":
      return (
        <div
          role="status"
          className="flex items-center gap-2 rounded-md border border-border bg-background p-3 text-sm text-muted-foreground"
        >
          <LoaderCircle className="h-4 w-4 animate-spin" />
          {t("runBatchAction.running")}
        </div>
      );
    case "unreachable":
      return (
        <ErrorState
          title={t("runBatchAction.requestFailed")}
          detail={view.error == null ? undefined : errorMessage(view.error)}
        />
      );
    case "finished":
      return <FinishedRun run={view.run} devices={devices} onReplay={onReplay} />;
  }
}

type FinishedRunProps = {
  run: TaskRun;
  devices: readonly DeviceRecord[];
  onReplay?: (request: BatchActionRequest) => void;
};

function FinishedRun({ run, devices, onReplay }: FinishedRunProps) {
  const { t } = useI18n();
  const request = run.request;
  const replayed = request ? resolveBatchAction(request.kind) : undefined;
  const { task, diagnostics, deviceResults, deviceHints, inputRoutes, scriptOutputs } = run;
  const summary = summarizeDeviceResults(deviceResults);
  const names = new Map(devices.map((device) => [device.id, deviceTitle(device, false, t)]));
  const hints = new Map(deviceHints.map((item) => [item.deviceId, item.hint]));
  const routes = new Map(inputRoutes.map((route) => [route.deviceId, route]));

  return (
    <section className="space-y-3 rounded-md border border-border bg-background p-3">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h3 className="truncate text-sm font-medium">
            {textValue(task.title, true, t) || t("runBatchAction.title")}
          </h3>
          <p role="status" className="mt-1 break-words text-xs leading-5 text-muted-foreground">
            {summary.failed > 0
              ? t("runBatchAction.summary", {
                  succeeded: String(summary.succeeded),
                  count: String(summary.total),
                })
              : textValue(task.detail, true, t)}
          </p>
        </div>
        <div className="flex shrink-0 flex-col items-end gap-2">
          <span className={cn("text-xs font-medium", taskClass(enumKey(TaskStatus, task.status)))}>
            {t(runtimeKey(`enums.taskStatus.${enumKey(TaskStatus, task.status)}`))}
          </span>
          {request && onReplay ? (
            <ConfirmButton
              type="button"
              size="sm"
              variant="ghost"
              ask={replayed?.destructive === true}
              title={t("runBatchAction.confirmReplay", {
                action: replayed ? t(replayed.labelKey) : enumKey(BatchActionKind, request.kind),
                count: String(request.deviceIds.length),
              })}
              confirmLabel={t("runBatchAction.replay")}
              cancelLabel={t("common.cancel")}
              onConfirm={() => onReplay(request)}
            >
              <RotateCcw className="h-4 w-4" />
              {t("runBatchAction.replay")}
            </ConfirmButton>
          ) : null}
        </div>
      </div>

      {diagnostics.map((diagnostic) => (
        <p key={diagnostic.id} className="text-xs leading-5">
          {textValue(diagnostic.recovery, true, t)}
        </p>
      ))}

      {deviceResults.length > 0 ? (
        <ul className="space-y-2">
          {orderDeviceResults(deviceResults).map((item) => (
            <li key={item.deviceId} className="flex items-start gap-2 text-sm">
              {item.success ? (
                <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0 text-emerald-600 dark:text-emerald-400" />
              ) : (
                <XCircle className="mt-0.5 h-4 w-4 shrink-0 text-destructive" />
              )}
              <div className="min-w-0">
                <div className="truncate">
                  <span className="sr-only">
                    {item.success ? t("runBatchAction.succeeded") : t("runBatchAction.failed")}
                  </span>
                  {names.get(item.deviceId) ?? t("runBatchAction.unknownDevice")}
                </div>
                {item.success ? null : (
                  <p className="mt-0.5 break-words text-xs leading-5 text-muted-foreground">
                    {t(runtimeKey(`enums.adbErrorKind.${enumKey(AdbErrorKind, item.errorKind)}`))}
                    {item.message ? ` · ${item.message}` : ""}
                  </p>
                )}
                {item.success || !hints.get(item.deviceId) ? null : (
                  <p className="mt-0.5 break-words text-xs leading-5">
                    {textValue(hints.get(item.deviceId), true, t)}
                  </p>
                )}
                <InputRouteNote route={routes.get(item.deviceId)} />
              </div>
            </li>
          ))}
        </ul>
      ) : null}

      <ScriptOutputList
        outputs={scriptOutputs}
        titleOf={(deviceId) => names.get(deviceId) ?? t("runBatchAction.unknownDevice")}
      />
    </section>
  );
}
