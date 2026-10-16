import { LoaderCircle } from "lucide-react";
import { useMemo, useState } from "react";

import { PhrasePicker } from "@/entities/phrase";
import { ScriptPicker } from "@/entities/script";
import {
  type BatchActionInput,
  BatchActionValueInput,
  buildBatchActionRequest,
  listBatchActions,
  resolveBatchAction,
} from "@/entities/task";
import { InputChannelList } from "@/features/input-guidance";
import { BatchRunResult, useRunBatchAction } from "@/features/run-batch-action";
import { useI18n } from "@/shared/i18n";
import { deviceTitle, textValue } from "@/shared/lib/workspace-view";
import { BatchActionKind, type DeviceRecord, enumKey } from "@/shared/proto";
import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  ConfirmButton,
} from "@/shared/ui";

type BatchPanelProps = {
  selectedDevices: readonly DeviceRecord[];
  devices: readonly DeviceRecord[];
};

const INPUT_ACTIONS = new Set<BatchActionKind>([
  BatchActionKind.INPUT_TEXT,
  BatchActionKind.CLIPBOARD_WRITE,
]);

export function BatchPanel({ selectedDevices, devices }: BatchPanelProps) {
  const { t } = useI18n();
  const actions = useMemo(() => listBatchActions(), []);
  const { run, running, view } = useRunBatchAction();
  const [kind, setKind] = useState<BatchActionKind>(
    () => actions[0]?.kind ?? BatchActionKind.SCREENSHOT,
  );
  const [values, setValues] = useState<Partial<Record<BatchActionInput, string>>>({});

  const descriptor = resolveBatchAction(kind);
  const input = descriptor?.input ?? "none";
  const destructive = descriptor?.destructive === true;
  const value = values[input] ?? "";
  const setValue = (next: string) => setValues((current) => ({ ...current, [input]: next }));
  const submit = () =>
    run(
      buildBatchActionRequest(
        kind,
        selectedDevices.map((device) => device.id),
        value,
      ),
    );

  return (
    <div className="grid h-full gap-6 xl:grid-cols-[1.1fr_0.9fr]">
      <Card className="rounded-none border-x-0 border-t-0">
        <CardHeader>
          <CardTitle>{t("batchPanel.title")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-2 md:grid-cols-4">
            {actions.map((action) => {
              const Icon = action.icon;
              const active = kind === action.kind;

              return (
                <Button
                  key={action.kind}
                  type="button"
                  variant={active ? "default" : "outline"}
                  aria-pressed={active}
                  className="h-auto min-h-14 justify-start px-3 py-3 text-left"
                  onClick={() => setKind(action.kind)}
                >
                  <Icon className="h-4 w-4 shrink-0" />
                  <span className="min-w-0 truncate">{t(action.labelKey)}</span>
                </Button>
              );
            })}
          </div>

          {input === "script" ? (
            <ScriptPicker value={value} onValueChange={setValue} />
          ) : (
            <BatchActionValueInput kind={kind} value={value} onValueChange={setValue} />
          )}
          {input === "text" ? <PhrasePicker onPick={setValue} /> : null}

          <ConfirmButton
            type="button"
            variant={destructive ? "destructive" : "default"}
            disabled={selectedDevices.length === 0 || running}
            ask={destructive}
            title={t("batchPanel.confirm.body", {
              action: descriptor ? t(descriptor.labelKey) : enumKey(BatchActionKind, kind),
              count: String(selectedDevices.length),
            })}
            confirmLabel={t("batchPanel.confirm.run")}
            cancelLabel={t("common.cancel")}
            onConfirm={submit}
          >
            {running ? <LoaderCircle className="h-4 w-4 animate-spin" /> : null}
            {running ? t("runBatchAction.running") : t("batchPanel.run")}
          </ConfirmButton>

          <BatchRunResult view={view} devices={devices} onReplay={running ? undefined : run} />
        </CardContent>
      </Card>

      <Card className="rounded-none border-x-0 border-t-0 xl:border-l">
        <CardHeader>
          <CardTitle>{t("batchPanel.selection.title")}</CardTitle>
          {selectedDevices.length > 0 ? (
            <CardDescription>
              {t("batchPanel.selection.description", { count: String(selectedDevices.length) })}
            </CardDescription>
          ) : null}
        </CardHeader>
        <CardContent className="space-y-2">
          {INPUT_ACTIONS.has(kind) ? <InputChannelList /> : null}
          {selectedDevices.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t("batchPanel.selection.empty")}</p>
          ) : (
            selectedDevices.map((device) => (
              <div key={device.id} className="rounded-md border border-border bg-background p-3">
                <div className="text-sm font-medium">{deviceTitle(device, false, t)}</div>
                <p className="mt-1 text-xs text-muted-foreground">
                  {textValue(device.model, false, t)}
                </p>
              </div>
            ))
          )}
        </CardContent>
      </Card>
    </div>
  );
}
