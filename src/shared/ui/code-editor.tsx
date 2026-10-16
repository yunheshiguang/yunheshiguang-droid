import { lazy, Suspense } from "react";

export type CodeLanguage = "shell" | "javascript" | "plain";

export type CodeEditorProps = {
  value: string;
  onChange: (value: string) => void;
  language: CodeLanguage;
  label: string;
  dark?: boolean;
  readOnly?: boolean;
  className?: string;
};

const CodeEditorView = lazy(() => import("./code-editor-view"));

export function CodeEditor(props: CodeEditorProps) {
  return (
    <Suspense
      fallback={<div className="min-h-64 animate-pulse rounded-md border border-input bg-muted" />}
    >
      <CodeEditorView {...props} />
    </Suspense>
  );
}
