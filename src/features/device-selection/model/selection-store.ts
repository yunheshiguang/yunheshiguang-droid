import { createStore, useStore } from "zustand";

export type DeviceSelectionActions = {
  toggle: (id: string) => void;
  add: (ids: readonly string[]) => void;
  clear: () => void;
};

type DeviceSelectionState = {
  selectedIds: readonly string[];
  actions: DeviceSelectionActions;
};

export function createDeviceSelectionStore() {
  return createStore<DeviceSelectionState>()((set) => ({
    selectedIds: [],
    actions: {
      toggle: (id) =>
        set((state) => ({
          selectedIds: state.selectedIds.includes(id)
            ? state.selectedIds.filter((item) => item !== id)
            : [...state.selectedIds, id],
        })),
      add: (ids) =>
        set((state) => {
          const selected = new Set(state.selectedIds);
          const additions = [...new Set(ids)].filter((id) => !selected.has(id));

          return additions.length === 0
            ? state
            : { selectedIds: [...state.selectedIds, ...additions] };
        }),
      clear: () => set({ selectedIds: [] }),
    },
  }));
}

const deviceSelectionStore = createDeviceSelectionStore();

export function useSelectedDeviceIds(): readonly string[] {
  return useStore(deviceSelectionStore, (state) => state.selectedIds);
}

export function useDeviceSelectionActions(): DeviceSelectionActions {
  return useStore(deviceSelectionStore, (state) => state.actions);
}

export function useIsDeviceSelected(id: string): boolean {
  return useStore(deviceSelectionStore, (state) => state.selectedIds.includes(id));
}

export function countSelected(
  availableIds: readonly string[],
  selectedIds: readonly string[],
): number {
  const selected = new Set(selectedIds);

  return availableIds.filter((id) => selected.has(id)).length;
}

export function pickSelectedDevices<T extends { id: string }>(
  devices: readonly T[],
  selectedIds: readonly string[],
): T[] {
  if (selectedIds.length === 0) {
    return [];
  }

  const selected = new Set(selectedIds);

  return devices.filter((device) => selected.has(device.id));
}
