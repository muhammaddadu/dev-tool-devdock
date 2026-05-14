import { cn } from "@/lib/cn";
import type { ServiceStatus } from "@/lib/types";

type Props = {
  status: ServiceStatus;
  className?: string;
};

const colors: Record<ServiceStatus, string> = {
  running: "bg-[hsl(var(--success))] shadow-[0_0_0_3px_hsl(var(--success)/0.18)]",
  starting: "bg-[hsl(var(--warning))] animate-pulse",
  stopped: "bg-muted-foreground/50",
  crashed: "bg-destructive",
  unknown: "bg-muted-foreground/40",
};

export function StatusDot({ status, className }: Props) {
  return (
    <span
      aria-label={status}
      className={cn("inline-block h-2 w-2 rounded-full", colors[status], className)}
    />
  );
}
