import { useEffect, useState } from "react";

/**
 * Returns the current Unix-seconds timestamp, updated on an interval. Use as
 * a shared "tick" for relative time labels so every card re-renders together.
 */
export function useNowTick(intervalMs = 15_000): number {
  const [now, setNow] = useState(() => Math.floor(Date.now() / 1000));
  useEffect(() => {
    const id = window.setInterval(() => {
      setNow(Math.floor(Date.now() / 1000));
    }, intervalMs);
    return () => window.clearInterval(id);
  }, [intervalMs]);
  return now;
}
