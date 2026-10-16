import CodeMirror, { EditorView, type Extension } from "@uiw/react-codemirror";
import { useEffect, useMemo, useState } from "react";

import { cn } from "@/shared/lib/utils";

import type { CodeEditorProps, CodeLanguage } from "./code-editor";

const languageLoaders: Record<CodeLanguage, () => Promise<readonly Extension[]>> = {
  shell: async () => {
    const [{ StreamLanguage }, { shell }] = await Promise.all([
      import("@codemirror/language"),
      import("@codemirror/legacy-modes/mode/shell"),
    ]);
    return [StreamLanguage.define(shell)];
  },
  javascript: async () => [(await import("@codemirror/lang-javascript")).javascript()],
  plain: async () => [],
};

function useLanguageExtensions(language: CodeLanguage): readonly Extension[] {
  const [loaded, setLoaded] = useState<{
    language: CodeLanguage;
    extensions: readonly Extension[];
  }>();

  useEffect(() => {
    let current = true;

    void languageLoaders[language]().then((extensions) => {
      if (current) {
        setLoaded({ language, extensions });
      }
    });
    return () => {
      current = false;
    };
  }, [language]);

  return loaded?.language === language ? loaded.extensions : [];
}

export default function CodeEditorView({
  value,
  onChange,
  language,
  label,
  dark = false,
  readOnly = false,
  className,
}: CodeEditorProps) {
  const highlighting = useLanguageExtensions(language);
  const extensions = useMemo(
    () => [
      ...highlighting,
      EditorView.lineWrapping,
      EditorView.contentAttributes.of({ "aria-label": label }),
    ],
    [highlighting, label],
  );

  return (
    <CodeMirror
      value={value}
      onChange={onChange}
      extensions={extensions}
      theme={dark ? "dark" : "light"}
      readOnly={readOnly}
      minHeight="16rem"
      maxHeight="32rem"
      basicSetup={{ foldGutter: false, highlightActiveLine: !readOnly }}
      className={cn("overflow-hidden rounded-md border border-input text-sm", className)}
    />
  );
}
