# replay-repairs: what the first real goal on a fresh box tripped over

The six faults of `inputs/request.md`, cut into seven tasks: one per item, but item 1, whose fix is jig's and whose story is temper's. Shapes that cross tasks are in `notes/contracts.md`; standing rules in `notes/common.md`.

## Parts and tasks

| Part | Task | Tree | What it makes true | Item |
|---|---|---|---|---|
| `walk-order` | `setting-order` | jig | `Setting` derives the store's key order (`Model` last), so no settings page continues "backward" | 1 |
| `walk-order` | `restart-story`, needs `setting-order` | temper | the engine's domain world restarts across a model's setting, one row a page; temper's readers of the new order follow it | 1 |
| `review-files` | `review-files` | temper | a page of a pull's files past the client's limit leaves a files-omitted notice, and a review starts | 2 |
| `turn-input` | `turn-whole` | jig | a turn read alone is decoded whole, so `task turn` pages every byte of a call's input | 3 |
| `check-refusal` | `check-named` | smith | a failed check's feedback names its check and the time it had, and a timed-out check's tail is pinned | 4 |
| `room` | `reference-room` (landed) | temper | `room.json` carries `root.journal.run_bytes` 65,536, its README's commands merge it, and it says what a charter must fit | 5 |
| `refusal-at` | `settings-at` (landed) | jig | an out-of-shape settings refusal's `at` is the failing item's position | 6 |

**Critical path:** `setting-order` (60 min), then `restart-story` (60 min). The other three run beside them from the start.

**Question for the user** (`plan.toml`, `too-large-measure`): naming the bound met and the size measured in a `TooLarge` refusal (item 2's last sentence) changes skein's forge interface and temper's stored answer; no task waits for it.

## What this cut decided and found

- **Item 1, the restart panic.** jig's `Setting` derives `Model` before `Application` and `ProjectPolicy`; the store's key writer tags it 6, after their 4 and 5. A startup settings page ending on an application's setting or a project's policy continues at a model's, which the domain reads as moving back, and `loads.rs` asserts. `store.md`, 4.1 requires the two orders to agree, so the fix is the enum's order, with no stored byte moved; the assertion stays (the design never lets a walk repeat). **What the cancel left:** the survey read the path a grace-0 cancel of a tree with a held, never-run reviewer takes: the tree's tasks closed deepest first (the held one at once, with no attempt), each one's live row erased, its ended record, ended indexes, result row and history written, its placed hold erased. None of that is a settings record, so the cancel doesn't move the settings walk. The likeliest reading is that this was the box's first restart after the reference's project policy (or an application setting) sat beside a model's setting. The incident's range and cursor pair are unknown, so this is the demonstrated defect, not a replay of the incident.
- **Item 2, the review's files.** Pull files are paged (one file a page under the reference's figures), and each page carries its files whole on both sides (`skein/docs/design/forge.md`, 5.2). A one-line change to a large document therefore passes `forge.client.answer_bytes` (255,570 with the reference's `limits.json`, 65,536 without it). Whether the box had `limits.json` merged is unknown. The design already keeps a pull section whose comparison is past the limit; `forge.md`, 8.4 now does the same for a page of files, so the review starts. Neither paging below a file nor sizing the bound from the room fits the design: a page is a whole file, and the bound is already sized from the room. The host limit of 63,740 in the tool's refusal is the agent's payload, not the bound that was met.
- **Item 3, the turn's input.** The engine and the wire carry the turn whole. The client's link decoded a turn read alone under the live turns' 32,768-byte `turn_text`, and the tools paged that cut copy. A turn read alone now has its own limits (`client.md`, 10).
- **Item 4, the silent timeout (read, not run).** `deliver` runs the repository's `.temper/pre-pr` under `check_timeout`, 60 s in temper init's `agent.json`. The script writes nothing before it ends, so a check killed at 60 s has nothing in its tail, and the empty tail is true. **The figure is `check_timeout`, left as it is.** The feedback now names the check and the time it had (`run.md`, 8.1).
- **Item 5:** `root.journal.run_bytes` goes in `room.json`, as the request says, in an `application` member merged as `limits.json` is (`configuration.md`, 13.9). Item 2 settles no new figure.
- **Item 6:** `at` counts from 0, as a batch's members do. temper prints it as given (`temper-settings-form`, "out of shape; item N"), while its settings file and its own translation refusal count items from 1. So the third item reads "item 2", not "item 0". Making the person's words count from 1 is temper's, and it is left out of this plan (below).

## Seven tasks, not six

The request drops from the bottom past six tasks. Items 5 and 6 went out first, as cutting rules ask of tasks that need no survey. The surveys then showed item 1 needs two trees: a branch stays in its tree, and the request wants the failing story in temper's world. Dropping a higher-ranked item to keep six would invert the ranking, so the plan holds seven small ones. **Dropped:** temper's wording of a settings refusal's position from 1 (`temper-settings-form`'s `refusal`, and `temper-service/src/clients/translation.rs` counting from 0 as jig does), a one-task follow-up for the next plan.

## Overlaps with the drafts beside this one

- `settings-at` changed `jig/crates/jig-domain-core/src/routing/settings/dispatch.rs`, which `model-catalogue-3/core-reads` changed on its trunk: the file the request names.
- `setting-order`'s readers may be settings tests that `model-catalogue-3` changed (`jig-domain-core/src/tests/settings/**`, `jig-protocol-store/src/tests/settings.rs`); its own files are `jig-protocol-store`'s `tests/intervals.rs`, `tests/keys.rs` and `tests/fixtures/keys.rs`, which no draft changes.
- `restart-story`'s `temper/tests/engine/domain/tests/suite/loads.rs` and `review-files`'s `temper-engine-domain-forge/src/brief.rs` lie under `limits-fuzzy/engine-sweeps`'s globs; `limits-fuzzy`'s trunk changes four lines of `brief.rs`.
- `turn-whole` changes `jig-client-protocol-link`'s `limits.rs`, `translate/turn.rs` and tests; `ui-wait-words/bounds-wait` changes other files of the same crate.
- The status files take one entry each, which each task removes; `model-catalogue-3/list-close` globs `**/*.status.md`.
