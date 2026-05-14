import { useState } from "react";
import { ChevronRight } from "lucide-react";
import type { ServiceView } from "@/lib/types";
import { ServiceCard } from "./ServiceCard";
import { cn } from "@/lib/cn";

type Props = {
  title: string;
  services: ServiceView[];
  defaultOpen?: boolean;
  compact?: boolean;
};

export function CollapsibleSection({
  title,
  services,
  defaultOpen = false,
  compact = false,
}: Props) {
  const [open, setOpen] = useState(defaultOpen);

  if (services.length === 0) return null;

  return (
    <section className="flex flex-col gap-1">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
        className={cn(
          "group flex items-center justify-between rounded-md px-1 pt-2 pb-1",
          "text-left hover:bg-muted/40 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
        )}
      >
        <div className="flex items-center gap-1">
          <ChevronRight
            className={cn(
              "h-3 w-3 text-muted-foreground transition-transform",
              open && "rotate-90",
            )}
          />
          <h2 className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
            {title}
          </h2>
        </div>
        <span className="text-[10px] tabular-nums text-muted-foreground">
          {services.length}
        </span>
      </button>
      {open && (
        <ul className={cn("flex flex-col gap-1", compact && "opacity-80")}>
          {services.map((s) => (
            <li key={s.id}>
              <ServiceCard service={s} compact={compact} />
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
