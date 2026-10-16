import { runtimeKey, type Translate } from "@/shared/i18n";
import { textValue } from "@/shared/lib/workspace-view";
import {
  ConnectionHealth,
  ConnectionType,
  DeviceStatus,
  DiagnosticCode,
  enumKey,
  PiiClass,
  type SearchableText,
  type UiText,
} from "@/shared/proto";

import type { DeviceRecord } from "./types";

function searchTokens(value: SearchableText | undefined, includePii: boolean): string[] {
  if (!value?.searchable) {
    return [];
  }

  if (includePii || value.piiClass === PiiClass.NONE) {
    return [value.displayValue, value.canonicalValue, value.redactedValue, ...value.aliases].filter(
      Boolean,
    );
  }

  return value.redactedValue ? [value.redactedValue] : [];
}

function uiTextTokens(value: UiText | undefined, translate?: Translate): string[] {
  if (!value) {
    return [];
  }
  const shown = translate ? textValue(value, true, translate) : value.key;

  return [shown, value.fallback, ...Object.values(value.params)].filter(Boolean);
}

function enumTokens(device: DeviceRecord, translate?: Translate): string[] {
  const facets = [
    ["deviceStatus", enumKey(DeviceStatus, device.status)],
    ["connectionType", enumKey(ConnectionType, device.connection)],
    ["connectionHealth", enumKey(ConnectionHealth, device.health)],
  ] as const;

  return facets
    .flatMap(([group, key]) => [key, translate?.(runtimeKey(`enums.${group}.${key}`))])
    .concat(enumKey(DiagnosticCode, device.diagnostic))
    .filter((token): token is string => Boolean(token));
}

function labelTokens(device: DeviceRecord): string[] {
  const labels = device.labels;

  if (!labels) {
    return [];
  }

  const number = labels.number > 0 ? [`#${labels.number}`, String(labels.number)] : [];

  return [labels.alias, ...labels.tags, ...number].filter(Boolean);
}

export function deviceSearchText(
  device: DeviceRecord,
  includePii: boolean,
  translate?: Translate,
): string {
  const fields = [
    device.name,
    device.model,
    device.group,
    device.note,
    device.labels?.note,
    ...device.tags,
  ];

  return fields
    .flatMap((field) => {
      if (!field) {
        return [];
      }

      return "displayValue" in field
        ? searchTokens(field, includePii)
        : uiTextTokens(field, translate);
    })
    .concat(labelTokens(device))
    .concat(enumTokens(device, translate))
    .concat(device.groupId)
    .join(" ")
    .toLowerCase();
}
