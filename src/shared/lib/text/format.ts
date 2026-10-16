export function formatCompactNumber(value: number, locale: string) {
  return new Intl.NumberFormat(locale, {
    notation: "compact",
    maximumFractionDigits: 1,
  }).format(value);
}

export function formatPercent(value: number | undefined, locale: string) {
  if (value === undefined) {
    return "-";
  }

  return `${new Intl.NumberFormat(locale, {
    maximumFractionDigits: 1,
  }).format(value)}%`;
}

export function formatLatency(value: number | undefined, locale: string) {
  if (value === undefined) {
    return "-";
  }

  return `${new Intl.NumberFormat(locale, {
    maximumFractionDigits: 0,
  }).format(value)}ms`;
}

export function formatMetricValue(value: number | undefined, locale: string) {
  if (value === undefined) {
    return "-";
  }

  return new Intl.NumberFormat(locale, {
    maximumFractionDigits: 0,
  }).format(value);
}
