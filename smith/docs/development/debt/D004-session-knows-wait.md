# D004: The session knows the run's `wait` tool

- **Kind:** coupling
- **Where:** `smith-domain-session/src/session.rs`, `smith-domain-session/src/session/restore.rs`, `smith-domain-session/src/domain.rs`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 03, finding 11)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's closing report (outside the repository): left as a change across siblings
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (the session stops depending on one opener's tool name; a second opener can withhold a call)
- **Size:** S
- **Status:** open
- **Paid when:** no code in `smith-domain-session` matches a tool name; the opener names the withheld call in its opening, and `Event::Answered` and `Event::AnsweredAndYield` are one event.

## What

A session knows a conversation and "nothing of runs, charters, results or
hosts" (`smith/docs/design/domain/session.md`, 1). Yet three places decide
behaviour on the run's tool name:

- `session.rs:1929`: `if waiting && name.as_ref() == b"wait"`;
- `session/restore.rs:218` and `:503`, restoring a transcript.

Beside it, `Event::Answered` and `Event::AnsweredAndYield` (`domain.rs:142`)
are twins that differ by what follows.

This is also a departure from session.md, which its status file does not
record (`smith/docs/design/domain/session.status.md` says "No gap").

## Why it matters

The session's contract says it serves any opener. A run that renames its
tool, or another opener that withholds a call, silently changes the
session's behaviour.

## What pays it back

The opener names the withheld call in its `Opening` (`record.rs`), and
the session matches that name, not `b"wait"`. Merge
the two answered events with one field for what follows. Touches the run's
opening and the session's restore, and their tests.
