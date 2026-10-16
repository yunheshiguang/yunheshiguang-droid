import { describe, expect, it } from "vitest";

import { collectPhraseTags, filterPhrases, readPhrase } from "./phrase";
import { readPhraseDocuments, toPhraseDocumentBody } from "./phrase-documents";

const greeting = { id: "b", title: "欢迎语", body: "你好，欢迎光临！", tags: ["客服", "TikTok"] };
const followUp = { id: "a", title: "催付款", body: "Hi, your order is waiting.", tags: ["TikTok"] };

describe("phrases", () => {
  it("round-trips through a document and skips documents it cannot read", () => {
    const list = {
      collection: "phrases",
      documents: [
        { id: greeting.id, body: toPhraseDocumentBody(greeting) },
        { id: followUp.id, body: toPhraseDocumentBody(followUp) },
        { id: "broken", body: "{" },
        { id: "foreign", body: JSON.stringify({ name: "a flow" }) },
      ],
    };

    expect(readPhraseDocuments(list)).toEqual([followUp, greeting]);
  });

  it("keeps a phrase whose tags are missing or malformed", () => {
    expect(readPhrase("x", { title: "t", body: "b" })?.tags).toEqual([]);
    expect(readPhrase("x", { title: "t", body: "b", tags: ["ok", 3, ""] })?.tags).toEqual(["ok"]);
  });

  it("searches titles, bodies and tags, and narrows to a tag", () => {
    const phrases = [greeting, followUp];

    expect(filterPhrases(phrases, "欢迎", "all").map((phrase) => phrase.id)).toEqual(["b"]);
    expect(filterPhrases(phrases, "ORDER", "all").map((phrase) => phrase.id)).toEqual(["a"]);
    expect(filterPhrases(phrases, "", "客服").map((phrase) => phrase.id)).toEqual(["b"]);
    expect(collectPhraseTags(phrases)).toEqual(["TikTok", "客服"]);
  });
});
