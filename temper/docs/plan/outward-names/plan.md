# outward-names: plan

The lab's names leave fabrig's documents, scripts and tests (`inputs/request.md`, step 9 of 12):
every file `tests/boundary/outward.txt` lists, except those other steps own (`tools/tiers/**`,
`qa.md`, `v0.1.0-ladder.md`, `reference-walks.md`, the drafts `notes.md`,
`reference-case.execution.md`, `configuring.execution.md`, `dashboards.md`, anything under
`docs/plan/`).

## What the lead did in the first push

- Read every in-scope file's names with the check's own matcher (a throwaway replica of
  `outward.rs`'s `names`, deleted): 50 entries in scope; four already hold no name at this base
  (`temper/docs/design/draft/archive/first-run.md`, `macos.md`, `upgrades.md`,
  `tools/citations/citations.py`), `tools/citations/README.md` holds 1 of its 2.
- Reworded the two design files whose lines no other draft touches:
  `skein/docs/design/fake-forge.md` ~l.1217 and `temper/docs/design/draft/github.md` ~l.632 (the
  fixtures' scratch path → "a private directory outside any repository", "the person's private
  fixtures directory"). The retired-plans-out draft's `keep-decisions` changes other lines of
  both (nearest: fake-forge.md 1212-1213, three lines away).
- Wrote `notes/common.md` (standing rules, how a task checks its files clean without committing
  the inventory) and `notes/contracts.md` (the scripts' wrapper, `regenerate.sh DATA`, the
  wording every tree uses).

## Parts and tasks

| Task | Part | Needs | Files | Minutes |
| --- | --- | --- | --- | --- |
| `skein-names` | skein | - | 10 files: READMEs, test hints, 4 reviews | 30 |
| `smith-names` | smith | - | benchmark README, 7 recordings + manifest, report, 2 reviews, 2 doc comments | 50 |
| `jig-names` | jig | - | `live-sign-in.sh` (`JIG_WRAPPER`), 5 test hints, 1 review | 35 |
| `temper-names` | temper | - | 3 proof-live scripts (`TEMPER_WRAPPER`), 2 regenerators (`DATA` required), `forge_land.rs` stubs + 1 new test, hints, messages, twin README, 3 notes | 80 |
| `citations-names` | citations | - | `tools/citations/README.md`, one phrase | 10 |
| `status-words` | status | `temper-names` | `browser.status.md`, `playground.status.md`, only once the playground-browser rewrite is at its base | 10 |
| `outward-inventory` | inventory | all six | `tests/boundary/outward.txt` | 20 |

## Decisions

- **One writer for the inventory.** Each tree's entries sit next to another tree's; git
  conflicts on adjacent line removals (tried), so the five parallel tree tasks never commit
  `outward.txt`. Each checks its files clean by deleting its entries in the working tree, running
  the check, and restoring the inventory (`notes/common.md`). `outward-inventory` drops them all
  at the end, and keeps an entry at today's count, with a `# Kept:` line, only where a file
  still holds names.
- **The wrapper:** `TEMPER_WRAPPER` for temper's scripts, `JIG_WRAPPER` for jig's: one command
  from the environment, run before each command that went through the build tool before; unset,
  plain cargo (`notes/contracts.md`, 1). It has the shape of the tiers' wrapper, which a parallel
  step adds; each tree keeps its own variable, so jig names nothing of temper or of `tools/`.
  `proof-live.sh rung-b0-offline` stops refusing to run without the tool.
- **Behaviour holds:** `proof-live-github-managed.sh`'s two `/home/free/...` defaults become
  `$HOME/...` (the same paths for their owner, like `proof-live-github.sh`'s existing
  `$HOME/src/rust/tmp/...` default). The regenerators lose their default data path, as the
  request asks.
- **Recordings:** `smith-names` defaults to `/home/free/` → `/home/user/` (same length) in the
  seven Codex recordings and the manifest, with the manifest's `sha256` updated (a test checks
  it) and `source_sha256` kept; if a test cannot hold, it keeps them and the inventory gives the
  reason.
- **Design files:** the lead changed two (above); `status-words` changes the two status files
  under a stated departure; no other task touches `docs/design/`.
- Bare mentions the check does not count but the request names (`cluster.rs`'s "run them
  through heavy", `support/live.rs`'s comment, smith's two doc comments) are in their tree's
  task.

## Overlaps with other drafts

- **Could not avoid, line-level:** `temper/docs/design/client/browser.status.md` and
  `temper/docs/design/playground.status.md`: playground-browser's `browser-walk` and
  `status-nine` rewrite the entries that say "fab's gate list", and their versions still say it.
  `status-words` runs after `temper-names` and changes a file only if that rewrite is at its
  base; otherwise it leaves the file, the inventory keeps its entry (count 1), and the name
  waits for a later step or for the playground-browser draft to say "the gate's list".
- **Same file, different lines (no conflict expected):**
  `temper/crates/temper/tests/suite/startup.rs` (fabrig-scale's `init-standard`,
  `reference-trim`, `stock-scale`: their nearest hunks are ~l.1183 and ~l.1482; ours is ~l.1435);
  `temper/tests/client-browser/src/lib.rs` (playground-browser's `browser-walk`: ~l.20 and
  ~l.74; ours ~l.37); `skein/docs/design/fake-forge.md` and `temper/docs/design/draft/github.md`
  (retired-plans-out's `keep-decisions`, above).
- `tests/boundary/outward.txt`: the parallel tiers step lowers the `tools/tiers/` entries; ours
  are other lines (`tools/citations/citations.py` is the line above `tools/tiers/cpu-run`: a
  conflict there is one line, resolved by keeping both removals).

## Critical path

`temper-names` (80) → `status-words` (10) → `outward-inventory` (20): about 110 implementer
minutes. `smith-names` (50), `jig-names` (35), `skein-names` (30) and `citations-names` (10) run
beside `temper-names` from the first push.
