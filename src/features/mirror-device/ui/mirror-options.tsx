import { type PlainMessageKey, runtimeKey, useI18n } from "@/shared/i18n";
import { enumKey, MAX_UINT32, ScrcpyCaptureOrientation, type ScrcpyConfig } from "@/shared/proto";
import { Input, Label, Select } from "@/shared/ui";

type Toggle = keyof Pick<
  ScrcpyConfig,
  | "videoEnabled"
  | "audioEnabled"
  | "controlEnabled"
  | "turnScreenOff"
  | "stayAwake"
  | "showTouches"
  | "keepActive"
  | "alwaysOnTop"
  | "fullscreen"
>;

const TOGGLES: Record<Toggle, PlainMessageKey> = {
  videoEnabled: "mirrorDevice.options.video",
  audioEnabled: "mirrorDevice.options.audio",
  controlEnabled: "mirrorDevice.options.control",
  turnScreenOff: "mirrorDevice.options.turnScreenOff",
  stayAwake: "mirrorDevice.options.stayAwake",
  showTouches: "mirrorDevice.options.showTouches",
  keepActive: "mirrorDevice.options.keepActive",
  alwaysOnTop: "mirrorDevice.options.alwaysOnTop",
  fullscreen: "mirrorDevice.options.fullscreen",
};

const ORIENTATIONS = ["ROTATION_0", "ROTATION_90", "ROTATION_180", "ROTATION_270"] as const;

const MBPS = 1_000_000;

type MirrorOptionsProps = {
  config: ScrcpyConfig;
  onChange: (config: ScrcpyConfig) => void;
  disabled: boolean;
};

function whole(value: string): number {
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? Math.min(Math.floor(parsed), MAX_UINT32) : 0;
}

function bitRate(megabits: string): bigint {
  const bits = Math.round(Number(megabits) * MBPS);

  return Number.isFinite(bits) && bits > 0 ? BigInt(Math.min(bits, Number.MAX_SAFE_INTEGER)) : 0n;
}

export function MirrorOptions({ config, onChange, disabled }: MirrorOptionsProps) {
  const { t } = useI18n();
  const set = (patch: Partial<ScrcpyConfig>) => onChange({ ...config, ...patch });
  const numbers = [
    { id: "scrcpy-max-size", key: "maxSize", value: config.maxSize, label: "maxSize" },
    { id: "scrcpy-max-fps", key: "maxFps", value: config.maxFps, label: "maxFps" },
  ] as const;

  return (
    <fieldset className="space-y-4" disabled={disabled}>
      <legend className="text-sm font-medium">{t("mirrorDevice.options.title")}</legend>
      <div className="grid gap-2 sm:grid-cols-2">
        {(Object.entries(TOGGLES) as [Toggle, PlainMessageKey][]).map(([toggle, labelKey]) => (
          <label key={toggle} className="flex items-start gap-2 text-sm">
            <input
              type="checkbox"
              className="mt-1"
              checked={config[toggle]}
              onChange={(event) => set({ [toggle]: event.currentTarget.checked })}
            />
            <span>{t(labelKey)}</span>
          </label>
        ))}
      </div>

      <div className="grid gap-3 sm:grid-cols-3">
        {numbers.map((field) => (
          <div key={field.id} className="space-y-1.5">
            <Label htmlFor={field.id}>{t(`mirrorDevice.options.${field.label}`)}</Label>
            <Input
              id={field.id}
              type="number"
              min={0}
              value={field.value}
              onChange={(event) => set({ [field.key]: whole(event.currentTarget.value) })}
            />
          </div>
        ))}
        <div className="space-y-1.5">
          <Label htmlFor="scrcpy-bit-rate">{t("mirrorDevice.options.bitRate")}</Label>
          <Input
            id="scrcpy-bit-rate"
            type="number"
            min={0}
            value={Number(config.videoBitRateBps) / MBPS}
            onChange={(event) => set({ videoBitRateBps: bitRate(event.currentTarget.value) })}
          />
        </div>
      </div>

      <div className="grid gap-3 sm:grid-cols-3">
        <div className="space-y-1.5">
          <Label htmlFor="scrcpy-window-title">{t("mirrorDevice.options.windowTitle")}</Label>
          <Input
            id="scrcpy-window-title"
            value={config.windowTitle}
            onChange={(event) => set({ windowTitle: event.currentTarget.value })}
          />
        </div>
        <div className="space-y-1.5">
          <Label htmlFor="scrcpy-crop">{t("mirrorDevice.options.crop")}</Label>
          <Input
            id="scrcpy-crop"
            value={config.crop}
            placeholder="1080:1920:0:0"
            onChange={(event) => set({ crop: event.currentTarget.value })}
          />
        </div>
        <div className="space-y-1.5">
          <Label htmlFor="scrcpy-orientation">{t("mirrorDevice.options.orientation")}</Label>
          <Select
            id="scrcpy-orientation"
            value={enumKey(ScrcpyCaptureOrientation, config.captureOrientation)}
            onChange={(event) =>
              set({
                captureOrientation:
                  ScrcpyCaptureOrientation[
                    event.currentTarget.value as (typeof ORIENTATIONS)[number]
                  ],
              })
            }
          >
            {ORIENTATIONS.map((orientation) => (
              <option key={orientation} value={orientation}>
                {t(runtimeKey(`enums.scrcpyCaptureOrientation.${orientation}`))}
              </option>
            ))}
          </Select>
        </div>
      </div>
    </fieldset>
  );
}
