export function DetailRow({ label, value }: { label: string; value: string }) {
  if (!value) {
    return null;
  }

  return (
    <div className="grid gap-1 md:grid-cols-[10rem_minmax(0,1fr)]">
      <div className="text-xs font-medium uppercase text-muted-foreground">{label}</div>
      <div className="min-w-0 truncate text-sm">{value}</div>
    </div>
  );
}
