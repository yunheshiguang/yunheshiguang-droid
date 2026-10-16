import { useI18n } from "@/shared/i18n";
import { cn } from "@/shared/lib/utils";
import { textValue } from "@/shared/lib/workspace-view";
import type { DeviceScriptOutput } from "@/shared/proto";

type ScriptOutputListProps = {
  outputs: readonly DeviceScriptOutput[];
  titleOf: (deviceId: string) => string;
};

function failed(output: DeviceScriptOutput): boolean {
  return Boolean(output.failure) || (output.exitCode !== undefined && output.exitCode !== 0);
}

function ExitStatus({ output }: { output: DeviceScriptOutput }) {
  const { t } = useI18n();

  if (output.failure) {
    return t("script.output.notRun");
  }
  if (output.exitCode === undefined) {
    return t("script.output.launched");
  }

  return t("script.output.exited", { code: String(output.exitCode) });
}

export function ScriptOutputList({ outputs, titleOf }: ScriptOutputListProps) {
  const { t } = useI18n();

  if (outputs.length === 0) {
    return null;
  }

  return (
    <ul className="space-y-3">
      {outputs.map((output) => {
        const engine = textValue(output.engine, true, t) || output.engineId;
        const device = titleOf(output.deviceId);

        return (
          <li key={output.deviceId} className="space-y-1 rounded-md border border-border p-3">
            <div className="flex items-baseline justify-between gap-3">
              <span className="truncate text-sm font-medium">{device}</span>
              <span
                className={cn(
                  "shrink-0 text-xs font-medium",
                  failed(output) ? "text-destructive" : "text-muted-foreground",
                )}
              >
                <ExitStatus output={output} />
              </span>
            </div>
            {engine ? (
              <p className="text-xs text-muted-foreground">{t("script.output.via", { engine })}</p>
            ) : null}
            {output.failure ? (
              <p className="break-words text-xs leading-5 text-destructive">
                {textValue(output.failure, true, t)}
              </p>
            ) : null}
            {output.output ? (
              <figure aria-label={t("script.output.label", { device })}>
                <pre className="max-h-64 overflow-auto whitespace-pre-wrap break-words rounded bg-muted p-2 font-mono text-xs">
                  {output.output}
                </pre>
              </figure>
            ) : null}
            {output.truncated ? (
              <p className="text-xs text-muted-foreground">{t("script.output.truncated")}</p>
            ) : null}
          </li>
        );
      })}
    </ul>
  );
}
