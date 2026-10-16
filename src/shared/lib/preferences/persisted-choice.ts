export type PersistedChoiceOptions<T extends string> = {
  storageKey: string;
  isValid: (value: string) => value is T;
  fallback: () => T;
};

export type PersistedChoice<T extends string> = {
  read: () => T;
  write: (value: T) => void;
  coerce: (value: string) => T;
  valueFromStorageEvent: (event: StorageEvent) => T | undefined;
};

export function createPersistedChoice<T extends string>({
  storageKey,
  isValid,
  fallback,
}: PersistedChoiceOptions<T>): PersistedChoice<T> {
  const accept = (value: string | null | undefined): T | undefined =>
    value != null && isValid(value) ? value : undefined;

  return {
    read() {
      if (typeof window === "undefined") {
        return fallback();
      }

      try {
        return accept(window.localStorage.getItem(storageKey)) ?? fallback();
      } catch {
        return fallback();
      }
    },

    write(value) {
      if (typeof window === "undefined") {
        return;
      }

      try {
        window.localStorage.setItem(storageKey, value);
      } catch {}
    },

    coerce(value) {
      return accept(value) ?? fallback();
    },

    valueFromStorageEvent(event) {
      if (event.key !== storageKey) {
        return undefined;
      }

      return accept(event.newValue) ?? fallback();
    },
  };
}
