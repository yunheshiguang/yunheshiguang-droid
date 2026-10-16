import { FolderOpen } from "lucide-react";

import { cn } from "@/shared/lib/utils";

import { Button } from "./button";
import { Input } from "./input";

type PathInputProps = {
  value: string;
  onValueChange: (value: string) => void;
  onBrowse: () => Promise<string | null>;
  browseLabel: string;
  id?: string;
  placeholder?: string;
  label?: string;
  disabled?: boolean;
  className?: string;
};

export function PathInput({
  value,
  onValueChange,
  onBrowse,
  browseLabel,
  id,
  placeholder,
  label,
  disabled,
  className,
}: PathInputProps) {
  const browse = async () => {
    const picked = await onBrowse();

    if (picked) {
      onValueChange(picked);
    }
  };

  return (
    <div className={cn("flex min-w-0 gap-2", className)}>
      <Input
        id={id}
        aria-label={label}
        value={value}
        onChange={(event) => onValueChange(event.currentTarget.value)}
        placeholder={placeholder}
        disabled={disabled}
      />
      <Button
        type="button"
        variant="outline"
        className="shrink-0"
        disabled={disabled}
        onClick={() => void browse()}
      >
        <FolderOpen className="h-4 w-4" />
        {browseLabel}
      </Button>
    </div>
  );
}
