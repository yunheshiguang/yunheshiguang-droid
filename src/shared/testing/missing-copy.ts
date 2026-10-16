const missing: string[] = [];

export function recordMissingCopy(key: string, locale: string) {
  missing.push(`${key} (${locale})`);
}

export function takeMissingCopy(): string[] {
  return missing.splice(0);
}
