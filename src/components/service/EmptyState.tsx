import { Inbox } from "lucide-react";
import { Button } from "@/components/ui/button";

type Props = {
  onAddCommand?: () => void;
};

/**
 * Full-height empty state shown only when no listening services are detected
 * at all (rare on a developer's machine). The "foreground is quiet" case
 * lives inline above the section list, not here — that case shouldn't hide
 * the collapsible Tooling/System sections.
 */
export function EmptyState({ onAddCommand }: Props) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 px-6 py-12 text-center">
      <div className="rounded-full bg-muted p-3">
        <Inbox className="h-5 w-5 text-muted-foreground" />
      </div>
      <div className="space-y-1">
        <p className="text-sm font-medium">No local services found</p>
        <p className="text-xs text-muted-foreground">
          Start a dev server in your terminal, or add a command manually.
        </p>
      </div>
      {onAddCommand && (
        <Button variant="default" size="sm" onClick={onAddCommand}>
          Add Command
        </Button>
      )}
    </div>
  );
}
