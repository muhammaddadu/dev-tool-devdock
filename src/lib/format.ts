export function tildeHome(path: string | null | undefined): string {
  if (!path) return "";
  // Best-effort. The real home path is unknown to the frontend; the backend will
  // pre-tilde when convenient. This is a safe fallback for display.
  return path.replace(/^\/Users\/[^/]+/, "~").replace(/^\/home\/[^/]+/, "~");
}

export function formatPort(port: number | null | undefined): string {
  if (port == null) return "";
  return `:${port}`;
}

/**
 * POSIX-safe single-quote for a shell argument. Wraps the input in single
 * quotes and escapes any single quotes inside via the standard `'\''` trick.
 * Works on bash, zsh, and sh. Used by Copy command so pasting into a terminal
 * actually runs.
 */
export function shellQuote(value: string): string {
  return `'${value.replace(/'/g, "'\\''")}'`;
}

/**
 * Build a paste-into-terminal command: `cd <quoted-cwd> && <command>`.
 * When `cwd` is missing, returns the command unchanged.
 */
export function buildPasteCommand(command: string, cwd: string | null | undefined): string {
  if (!cwd) return command;
  return `cd ${shellQuote(cwd)} && ${command}`;
}

/**
 * Compact "time since" formatter. Examples: `12s`, `4m`, `2h`, `3d`. Returns
 * an empty string when the timestamp is missing or in the future (clock skew).
 *
 * `nowUnix` is injectable so the ServiceCard can re-render on a shared tick
 * rather than each card calling `Date.now()` independently.
 */
export function formatAge(
  startedAtUnix: number | null | undefined,
  nowUnix: number = Math.floor(Date.now() / 1000),
): string {
  if (startedAtUnix == null) return "";
  const seconds = Math.max(0, nowUnix - startedAtUnix);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h`;
  const days = Math.floor(hours / 24);
  return `${days}d`;
}
