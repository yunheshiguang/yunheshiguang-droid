import { Allow, parse } from "partial-json";

function readPartial(text: string): unknown {
  try {
    return parse(text, Allow.ALL);
  } catch {
    return undefined;
  }
}

export function parsePartialDraft(text: string): unknown {
  const start = text.indexOf("{");

  if (start < 0) {
    return undefined;
  }

  const object = text.slice(start);

  return readPartial(object) ?? readPartial(object.slice(0, object.lastIndexOf("}") + 1));
}
