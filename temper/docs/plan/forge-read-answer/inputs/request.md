# forge-read-answer: an agent's forge read never takes down `serve`

live-temper's two stories, `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` and its Claude Code twin, have been red since the merge of `ui-qa-small` (`aaa951702a`, 11-06). In each, `temper serve` panics at `temper/crates/temper-engine-smith/src/answers.rs:1451`, `internal error: entered unreachable code: agent forge reads do not request Actions or trees`, just after an agent's `read_file` was refused twice: `read refused task=4 attempt=3 tool=read_file layer=forge-client reason=read admission busy error=Some(Busy)`. The deployment dies with it, and the story then waits out its patience, about 40 minutes, before failing at `temper/crates/temper/tests/live.rs:113` with "no surviving children". Its log is outside this repository, so the words here are what you have. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **The failing test first, below the live tier:** an agent's forge read, in `temper-engine-smith`'s or the engine domain's world, whose answer arrives as a kind the agent never asks for (`Runs`, `Run`, `Commits`, `Tree` or `Queued`), the way it reaches `answers.rs` after a read refused `Busy` and retried. Find the cause: how a forge answer of another request's kind reaches the agent's rendering, whether by a retry after `Busy`, a correlation of answer to request, or a read `ui-qa-small` added. **Read, not run:** that merge's diff, for what changed in forge reads or the forge client's admission.

2. **Fix the cause,** so the agent's read gets its own answer, and a `Busy` read is retried or refused as such.

3. **No answer takes down `serve`.** An answer of an unexpected kind becomes a refused read, with words that name the read and the kind, as the run's tool result, and a `Problem` for the log, not an `unreachable!`. The same for `api::Answer::Queued` beside it. A row in an existing test of `answers.rs`'s rendering.

4. **The live stories' fallout, read, not run:** the two stories are written in the live tier, which after-landing runs. Say whether item 1's test reproduces what they meet, and if any change they need beyond the fix, make it and compile it (`--no-run`).

## Not in this plan

Forge bounds and room figures, and how a too-large refusal is worded.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and a new story goes in a world below the real loop. Failing tests first; no test reaches a real host. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** The live stories are after-landing's to run. A stored format or protocol change is a question first; both main areas write `temper-engine-domain` and `forge_route.rs`. Formats change in place before v0.1.0. `walk = false`.
