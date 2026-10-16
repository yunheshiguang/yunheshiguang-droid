import { MessageSquareText, Pencil, Plus } from "lucide-react";
import { useState } from "react";

import {
  collectPhraseTags,
  createPhrase,
  filterPhrases,
  type Phrase,
  usePhrases,
  usePhrasesError,
} from "@/entities/phrase";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { Badge, Button, ErrorState, Input, SectionCard, Select } from "@/shared/ui";

import { PhraseEditor } from "./phrase-editor";

export function PhraseLibrary() {
  const { t } = useI18n();
  const phrases = usePhrases();
  const loadError = usePhrasesError();
  const [query, setQuery] = useState("");
  const [tag, setTag] = useState<string>("all");
  const [draft, setDraft] = useState<Phrase | null>(null);
  const tags = collectPhraseTags(phrases);
  const activeTag = tags.includes(tag) ? tag : "all";
  const visible = filterPhrases(phrases, query, activeTag);

  return (
    <SectionCard
      icon={MessageSquareText}
      title={t("managePhrases.title")}
      contentClassName="space-y-4"
    >
      {loadError ? (
        <ErrorState title={t("common.documentsError")} detail={errorMessage(loadError)} />
      ) : null}
      <div className="grid gap-2 md:grid-cols-[1fr_12rem_auto]">
        <Input
          value={query}
          aria-label={t("managePhrases.search")}
          placeholder={t("managePhrases.search")}
          onChange={(event) => setQuery(event.currentTarget.value)}
        />
        <Select
          value={activeTag}
          aria-label={t("managePhrases.tag")}
          onChange={(event) => setTag(event.currentTarget.value)}
        >
          <option value="all">{t("managePhrases.tagAll")}</option>
          {tags.map((item) => (
            <option key={item} value={item}>
              {item}
            </option>
          ))}
        </Select>
        <Button type="button" onClick={() => setDraft(createPhrase())}>
          <Plus className="h-4 w-4" />
          {t("managePhrases.create")}
        </Button>
      </div>

      {draft ? (
        <PhraseEditor
          key={draft.id}
          initial={draft}
          stored={phrases.some((phrase) => phrase.id === draft.id)}
          onClose={() => setDraft(null)}
        />
      ) : null}

      {phrases.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("managePhrases.empty")}</p>
      ) : visible.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("managePhrases.noMatch")}</p>
      ) : (
        <ul aria-label={t("managePhrases.list")} className="grid gap-3 md:grid-cols-2">
          {visible.map((phrase) => (
            <li key={phrase.id} className="rounded-md border border-border bg-background p-3">
              <div className="flex items-start justify-between gap-2">
                <h3 className="min-w-0 truncate text-sm font-medium">{phrase.title}</h3>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  onClick={() => setDraft(phrase)}
                  aria-label={t("managePhrases.edit", { title: phrase.title })}
                >
                  <Pencil className="h-4 w-4" />
                </Button>
              </div>
              <p className="mt-1 line-clamp-3 whitespace-pre-wrap break-words text-xs leading-5 text-muted-foreground">
                {phrase.body}
              </p>
              {phrase.tags.length > 0 ? (
                <div className="mt-2 flex flex-wrap gap-1">
                  {phrase.tags.map((item) => (
                    <Badge key={item} variant="outline">
                      {item}
                    </Badge>
                  ))}
                </div>
              ) : null}
            </li>
          ))}
        </ul>
      )}
    </SectionCard>
  );
}
