import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef, useState } from "react";

import type { DeviceRecord } from "@/entities/device";
import type { DeviceGroup } from "@/entities/group";
import { DeviceProfileDialog } from "@/features/organize-devices";

import { DeviceCard } from "./device-card";

const ESTIMATED_ROW_HEIGHT = 208;

export type DeviceGridProps = {
  devices: readonly DeviceRecord[];
  groups: readonly DeviceGroup[];
  revealPii: boolean;
};

export function DeviceGrid({ devices, groups, revealPii }: DeviceGridProps) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const [editing, setEditing] = useState<DeviceRecord | null>(null);
  const rowVirtualizer = useVirtualizer({
    count: devices.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ESTIMATED_ROW_HEIGHT,
    getItemKey: (index) => devices[index]?.id ?? index,
    overscan: 8,
  });

  return (
    <div ref={parentRef} className="min-h-[28rem] overflow-auto px-4 py-4">
      <div style={{ height: `${rowVirtualizer.getTotalSize()}px`, position: "relative" }}>
        {rowVirtualizer.getVirtualItems().map((virtualRow) => (
          <div
            key={virtualRow.key}
            ref={rowVirtualizer.measureElement}
            data-index={virtualRow.index}
            className="absolute left-0 top-0 w-full px-0 pb-3"
            style={{ transform: `translateY(${virtualRow.start}px)` }}
          >
            <DeviceCard
              device={devices[virtualRow.index]}
              revealPii={revealPii}
              onEditProfile={setEditing}
            />
          </div>
        ))}
      </div>
      <DeviceProfileDialog device={editing} groups={groups} onClose={() => setEditing(null)} />
    </div>
  );
}
