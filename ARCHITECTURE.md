# Architecture

Litterbox uses a daemon-based architecture to manage container lifecycle and SSH key access.

## The Daemon

A long-running daemon process manages each Litterbox. The daemon:

- Is automatically started when you enter a Litterbox
- Handles SSH agent requests if keys are attached
- Monitors active sessions and automatically stops the container when all sessions close
- Runs alongside the container and persists until the container is stopped

The daemon uses two lockfiles in `~/Litterbox/`:

- `.daemon-{LBX_NAME}.lock` - Contains the daemon's PID (single process). Prevents multiple daemons from running simultaneously.
- `.session-{LBX_NAME}.lock` - Contains PIDs of all active terminal sessions (one per line). Used to track when all users have exited.

## Session Tracking

Each time you run `litterbox enter`:

1. If no daemon is running, one is started
2. Your terminal's PID is added to the session lockfile
3. When your terminal exits, your PID is removed from the session lockfile

The daemon periodically checks the session lockfile. When it becomes empty (or contains only stale PIDs for dead processes), the daemon stops the container and exits.
