import { act } from "@testing-library/react";
import { afterEach, vi } from "vitest";
import type * as Zustand from "zustand";

const actual = await vi.importActual<typeof Zustand>("zustand");
const resets = new Set<() => void>();

function track<T>(store: Zustand.StoreApi<T>): Zustand.StoreApi<T> {
  const initialState = store.getInitialState();

  resets.add(() => store.setState(initialState, true));

  return store;
}

const createStoreUncurried = <T>(creator: Zustand.StateCreator<T>) =>
  track(actual.createStore(creator));

const createUncurried = <T>(creator: Zustand.StateCreator<T>) => {
  const useBoundStore = actual.create(creator);

  track(useBoundStore);

  return useBoundStore;
};

afterEach(() => {
  act(() => {
    for (const reset of resets) {
      reset();
    }
  });
});

export const zustandWithReset = {
  ...actual,
  createStore: (<T>(creator?: Zustand.StateCreator<T>) =>
    creator ? createStoreUncurried(creator) : createStoreUncurried) as typeof actual.createStore,
  create: (<T>(creator?: Zustand.StateCreator<T>) =>
    creator ? createUncurried(creator) : createUncurried) as typeof actual.create,
};
