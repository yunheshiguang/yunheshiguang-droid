import { type ComponentType, createElement, lazy } from "react";

const preloaders = new WeakMap<ComponentType, () => Promise<unknown>>();

export function lazyModulePage<Module>(
  load: () => Promise<Module>,
  pick: (module: Module) => ComponentType,
): ComponentType {
  let loading: Promise<{ default: ComponentType }> | undefined;
  const loadPage = () => {
    loading ??= load().then(
      (module) => ({ default: pick(module) }),
      (error: unknown) => {
        loading = undefined;
        current = lazy(loadPage);
        throw error;
      },
    );
    return loading;
  };
  let current = lazy(loadPage);
  function ModulePage() {
    return createElement(current);
  }

  preloaders.set(ModulePage, loadPage);
  return ModulePage;
}

export async function preloadModulePage(page: ComponentType): Promise<void> {
  await preloaders.get(page)?.();
}
