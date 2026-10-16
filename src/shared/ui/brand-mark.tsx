import { brand } from "@/shared/config";
import { useI18n } from "@/shared/i18n";
import { cn } from "@/shared/lib/utils";

type BrandMarkProps = {
  className?: string;
  size?: "sm" | "md" | "lg";
};

const sizeClasses = {
  sm: "h-8 w-8",
  md: "h-10 w-10",
  lg: "h-12 w-12",
} as const;

export function BrandMark({ className, size = "md" }: BrandMarkProps) {
  const { t } = useI18n();

  return (
    <div
      className={cn(
        "flex shrink-0 items-center justify-center overflow-hidden rounded-md border border-border bg-background",
        sizeClasses[size],
        className,
      )}
    >
      <img src={brand.logoUrl} alt={t("app.name")} className="h-full w-full object-contain p-1.5" />
    </div>
  );
}
