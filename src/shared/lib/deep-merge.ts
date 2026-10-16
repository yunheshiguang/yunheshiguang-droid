export type PlainObject = Record<string, unknown>;

export function isPlainObject(value: unknown): value is PlainObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function deepMerge(base: PlainObject, overlay: PlainObject): PlainObject {
  const merged: PlainObject = { ...base };

  for (const [key, value] of Object.entries(overlay)) {
    const current = merged[key];

    merged[key] =
      isPlainObject(current) && isPlainObject(value) ? deepMerge(current, value) : value;
  }

  return merged;
}
