import { useCallback, useEffect, useState } from "react";

import { createPersistedChoice, type PersistedChoiceOptions } from "./persisted-choice";

export type UsePersistedChoiceOptions<T extends string> = PersistedChoiceOptions<T> & {
  watchFallback?: (onChange: () => void) => () => void;
};

export function usePersistedChoice<T extends string>(
  options: UsePersistedChoiceOptions<T>,
): [T, (value: T) => void] {
  const [{ choice, watchFallback }] = useState(() => ({
    choice: createPersistedChoice(options),
    watchFallback: options.watchFallback,
  }));
  const [value, setValue] = useState<T>(choice.read);

  useEffect(() => {
    const handleStorage = (event: StorageEvent) => {
      const next = choice.valueFromStorageEvent(event);

      if (next !== undefined) {
        setValue(next);
      }
    };

    window.addEventListener("storage", handleStorage);
    const unwatch = watchFallback?.(() => setValue(choice.read()));

    return () => {
      window.removeEventListener("storage", handleStorage);
      unwatch?.();
    };
  }, [choice, watchFallback]);

  const select = useCallback(
    (next: T) => {
      const accepted = choice.coerce(next);

      choice.write(accepted);
      setValue(accepted);
    },
    [choice],
  );

  return [value, select];
}
