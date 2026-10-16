import { type LucideIcon, Maximize, Minimize2, PanelTopOpen, X } from "lucide-react";

import type { PlainMessageKey } from "@/shared/i18n";

export type WindowControlId = "minimize" | "maximize" | "close";

export type WindowControlDescriptor = {
  id: WindowControlId;
  icon: LucideIcon;
  labelKey: PlainMessageKey;
  macAccentClassName: string;
  desktopVariant: "outline" | "destructive";
};

export const windowControls: readonly WindowControlDescriptor[] = [
  {
    id: "minimize",
    icon: Minimize2,
    labelKey: "windowChrome.minimize",
    macAccentClassName: "bg-amber-500 text-white hover:bg-amber-500/90",
    desktopVariant: "outline",
  },
  {
    id: "maximize",
    icon: Maximize,
    labelKey: "windowChrome.maximize",
    macAccentClassName: "bg-emerald-500 text-white hover:bg-emerald-500/90",
    desktopVariant: "outline",
  },
  {
    id: "close",
    icon: X,
    labelKey: "windowChrome.close",
    macAccentClassName: "bg-rose-500 text-white hover:bg-rose-500/90",
    desktopVariant: "destructive",
  },
];

export function restoredMaximizeControl(control: WindowControlDescriptor): WindowControlDescriptor {
  return control.id === "maximize"
    ? { ...control, icon: PanelTopOpen, labelKey: "windowChrome.restore" }
    : control;
}
