import {
  backendKeyConstants,
  frontendSources,
  rustSources as readRustSources,
} from "./source-scan";

const PLACEHOLDER_PATTERN = /\{\{(\w+)\}\}/g;

const tsSources = frontendSources().map((source) => source.contents);
const rustSources = readRustSources().map((source) => source.contents);

export function placeholdersIn(template: string): Set<string> {
  return new Set(Array.from(template.matchAll(PLACEHOLDER_PATTERN), (match) => match[1]));
}

function rustFunctionBodies(contents: string): string[] {
  return contents.split(/\n\s*(?:pub(?:\([^)]*\))?\s+)?fn\s+/).slice(1);
}

function callBodies(contents: string, name: string): string[] {
  const bodies: string[] = [];

  for (const match of contents.matchAll(new RegExp(`\\b${name}\\(`, "g"))) {
    const start = (match.index ?? 0) + match[0].length;
    let depth = 1;
    let cursor = start;

    while (cursor < contents.length && depth > 0) {
      const char = contents[cursor];

      if (char === "(") {
        depth += 1;
      } else if (char === ")") {
        depth -= 1;
      }

      cursor += 1;
    }

    bodies.push(contents.slice(start, cursor - 1));
  }

  return bodies;
}

export type ParamSuppliers = {
  byKey: Map<string, Set<string>>;
  exactByKey: Map<string, Set<string>>;
  viaComputedKey: Set<string>;
};

export function suppliedParams(): ParamSuppliers {
  const byKey = new Map<string, Set<string>>();
  const exactByKey = new Map<string, Set<string>>();
  const viaComputedKey = new Set<string>();
  const constants = backendKeyConstants();

  const recordInto = (target: Map<string, Set<string>>, key: string, names: Iterable<string>) => {
    const existing = target.get(key) ?? new Set<string>();

    for (const name of names) {
      existing.add(name);
    }

    target.set(key, existing);
  };

  const paramNames = (body: string) =>
    Array.from(body.matchAll(/\(\s*"(\w+)"\s*,/g), (param) => param[1]);

  for (const contents of rustSources) {
    for (const call of callBodies(contents, "ui_text_with_params")) {
      const target = call.match(/^\s*(\w+)/);
      const key = target ? constants.get(target[1]) : undefined;

      if (key) {
        recordInto(exactByKey, key, paramNames(call));
        recordInto(byKey, key, paramNames(call));
      }
    }

    for (const body of rustFunctionBodies(contents)) {
      const names = paramNames(body);

      if (names.length === 0) {
        continue;
      }

      for (const reference of body.matchAll(/\b(\w+_KEY)\b/g)) {
        const key = constants.get(reference[1]);

        if (key) {
          recordInto(byKey, key, names);
        }
      }
    }
  }

  for (const contents of tsSources) {
    for (const call of contents.matchAll(/\bt\(\s*([^,()]*?)\s*,\s*\{([^}]*)\}/g)) {
      const names = Array.from(
        call[2].matchAll(/(?:^|,)\s*(\w+)\s*(?=[:,]|$)/g),
        (param) => param[1],
      );
      const literal = call[1].match(/^"([a-zA-Z][\w.]*\.[\w.]+)"$/);

      if (literal) {
        recordInto(exactByKey, literal[1], names);
        recordInto(byKey, literal[1], names);
      } else {
        for (const name of names) {
          viaComputedKey.add(name);
        }
      }
    }
  }

  return { byKey, exactByKey, viaComputedKey };
}
