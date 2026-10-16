export interface PackRegistry<TPack, TKey extends string = string> {
  readonly packs: readonly TPack[];
  readonly defaultKey: TKey;
  has(key: TKey): boolean;
  resolve(key: TKey): TPack | undefined;
  resolveOrDefault(key: TKey): TPack | undefined;
  next(key: TKey): TPack | undefined;
}

export function collectPacks<TPack>(
  entrypoints: Record<string, Record<string, unknown>>,
  isPack: (value: unknown) => value is TPack,
  label: string,
): TPack[] {
  const discovered: TPack[] = [];

  for (const [entrypoint, exported] of Object.entries(entrypoints)) {
    const packs = Object.values(exported).filter(isPack);

    if (packs.length === 0) {
      console.warn(`[${label}] ${entrypoint} exports no ${label} pack`);
      continue;
    }

    discovered.push(...packs);
  }

  return discovered;
}

export interface PackRegistryOptions<TPack, TKey extends string> {
  packs: readonly TPack[];
  keyOf: (pack: TPack) => TKey;
  preferredDefault: TKey;
}

export function createPackRegistry<TPack, TKey extends string = string>({
  packs,
  keyOf,
  preferredDefault,
}: PackRegistryOptions<TPack, TKey>): PackRegistry<TPack, TKey> {
  const byKey = new Map(packs.map((pack) => [keyOf(pack), pack]));
  const defaultKey = byKey.has(preferredDefault)
    ? preferredDefault
    : packs[0] !== undefined
      ? keyOf(packs[0])
      : preferredDefault;

  return {
    packs,
    defaultKey,
    has: (key) => byKey.has(key),
    resolve: (key) => byKey.get(key),
    resolveOrDefault: (key) => byKey.get(key) ?? byKey.get(defaultKey),
    next(key) {
      if (packs.length === 0) {
        return undefined;
      }

      const index = packs.findIndex((pack) => keyOf(pack) === key);

      return packs[(index + 1) % packs.length];
    },
  };
}
