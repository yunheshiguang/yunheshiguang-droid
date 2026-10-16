const names = new Intl.Collator("en", { numeric: true });

export function compareNames(left: string, right: string): number {
  return names.compare(left, right);
}

export function compareIds(left: string, right: string): number {
  if (left === right) {
    return 0;
  }

  return left < right ? -1 : 1;
}
