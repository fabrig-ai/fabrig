# resume-fits-room: a resumed run's start always fits its worker link's room

`worker-link-quiet`, this plan's base, makes a start past the worker link's room (`io.output`) fail by name instead of silencing the link. Its design records why such a start happens. A run resumed after a loss carries every committed call. On both stock profiles, 32 committed calls at the largest answer pass 1 MiB alone, beside the charter, the transcript window and the inbox.

On a real deployment, long producers on a Codex program pass that room after an hour or so of work. From then on every resumption of them fails. Before `worker-link-quiet`, each one also cost the link and every other run on it. A single link loss resumes every run in flight, so the longest ones then can't come back at all. The evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **The failing story first,** at the lowest tier that holds the engine and a remote worker's link. A producer with more committed calls, at the largest answer the stock profiles admit, than one start can carry is resumed, and it starts and goes on. The same holds on both stock profiles.
2. **The fix,** chosen in the design: what a resumed start carries is bounded by the link's room, which the setting, the worker's sizes and the start share. Candidates:
   - the newest committed calls that fit, with the rest left to the transcript window the run can read;
   - calls that move in pieces after the start;
   - a smaller bound on what one call record keeps.

   Each must leave the program a consistent session. A configuration in which even the smallest resumable start can't fit is refused when it's set, naming the bound.
3. **The design says so:** `temper/docs/design/domain/worker.md` (4.3) and the session's resumption design, with their status files.

## Not in this plan

Any figure: the stock profiles' and examples' numbers stay as they are. Making runs survive a link loss is another milestone's work.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and no test reaches a real host. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
