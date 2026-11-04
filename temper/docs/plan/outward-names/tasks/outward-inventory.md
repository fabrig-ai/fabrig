+++
id = "outward-inventory"
part = "inventory"
kind = "docs"
needs = ["skein-names", "smith-names", "jig-names", "temper-names", "citations-names", "status-words"]
files = ["tests/boundary/outward.txt"]
effort = "medium"
minutes = 20
+++

# The outward inventory loses the files this plan cleaned

## Goal

`tests/boundary/outward.txt` lists each file that still names the lab and its count, which may
only fall (`tests/boundary/tests/outward.rs`). This plan's tasks cleaned the files of
`## Owns and may change`; this task drops their entries, or lowers an entry to the file's count
today, and touches no other entry. It is the inventory's only writer in this plan: parallel
removals of neighbouring lines conflict in git.

Acceptance:

- Each entry of the list below is gone when its file holds no name at your base, or holds the
  file's count today when it holds some (a recording kept under `smith-names`' option B, a name a
  task left), with a reason line directly above it (`## Owns and may change`).
- No other line of the file changes: the header, the `# Raised:` line and every entry not listed
  below (`tools/tiers/**`, `qa.md`, `v0.1.0-ladder.md`, `reference-walks.md`, the four drafts a
  later step owns) stay byte for byte, even where their counts could fall.
- `heavy cargo nextest run -p fabrig-boundary` passes; its count matches the base's (13 at this
  plan's base).

## Inputs

- `temper/docs/plan/outward-names/notes/common.md`: the standing rules.
- `temper/docs/plan/outward-names/inputs/request.md`, Bounds: each cleaned file's entry leaves
  the inventory, or is lowered with its reason.
- `tests/boundary/tests/outward.rs`: the inventory's form (`inventory`, `needed`, `verdict`), and
  the failure that prints "The inventory the tree needs".

## Owns and may change

- **`tests/boundary/outward.txt`**, these entries alone (this plan's files; four held no name
  already at this plan's base):
  - jig: `jig/docs/development/debt/reviews/2026-10-31-jig-quality-run.md`,
    `jig/scripts/live-sign-in.sh`, `jig/tests/client-browser/src/lib.rs`,
    `jig/tests/client-wire/tests/suite/placed_holds.rs`, `jig/tests/codec-families.rs`,
    `jig/tests/protocol-accounts/tests/suite/drift.rs`, `jig/tests/worker-wire/tests/suite/drift.rs`;
  - skein: `skein/crates/skein-llm/README.md`, `skein/docs/design/fake-forge.md`, the four
    `skein/docs/development/debt/reviews/2026-10-31-*.md`, `skein/tests/codec/tests/suite/diff.rs`,
    `skein/tests/codec/tests/suite/patch.rs`, `skein/tests/forge/conformance/README.md`,
    `skein/tests/forge/github/README.md`, `skein/tests/oauth/tests/suite/drift.rs`;
  - smith: `smith/benchmarks/README.md`, the seven `smith/benchmarks/tests/recorded/codex/`
    entries, `smith/docs/development/benchmarks/smith-codex-2026-10-14.md`, the two
    `smith/docs/development/debt/reviews/2026-10-31-quality*.md`;
  - temper: `temper/crates/temper/tests/suite/forge_land.rs`,
    `temper/crates/temper/tests/suite/startup.rs`, `temper/docs/design/client/browser.status.md`,
    `temper/docs/design/draft/archive/first-run.md`, `temper/docs/design/draft/github.md`,
    `temper/docs/design/draft/macos.md`, `temper/docs/design/draft/upgrades.md`,
    `temper/docs/design/playground.status.md`,
    `temper/docs/development/debt/D008-pipeline-names-in-code.md`,
    `temper/docs/development/playground-memory.md`, `temper/docs/development/pre-pr-numbers.md`,
    the three `temper/scripts/proof-live*.sh`, `temper/tests/client-browser/src/lib.rs`,
    `temper/tests/codecs/tests/suite/drift.rs`, the two
    `temper/tests/real-loop/tests/fixtures/{b0,b1}/regenerate.sh`, `temper/tools/twin/README.md`;
  - tools: `tools/citations/README.md`, `tools/citations/citations.py`.
- **A listed file that still holds names** keeps its entry at today's count:
  - `temper/docs/design/client/browser.status.md` or `temper/docs/design/playground.status.md`
    that `status-words` left at the base's wording (the playground-browser draft's rewrite had
    not landed): the entry as the base has it, no line added; it goes under `## Left`.
  - a `smith/benchmarks/tests/recorded/codex/` file kept under `smith-names`' option B, with one
    line directly above it:
    `# Kept: outward-names/smith-names, recorded Codex output; the home directory is the recording's own bytes, pinned by manifest.json.`
  - any other, with one line directly above it:
    `# Kept: outward-names/outward-inventory, <the task that owned it> left <n> names; see its report.`
    and the file, its lines and names go under `## Left` (they are that task's, not yours).
- **An entry missing at your base** (a parallel step dropped it) stays missing. **An entry not in
  the list** is never touched, whatever its count.
- **Departures from the implementer's text:** no Rust changes, so no `cargo fmt` or clippy;
  `tests/` is no tree's, so no old-forms run.

## Steps

1. Delete every listed entry from `tests/boundary/outward.txt`.
2. `heavy cargo nextest run -p fabrig-boundary --test outward -E
   'test(=no_file_names_the_lab_beyond_the_inventory)'`. Passing, go to 4. Failing, its output
   lists each file still holding names, with its lines, and "The inventory the tree needs".
3. For each listed file the failure names, add its entry back at the count it prints, in path
   order, with its reason line (`## Owns and may change`); a file it names that is not listed is
   not yours: stop and report it under `## Left` with its lines, leaving its entry as the base
   has it. Run step 2 again.
4. `git diff tests/boundary/outward.txt` shows only removed listed entries and any re-added ones
   with their reason lines. `heavy cargo nextest run -p fabrig-boundary` (all, count as
   Acceptance).
5. Commit; `python3 -I -B tools/citations/citations.py --all` (0 broken); push.

## Report

As the implementer's text says, and: the entries removed (count) and any kept, with their
counts and reason lines; step 2's outcome each time it ran; the full run's count; anything under
`## Left`.
