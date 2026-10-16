type NetworkEndpoint = { host: string; port: number };

export const DEFAULT_DEBUG_PORT = 5555;

function parseUrl(value: string): URL | undefined {
  try {
    return new URL(value);
  } catch {
    return undefined;
  }
}

export function parseEndpointInput(
  input: string,
  defaultPort?: number,
): NetworkEndpoint | undefined {
  const value = input.trim();

  if (!value) {
    return undefined;
  }

  if (!value.startsWith("[") && value.split(":").length > 2) {
    return defaultPort === undefined ? undefined : { host: value, port: defaultPort };
  }

  const url = parseUrl(`adb://${value}`);

  if (!url?.hostname || url.username || url.password || url.pathname || url.search || url.hash) {
    return undefined;
  }

  if (url.port) {
    return { host: url.hostname, port: Number(url.port) };
  }

  return defaultPort === undefined ? undefined : { host: url.hostname, port: defaultPort };
}
