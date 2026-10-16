import { backendKeyConstants, frontendSources } from "./source-scan";

const sources = frontendSources();

export function literalMessageKeys(): { key: string; file: string }[] {
  return sources.flatMap(({ file, contents }) =>
    Array.from(contents.matchAll(/\bt\(\s*"([a-zA-Z][\w.]*\.[\w.]+)"/g), (match) => ({
      key: match[1],
      file,
    })),
  );
}

export function dynamicKeyPrefixes(): string[] {
  const prefixes = sources.flatMap(({ contents }) =>
    Array.from(contents.matchAll(/`([a-zA-Z][\w.]*\.)\$\{/g), (match) => match[1]),
  );

  return Array.from(new Set(prefixes));
}

export function referencedKeyLiterals(): Set<string> {
  const referenced = new Set<string>();

  for (const { contents } of sources) {
    for (const match of contents.matchAll(/"([a-zA-Z][\w]*(?:\.[\w]+)+)"/g)) {
      referenced.add(match[1]);
    }
  }

  return referenced;
}

export function backendMessageKeys(): Set<string> {
  return new Set(backendKeyConstants().values());
}
