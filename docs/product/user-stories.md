# User Stories

Each story drives at least one user-visible behavior. Stories are written
in the "As a developer, I want X so that Y" form.

1. **Discover what is on a port.** As a developer, I want to open the menu
   bar and immediately see what is listening on each localhost port, with
   PID, command, and cwd, so I do not have to run `lsof` by hand.

2. **Save a detected service.** As a developer, when I find a service I
   started by hand, I want to save it with a label and remembered command
   so I can run it again later without typing the command.

3. **Run a saved service.** As a developer, returning the next day, I want
   to click a pinned service and have it start in the right directory with
   the right command, with logs captured.

4. **Stop a managed service cleanly.** As a developer, I want to stop a
   service I started from DevDock and trust it shuts down its full process
   group, not just the parent PID.

5. **Restart a managed service.** As a developer, after editing config or
   `.env`, I want to restart a service in one click without retyping the
   command.

6. **Kill an external process safely.** As a developer, when a port is
   stuck on a process I did not start through DevDock, I want a clearly
   destructive confirmation showing PID, command, and cwd before anything
   is killed.

7. **Scan a project.** As a developer, when I open a project directory, I
   want DevDock to scan its manifests and propose runnable services so I
   do not have to wire them up by hand.

8. **Switch projects.** As a developer working across several repos, I
   want to group pinned services by project and switch between them
   without losing context.

9. **See recently seen services.** As a developer, I want a "Recently
   Seen" list that shows things that were running today even if I did not
   save them, so I can recover after a terminal closes.

10. **Tail logs for a launched service.** As a developer, I want to see
    stdout and stderr for any service DevDock launched, scrolled to the
    latest line, with a way to open the full file.

11. **Toggle theme.** As a developer, I want DevDock to follow my system
    theme by default and let me override it without restart.

12. **Get optional AI hints.** As a developer who has Claude or Ollama
    installed, I want to opt in to letting DevDock suggest commands for an
    unfamiliar project, reviewed before anything runs.
