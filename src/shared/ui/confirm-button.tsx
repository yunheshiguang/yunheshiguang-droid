import { AlertDialog } from "radix-ui";
import type { ReactNode } from "react";

import { Button, type ButtonProps } from "./button";

type ConfirmButtonProps = Omit<ButtonProps, "onClick" | "asChild"> & {
  title: ReactNode;
  description?: ReactNode;
  confirmLabel: ReactNode;
  cancelLabel: ReactNode;
  onConfirm: () => void;
  ask?: boolean;
};

export function ConfirmButton({
  title,
  description,
  confirmLabel,
  cancelLabel,
  onConfirm,
  ask = true,
  children,
  ...props
}: ConfirmButtonProps) {
  if (!ask) {
    return (
      <Button {...props} onClick={onConfirm}>
        {children}
      </Button>
    );
  }

  return (
    <AlertDialog.Root>
      <AlertDialog.Trigger asChild>
        <Button {...props}>{children}</Button>
      </AlertDialog.Trigger>
      <AlertDialog.Portal>
        <AlertDialog.Overlay className="fixed inset-0 z-50 bg-black/40" />
        <AlertDialog.Content
          className="fixed left-1/2 top-1/2 z-50 grid w-[calc(100%-2rem)] max-w-md -translate-x-1/2 -translate-y-1/2 gap-4 rounded-lg border border-border bg-background p-5 text-foreground shadow-lg"
          {...(description ? {} : { "aria-describedby": undefined })}
        >
          <AlertDialog.Title className="text-base font-semibold">{title}</AlertDialog.Title>
          {description ? (
            <AlertDialog.Description className="text-sm text-muted-foreground">
              {description}
            </AlertDialog.Description>
          ) : null}
          <div className="flex flex-wrap justify-end gap-2">
            <AlertDialog.Cancel asChild>
              <Button type="button" variant="outline">
                {cancelLabel}
              </Button>
            </AlertDialog.Cancel>
            <AlertDialog.Action asChild>
              <Button type="button" variant="destructive" onClick={onConfirm}>
                {confirmLabel}
              </Button>
            </AlertDialog.Action>
          </div>
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
