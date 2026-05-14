import { Coffee } from "lucide-react";

type Props = {
  backgroundCount: number;
};

/**
 * Calm callout shown when the Dev bucket is empty but background services
 * exist. Bigger than a single-line strip so it reads as the intentional
 * resting state, not as a missed empty state.
 */
export function ForegroundQuiet({ backgroundCount }: Props) {
  return (
    <div className="flex items-start gap-3 rounded-lg border border-dashed border-border bg-muted/30 px-4 py-3.5">
      <div className="mt-0.5 rounded-full bg-muted p-1.5">
        <Coffee className="h-3.5 w-3.5 text-muted-foreground" />
      </div>
      <div className="min-w-0 flex-1 space-y-0.5">
        <p className="text-sm font-medium leading-snug">Nothing in the foreground</p>
        <p className="text-[11px] leading-snug text-muted-foreground">
          {backgroundCount} background service{backgroundCount === 1 ? "" : "s"} running below.
          Start a dev server and it'll show up here.
        </p>
      </div>
    </div>
  );
}
