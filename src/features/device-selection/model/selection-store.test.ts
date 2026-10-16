import { describe, expect, it } from "vitest";

import { createDeviceSelectionStore, pickSelectedDevices } from "./selection-store";

describe("device selection store", () => {
  it("toggles a device in and out of the selection", () => {
    const store = createDeviceSelectionStore();
    const { toggle } = store.getState().actions;

    toggle("a");
    toggle("b");
    toggle("a");

    expect(store.getState().selectedIds).toEqual(["b"]);
  });

  it("adds a batch without duplicating devices that are already selected", () => {
    const store = createDeviceSelectionStore();
    const { add, toggle } = store.getState().actions;

    toggle("a");
    add(["a", "b", "c"]);

    expect(store.getState().selectedIds).toEqual(["a", "b", "c"]);
  });

  it("ignores duplicates inside one batch", () => {
    const store = createDeviceSelectionStore();

    store.getState().actions.add(["a", "a", "b"]);

    expect(store.getState().selectedIds).toEqual(["a", "b"]);
  });

  it("keeps the same state when a batch adds nothing new", () => {
    const store = createDeviceSelectionStore();
    const { add } = store.getState().actions;

    add(["a"]);
    const before = store.getState();
    add(["a"]);

    expect(store.getState()).toBe(before);
  });

  it("clears the selection", () => {
    const store = createDeviceSelectionStore();
    const { add, clear } = store.getState().actions;

    add(["a", "b"]);
    clear();

    expect(store.getState().selectedIds).toEqual([]);
  });
});

describe("pickSelectedDevices", () => {
  const devices = [{ id: "a" }, { id: "b" }, { id: "c" }];

  it("keeps the device list order rather than the selection order", () => {
    expect(pickSelectedDevices(devices, ["c", "a"]).map((device) => device.id)).toEqual(["a", "c"]);
  });

  it("ignores selected devices that are no longer connected", () => {
    expect(pickSelectedDevices(devices, ["gone", "b"]).map((device) => device.id)).toEqual(["b"]);
  });

  it("selects nothing by default", () => {
    expect(pickSelectedDevices(devices, [])).toEqual([]);
  });
});
