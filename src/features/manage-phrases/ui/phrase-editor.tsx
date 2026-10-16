import { type FormEvent, useId, useState } from "react";

import {
  MAX_PHRASE_BODY_CHARS,
  MAX_PHRASE_TITLE_CHARS,
  type Phrase,
  usePhraseActions,
} from "@/entities/phrase";
import { useI18n } from "@/shared/i18n";
import { errorMessage } from "@/shared/lib/error-message";
import { parseTagList } from "@/shared/lib/text";
import { Button, ConfirmButton, ErrorState, Input, Label, Textarea } from "@/shared/ui";

type PhraseEditorProps = {
  initial: Phrase;
  stored: boolean;
  onClose: () => void;
};

export function PhraseEditor({ initial, stored, onClose }: PhraseEditorProps) {
  const { t } = useI18n();
  const id = useId();
  const { save, remove } = usePhraseActions();
  const [title, setTitle] = useState(initial.title);
  const [body, setBody] = useState(initial.body);
  const [tags, setTags] = useState(initial.tags.join(", "));
  const [attempted, setAttempted] = useState(false);
  const [writeError, setWriteError] = useState<string | null>(null);
  const titleIssue = !title.trim()
    ? t("managePhrases.issues.titleRequired")
    : title.trim().length > MAX_PHRASE_TITLE_CHARS
      ? t("managePhrases.issues.titleTooLong", { max: String(MAX_PHRASE_TITLE_CHARS) })
      : null;
  const bodyIssue = !body.trim()
    ? t("managePhrases.issues.bodyRequired")
    : body.trim().length > MAX_PHRASE_BODY_CHARS
      ? t("managePhrases.issues.bodyTooLong", { max: String(MAX_PHRASE_BODY_CHARS) })
      : null;

  const write = (operation: Promise<void>) => {
    setWriteError(null);
    operation.then(onClose, (error: unknown) => setWriteError(errorMessage(error)));
  };

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setAttempted(true);

    if (!titleIssue && !bodyIssue) {
      write(
        save({ id: initial.id, title: title.trim(), body: body.trim(), tags: parseTagList(tags) }),
      );
    }
  };

  return (
    <form
      onSubmit={submit}
      aria-label={t("managePhrases.editor")}
      className="space-y-3 rounded-md border border-border bg-background p-4"
    >
      <div className="space-y-1">
        <Label htmlFor={`${id}-title`}>{t("managePhrases.fields.title")}</Label>
        <Input
          id={`${id}-title`}
          value={title}
          onChange={(event) => setTitle(event.currentTarget.value)}
          placeholder={t("managePhrases.fields.titlePlaceholder")}
          aria-invalid={(attempted && Boolean(titleIssue)) || undefined}
        />
        {attempted && titleIssue ? <p className="text-xs text-destructive">{titleIssue}</p> : null}
      </div>
      <div className="space-y-1">
        <Label htmlFor={`${id}-body`}>{t("managePhrases.fields.body")}</Label>
        <Textarea
          id={`${id}-body`}
          value={body}
          onChange={(event) => setBody(event.currentTarget.value)}
          placeholder={t("managePhrases.fields.bodyPlaceholder")}
          aria-invalid={(attempted && Boolean(bodyIssue)) || undefined}
        />
        {attempted && bodyIssue ? <p className="text-xs text-destructive">{bodyIssue}</p> : null}
      </div>
      <div className="space-y-1">
        <Label htmlFor={`${id}-tags`}>{t("managePhrases.fields.tags")}</Label>
        <Input
          id={`${id}-tags`}
          value={tags}
          onChange={(event) => setTags(event.currentTarget.value)}
          placeholder={t("managePhrases.fields.tagsPlaceholder")}
        />
      </div>
      <div className="flex flex-wrap gap-2">
        <Button type="submit">{t("managePhrases.save")}</Button>
        {stored ? (
          <ConfirmButton
            type="button"
            variant="outline"
            title={t("managePhrases.confirmDelete", { name: initial.title })}
            description={t("common.irreversible")}
            confirmLabel={t("managePhrases.delete")}
            cancelLabel={t("common.cancel")}
            onConfirm={() => write(remove(initial.id))}
          >
            {t("managePhrases.delete")}
          </ConfirmButton>
        ) : null}
        <Button type="button" variant="ghost" onClick={onClose}>
          {t("managePhrases.close")}
        </Button>
      </div>
      {writeError ? (
        <ErrorState title={t("managePhrases.writeFailed")} detail={writeError} />
      ) : null}
    </form>
  );
}
