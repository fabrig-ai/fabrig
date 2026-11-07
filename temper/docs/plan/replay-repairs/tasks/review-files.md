+++
id = "review-files"
part = "review-files"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/src/brief.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/brief.rs",
  "temper/tests/engine/forge/tests/suite/brief.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 50
+++

# A review of a small change to a large document starts

## Goal

A reviewer was held before it ran: `section=pull stage=files error=TooLarge`. A page of a pull's files carries each file whole on both sides (`skein/docs/design/forge.md`, 5.2), so a one-line change to a large document passes the forge client's answer bytes, and a review's pull section, its lone section, then fails its brief (`temper/docs/design/domain/forge.md`, 8.4). The design now says what a comparison past the limit already does applies to a page of files: the section keeps its identity, its content and the pages before, appends a files-omitted notice naming the page and the tools that read it, and goes on to its comparison. Build that in `temper-engine-domain-forge/src/brief.rs`.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 4).

Acceptance:

1. Failing first, in the forge world (`temper/tests/engine/forge/tests/suite/brief.rs`, below the real loop): a review (`change::State::Gating`) of a pull whose one changed file's two sides together pass the world's client `answer_bytes` (4,096, `temper/tests/engine/forge/src/world.rs`) gathers its section. The world sees `BriefReadRefused { section, stage: "files", error: Some(api::Error::TooLarge) }` and then `BriefSized { section, size: Ok(_) }`, within the section's bytes; the taken brief holds the pull's identity (`Pull request: #N`, its branch, base, head and title) and the notice of `notes/contracts.md` for page 1. Before the fix it sees `BriefSized { size: Err(.. "the answer is past the client's limit" ..) }`. Default shape: `a_reviews_lone_pull_read_failure_fails_its_brief_without_words` becomes a table of `(how the pull is made, the refused stage and error, the outcome)`, its `Missing` row as now and this one new, renamed for both (`a_reviews_lone_pull_read_failure_fails_or_keeps_its_section`).
2. Every pull section takes the same rule (review, repair, resolution, other): one `Files` arm, not one per mode. The engine's log still has its `read refused` line for the page (`refused(..)`, as the comparison's arm calls it).
3. A `Files` read that fails any other way counts by the section's table as now: the existing `Missing` row and the step table `each_unread_error_has_its_count_and_fixed_words` (`temper-engine-domain-forge/src/tests/brief.rs`) pass unchanged.
4. The entry *8.4: a page of a pull's files past the forge client's answer byte or row limit...* is gone from `temper/docs/design/domain/forge.status.md`.
5. Focused tests pass in `temper-engine-domain-forge` (`--lib`), `temper-engine-forge-world` and `temper-engine-domain-world` (`forge_routes::`), and the forge world's fuzzy sweeps: `heavy cargo nextest run --profile fuzzy -p temper-engine-forge-world`.

## Inputs

- `temper/docs/plan/replay-repairs/notes/common.md`.
- `temper/docs/plan/replay-repairs/notes/contracts.md`, *A files-omitted notice*: its words bind.
- `temper/docs/design/domain/forge.md`, 8.4: the comparison-omitted paragraph and the new one after it, and *A brief section's read that fails*.
- `skein/docs/design/forge.md`, 5.2 (a pull's files: paged inventory, each file whole on both sides).
- `temper/crates/temper-engine-domain-forge/src/brief.rs`: the `Compare` stage's `TooLarge` arm is the pattern.

## Owns and may change

**Owned:** the `files` above.

**Readers:** `temper/tests/engine/domain/tests/suite/forge_routes/reviews.rs` and `brief_failure.rs` read the review brief's words; run them (acceptance 5) and change an assertion only if it held that a `Files` `TooLarge` fails the brief, and say so.

**Not in this task:** naming the bound met and the size measured in a `TooLarge` refusal. `api::Error::TooLarge` carries neither, and carrying them changes skein's forge interface and temper's stored answer (`temper-protocol-store/schema/store-v11.schema`, `too_large`): the plan holds it as a question for the user. The notice names no figure.

**Options, each with its default:**

- After the notice the section goes on as after its last page: the existing `comparison` match follows, unchanged. If sharing that tail with the `PullFiles` arm needs a small private function, write it.
- `[more files omitted]` (the section full) stays as it is; the new notice is for a page refused.
- If a page after the first is refused (page 2 of a review's files), the notice names that page; a story row for it is not needed.

## Steps

1. Acceptance 1's row; run it and see the brief fail: `heavy cargo nextest run -p temper-engine-forge-world --test suite -E 'test(/^brief::/)'`.
2. The `Files` arm in `brief.rs`; the row passes. Break it once (drop the arm), see the row fail, restore.
3. The status entry, acceptance 5.
4. `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-forge-world --all-targets -- -D warnings`, commit and push.

## Report

As the implementer's text says; also how the row makes its oversized file.
