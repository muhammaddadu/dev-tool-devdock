import { useEffect, useRef, type ReactNode } from "react";
import { cn } from "@/lib/cn";

type Props = {
  open: boolean;
  onClose: () => void;
  title?: string;
  children: ReactNode;
  className?: string;
};

/**
 * Minimal modal: backdrop dismiss, Escape dismiss, focus management. No
 * Radix dependency yet — when we need richer popovers, we'll layer it in.
 */
export function Dialog({ open, onClose, title, children, className }: Props) {
  const cardRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        onClose();
      }
    };
    document.addEventListener("keydown", onKey, { capture: true });
    return () =>
      document.removeEventListener("keydown", onKey, { capture: true } as EventListenerOptions);
  }, [open, onClose]);

  useEffect(() => {
    if (open) cardRef.current?.focus();
  }, [open]);

  if (!open) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      // `inset-2 rounded-2xl` matches the outer TrayPanel gutter + rounding,
      // so the modal backdrop respects the floating-popover shape instead of
      // painting to the rectangular window corners.
      className="fixed inset-2 z-50 flex items-center justify-center rounded-2xl bg-background/70 p-3 backdrop-blur-sm"
      onPointerDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        ref={cardRef}
        tabIndex={-1}
        className={cn(
          "w-full max-w-sm overflow-hidden rounded-lg border border-border bg-card shadow-xl",
          "focus-visible:outline-none",
          className,
        )}
      >
        {title && (
          <div className="border-b border-border px-4 py-3">
            <h2 className="text-sm font-semibold">{title}</h2>
          </div>
        )}
        <div className="px-4 py-3">{children}</div>
      </div>
    </div>
  );
}
