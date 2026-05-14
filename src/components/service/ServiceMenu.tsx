import { useEffect, useRef, useState } from "react";
import { MoreHorizontal } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/cn";

export type MenuAction = {
  label: string;
  onSelect: () => void | Promise<void>;
  destructive?: boolean;
  disabled?: boolean;
};

type Props = {
  actions: MenuAction[];
};

/**
 * Compact dropdown menu anchored to a "⋯" button. Closes on outside click,
 * Escape, or after the user picks an action. Renders nothing when there are
 * no actions to surface.
 */
export function ServiceMenu({ actions }: Props) {
  const [open, setOpen] = useState(false);
  const wrapperRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;

    const onPointerDown = (e: PointerEvent) => {
      if (!wrapperRef.current?.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };

    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  if (actions.length === 0) return null;

  return (
    <div ref={wrapperRef} className="relative inline-flex">
      <Button
        variant="ghost"
        size="icon"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label="More actions"
        onClick={() => setOpen((v) => !v)}
      >
        <MoreHorizontal className="h-3.5 w-3.5" />
      </Button>
      {open && (
        <div
          role="menu"
          className={cn(
            "absolute right-0 top-full z-50 mt-1 min-w-[180px] overflow-hidden",
            "rounded-md border border-border bg-popover py-1 shadow-md",
            "text-popover-foreground",
          )}
        >
          {actions.map((action) => (
            <button
              key={action.label}
              type="button"
              role="menuitem"
              disabled={action.disabled}
              className={cn(
                "block w-full px-2.5 py-1.5 text-left text-xs",
                "hover:bg-accent hover:text-accent-foreground",
                "focus-visible:bg-accent focus-visible:outline-none",
                action.destructive && "text-destructive hover:bg-destructive/10",
                action.disabled && "cursor-not-allowed opacity-50",
              )}
              onClick={async () => {
                setOpen(false);
                await action.onSelect();
              }}
            >
              {action.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
