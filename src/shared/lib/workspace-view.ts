import { runtimeKey, type Translate } from "@/shared/i18n";
import {
  ConnectionHealth,
  type DeviceGroup,
  type DeviceRecord,
  enumKey,
  PiiClass,
  type SearchableText,
  type UiText,
  VisibilityMode,
} from "@/shared/proto";

import {
  connectionHealthTokens,
  deviceStatusTokens,
  groupToneTokens,
  installStatusTokens,
  resolveStatusToken,
  taskStatusTokens,
} from "./status-tokens";

type DisplayText = Pick<UiText, "key" | "params" | "fallback">;

export const UNGROUPED_GROUP_ID = "ungrouped";
const UNGROUPED_LABEL_KEY = "workspace.device.group.default";

type SensitiveText = Pick<
  SearchableText,
  "displayValue" | "redactedValue" | "visibility" | "piiClass"
>;

export function textValue(
  value: DisplayText | SensitiveText | undefined,
  revealPii: boolean,
  translate?: Translate,
) {
  if (!value) {
    return "";
  }

  if ("key" in value) {
    if (translate && value.key) {
      const translated = translate(runtimeKey(value.key), value.params);

      if (translated !== value.key) {
        return translated;
      }
    }

    return value.fallback || value.key;
  }

  if (
    revealPii ||
    value.visibility === VisibilityMode.VISIBLE ||
    value.piiClass === PiiClass.NONE
  ) {
    return value.displayValue;
  }

  return value.redactedValue || value.displayValue;
}

export function deviceTitle(
  device: Pick<DeviceRecord, "name" | "labels">,
  revealPii: boolean,
  translate?: Translate,
): string {
  return device.labels?.alias || textValue(device.name, revealPii, translate);
}

export function healthClass(health: ConnectionHealth) {
  return resolveStatusToken(connectionHealthTokens, enumKey(ConnectionHealth, health));
}

export function statusClass(status: string) {
  return resolveStatusToken(deviceStatusTokens, status);
}

export function taskClass(status: string) {
  return resolveStatusToken(taskStatusTokens, status);
}

export function installStatusClass(status: string) {
  return resolveStatusToken(installStatusTokens, status);
}

export function groupToneClass(tone: string | undefined) {
  return resolveStatusToken(groupToneTokens, tone);
}

export function groupLabel(
  groups: readonly Pick<DeviceGroup, "id" | "name">[],
  id: string,
  t: Translate,
): string | undefined {
  const named = groups.find((group) => group.id === id)?.name;

  if (named) {
    return textValue(named, true, t) || undefined;
  }
  return id === UNGROUPED_GROUP_ID ? t(runtimeKey(UNGROUPED_LABEL_KEY)) : undefined;
}
