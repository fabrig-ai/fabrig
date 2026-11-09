+++
id = "conflict-files-named"
part = "conflict-files"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/brief.rs", "temper/crates/temper-engine-domain-forge/src/tests/brief.rs", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 60
+++

## Goal

A resolution's brief names a conflicting file too large for it, with its
path and each side's bytes, for the resolver to read in its checkout,
rather than carrying it in part. Today a resolution's pull section carries
every file's two sides whole until the section's room runs out and cuts
the rest, so a large generated codec spends the brief half-carried
(`temper/docs/design/domain/forge.md`, 8.4, *A resolution*).

Acceptance:

- In a resolution's pull section (`PullMode::Conflict`), `brief.rs`'s
  `PullFiles` arm carries each file whole where all its bytes fit the room
  the section has left, and otherwise names it in the two lines
  `notes/contracts.md`, B, *A named conflicting file*, gives, byte for
  byte. A named file adds none of its contents; the files after it are
  carried as they fit. Review, repair and semantic sections carry files as
  now.
- One table test in `src/tests/brief.rs` holds it (Steps, 1), its rows
  written and seen failing first.
- The **Not yet** entry 8.4, *A resolution*, in the `Not yet:` list of
  `temper/docs/design/domain/forge.status.md` ("its pull section carries
  every file whole on both sides, …") is removed, its lines alone
  (`notes/common.md`, *Your status entry*).

## Inputs

- `temper/docs/design/domain/forge.md`, 8.4, the bullet *A resolution*:
  its last paragraph, the rule this task builds, and the paragraph before
  it on a files page past the client's limit, which stays.
- `temper/docs/plan/landing-behind-main/notes/contracts.md`, B.
- `temper/docs/plan/landing-behind-main/notes/common.md`.
- As they stand: `brief.rs`'s `PullFiles` arm in `brief_answer`,
  `append`, `BriefWords` and `decimal`; `src/tests/brief.rs`'s
  `pull_section` and `planned_read`, the fixture to follow.

## Owns and may change

- Owns the files in `files`; in `brief.rs`, the `PullFiles` arm and a
  helper it calls, alone.
- Pre-approved: if a reader test asserts a resolution's section carrying
  a file past its room (the engine domain world's
  `forge_routes/repair_cause.rs`, the forge world's `suite/brief.rs`),
  change that assertion to the named lines and name it in the report; its
  other steps stay.
- Not yours: the section's budget and every brief figure, the charter's
  fit (`temper-service`; the task `charter-worker-bound`), the files-page
  `TooLarge` notice, any mode but `Conflict`.

## Steps

1. **The table, failing first,** in `src/tests/brief.rs`: a fixture
   `resolution_section(files: Box<[skein_forge::api::File]>) -> Box<[u8]>`
   made from `pull_section`, its change `State::Resolving { requested:
   true, base: BASE }`, its section's budget 4,608, answering the item, the
   pull, one `PullFiles` page with `files` and `more: false`, then the
   comparison it asks (`Read::Compare { before: HEAD, after: BASE }`) with
   no files and no commits; and the test:

   ```rust
   #[test]
   fn a_resolutions_files_are_carried_whole_or_named() {
       // rows: (name, files, what the section holds)
   }
   ```

   | row | the page's files | the section holds |
   |---|---|---|
   | `small_files_whole` | `a`: 100 bytes before, 120 after; `b`: absent before, 50 after | both whole, `File: a\nBefore:\n…\nAfter:\n…\n` as now |
   | `a_large_file_named` | `codec`: 3,000 bytes before, 3,000 after | `File: codec\n[not carried, past the brief's room: 3000 bytes before, 3000 bytes after; read it in your checkout]\n`, and no byte of its contents |
   | `named_then_whole` | `codec` as above, then `a` as above | `codec` named, then `a` whole |
   | `an_added_large_file` | `codec`: absent before, 6,000 after | `…: absent before, 6000 bytes after; …` |

   Each row asserts the section's text after `Files and diff:\n` up to
   `What landed in the base:\n`. Run it: the last three rows fail today
   (the contents carried and cut); note it for the report.

2. **The rule** in the `PullFiles` arm, for `PullMode::Conflict` alone: a
   file's whole bytes measured against `fetch.words.bytes.room()` before
   any is appended; carried as now where they fit, else the two named
   lines, written with `append` and `decimal`. A comment cites
   `forge.md`, 8.4. Run the table: every row passes. Break the rule once by
   hand (measure the after side alone), see `an_added_large_file` or
   `a_large_file_named` fail for that reason, restore, and name it in the
   report.

3. **Narrow runs, then the readers:**
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/tests::brief::/)'`
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib`
   - `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/forge_routes::/)'`
   - `heavy cargo nextest run -p temper-engine-forge-world --test suite -E 'test(/^brief::/)'`
   Check each run's count; one that selected nothing proves nothing.

4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-engine-domain-forge --all-targets -- -D warnings`;
   remove the status entry; commit saying why (a resolution's large file is
   read in its checkout, not half-carried in its brief); push.

## Report

As the implementer's text has it, and: each row's failure before the
rule; the rule broken for the bite and the failure seen; each reader test
you edited and how; the counts of the runs in step 3.
