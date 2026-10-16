import { isPlainObject } from "@/shared/lib/deep-merge";

export type ThemeId = string;

export interface ThemePack {
  id: ThemeId;
  className: string;
  colorScheme: "light" | "dark";
  prefersDark: boolean;
  labelKey: string;
  order: number;
}

export function isThemePack(value: unknown): value is ThemePack {
  if (!isPlainObject(value)) {
    return false;
  }

  const candidate = value as Partial<ThemePack>;

  return (
    typeof candidate.id === "string" &&
    candidate.id.length > 0 &&
    typeof candidate.className === "string" &&
    (candidate.colorScheme === "light" || candidate.colorScheme === "dark") &&
    typeof candidate.prefersDark === "boolean" &&
    typeof candidate.labelKey === "string"
  );
}
