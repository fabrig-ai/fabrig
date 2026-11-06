+++
id = "jig-spot"
part = "jig-spot"
kind = "feature"
needs = []
files = ["jig/docs/development/debt/D034-world-replay-cost.md", "jig/docs/development/debt/D035-bare-numbers-in-worlds.md", "jig/docs/development/debt/D030-*.md", "jig/testing/jig-conformance/src/core/peers.rs", "jig/testing/jig-core-kit/src/protocol.rs", "jig/tests/ops/src/protocol/calls.rs", "jig/tests/tasks/tests/suite/placed.rs", "jig/tests/people/tests/suite/simulation.rs"]
effort = "medium"
minutes = 40
+++

# jig's half of the debt-25 spot review: two sweeps, one signature, three items' words

## Goal

Findings 2, 4 and part of 6 of the spot review of debt-25, each checked against the trunk by the cut. No test's value or outcome changes; the code's lines fall.

Acceptance:

- `settled_answer` (`jig-conformance/src/core/peers.rs`) takes no `to` and returns the `core::SettledCall`; its four callers, in `jig-core-kit/src/protocol.rs` and `jig/tests/ops/src/protocol/calls.rs`, use the `to` they hold.
- `jig/tests/tasks/tests/suite/placed.rs` says `STOP_HOLD` wherever it says hold `100`: `rg -n '\b100\b' jig/tests/tasks/tests/suite/placed.rs` prints the `const` alone, or a `100` you can say isn't that hold. `jig/tests/people/tests/suite/simulation.rs` says `STOPPED_CHAT` for each `7` that is that chat. Hold `101` stays as it is.
- D035's **What** lists what remains, by reading, in place of "These copies span four files": see the steps. Its measure is re-read.
- D034's **What** holds the two sentences below.
- D030's `Ops` row reads `766` and `56`, with **Checked:** `2026-11-06`.
- `jig-tasks-world`'s and `jig-people-world`'s tests pass with their counts unchanged; `jig-core-world` and `jig-ops-world`, which read `settled_answer` through `jig-core-kit` and their own library, pass too.

## Inputs

- `jig/docs/development/debt/D034-world-replay-cost.md`, `D035-bare-numbers-in-worlds.md`, `D030-*.md`, each whole.
- `temper/docs/plan/debt-26/inputs/debt24-review.md`, second half, findings 2, 4 and 6.
- `temper/docs/plan/debt-26/notes/common.md`.

## Owns and may change

Owns the files in `files`. The sweeps are one-word edits, and leave the other bare numbers on those lines (`stop(&mut w, 1, 1, STOP_HOLD)`). A `const` the file already names is what the sweep reads: declare nothing new.

Options, each pre-approved:

- **A `7` or a `100` that isn't that identity** (another task that happens to share the number) stays, and the report names it.
- **`settled_answer` and its reply:** `temper`'s `hub::settled_answer` and `jig_domain_hub_link::settled_answer` are other functions. Leave them. `protocol.rs`'s first caller wraps with `ReplyTo::new(to)`, which now moves to where it builds the event; keep what the event carries.

## Steps

**D034,** at the end of **What**'s first paragraph or as its own, keeping its form:

> `Observer::inbound` also builds the full view for every core event (`jig/testing/jig-core-kit/src/conformance.rs`'s `inbound`, `jig/tests/ops/src/conformance/mod.rs`'s `inbound`). And `Neighbours::busy_events` keeps `{event:?}` strings (`jig/tests/conformance/src/lib.rs`).

**D035.** Verify each row below with `rg` first, then replace the sentence `These copies span four files; they do not fit a one-file correction.`, keeping the four bullets above it that still hold, with a list of what else remains. The cut's reading, after your sweeps:

- `jig/tests/accounts/tests/suite/finish.rs`: `start(1, 7, 1)`, five times.
- `HOLDER = 3` in `jig/tests/tasks/tests/lifecycle/grace.rs`, `tests/suite/circles.rs` and `tests/suite/kept_start.rs`.
- `PARTY` and `PROJECT = 1` in `jig/tests/conformance/src/scenarios/conversations.rs`.
- `stop(&mut w, 1, 1, ..)`'s task and party in `placed.rs`, and any other bare number your sweeps left.

Write `2026-11-06` in **Checked:**, and rerun the **Measure**, updating its count if it moved. The **Status** stays `open`.

**D030:** the table's `Ops` row becomes `| \`Ops\` | 766 | 375 | 105 | 19 | 11 | 56 |`. Recount it by the item's rule first (the cut got those numbers from `impl Application for Ops` in `jig/tests/ops/src/conformance/mod.rs`); write what you count, and if another row has moved, fix that too.

Tests: `heavy cargo nextest run -p PKG` for each of those four, and compare counts with your base. Then fmt, `python3 -I -B scripts/old_forms.py` from `jig/`, `heavy cargo clippy -p jig-conformance -p jig-core-kit --all-targets -- -D warnings` with each world you changed, and the debt and citations tools. Commit, push.

## Report

As your text says: lines removed from code, the sweep's counts, any number you left, and what you recounted.
