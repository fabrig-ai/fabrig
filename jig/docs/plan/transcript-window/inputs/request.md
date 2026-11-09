# transcript-window: a long-running goal's transcript window always loads

On a real deployment at fabrig `69cff99edb`, a goal's coordinator was held again and again with `brief failed ... counts=transient reason=transcript: its window could not be loaded`. That held it as "out of tries", and a release failed the same way at once. Other goals on the same engine ran on. The same failure held five tasks for about a minute after each of two engine restarts, and then cleared. This one doesn't clear. The goal had run about an hour, with surveys and many tool calls, and had made several changes. Evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

The failure is raised in `jig/crates/jig-domain-core/src/routing/run/preparation.rs`, `transcript_loaded`, around lines 760-790. Either the store's load answers `TranscriptLoad::Failed`, or `core.append_transcript(task, row, text)` refuses a row of a page.

## What's wanted, ranked. Drop from the bottom if it passes three tasks, and say what you dropped

1. **The failing test first,** at the lowest tier that holds a run's preparation and the transcript store. Find which path fails for a long transcript. Candidates:
   - a row larger than the window's room;
   - a window whose rows or bytes pass `append_transcript`'s bound once the transcript grows;
   - a page boundary splitting a row;
   - the store's load refusing past a size.

   Rows of the existing preparation or transcript table: a transcript just inside the bound, just past it, and one with a single very large row (a large tool result).
2. **The fix:** a run's window is the transcript's tail that fits, as the design says the window is, never a failure for the transcript being long. A row too large for any window is cut with a marker, not refused. Say which bound refused, and why a restart's one-minute failure is the same or a different path.
3. **The design says so:** jig's design of the transcript window and a run's brief.

## Not in this plan

Any figure: the window's bounds stay as configured.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test; no test reaches a real host. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format change is a question first. jig never names temper. `walk = false`.
