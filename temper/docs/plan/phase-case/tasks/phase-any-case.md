+++
id = "phase-any-case"
part = "phase-case"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tools/src/jig/subject.rs",
  "temper/crates/temper-client-tools/src/tests/subject.rs",
  "temper/crates/temper-client-tools/src/jig/rows.rs",
  "temper/crates/temper-client-tools/src/tests/golden/help_task_show.txt",
  "temper/crates/temper-client-tools/src/tests/golden/help_groups.txt",
  "temper/docs/help/ctl.md",
  "temper/docs/help/reference/ctl-task.md",
]
effort = "medium"
minutes = 35
+++

# `--until phase:PHASE` parses its word in any case

## Goal

`temper ctl DIR task show TASK --until phase:Running` prints the generic usage line: only
the lower-case words parse, while `task show` prints the phase capitalised. Make
`subject::phase` match its words ignoring ASCII case (`Running`, `running`, `RUNNING`), and
`backing-off` also as `BackingOff` and as `--json`'s `backing_off`; say so in the help.

Acceptance:

- `subject::phase(w)` returns the same phase as for the lower-case word, for every word of
  `PHASES` in `src/tests/subject.rs` in lower, upper and capitalised case; `BackingOff`,
  `backingoff`, `backing_off` and `BACKING-OFF` give `Phase::BackingOff`.
- Words that are no phase still give `None`: `unknown`, `runnin`, `Backing off` (a space),
  `backing--off`, the empty word.
- `task show 41 --until phase:Running` and `--until phase:BackingOff` parse;
  `--until phase:Done` is refused with `Usage::TerminalPhase { phase: Phase::Done }`, as
  `phase:done` is.
- `phase_word` is unchanged: refusals and the table keep printing the lower-case word.
- `temper ctl help task show` and the `ctl` page name the accepted words (the lines below,
  verbatim); the help goldens and `reference/ctl-task.md` agree with `rows.rs`.
- `temper-client-tools` lib tests and `temper-docs` tests pass; clippy is clean.

## Inputs

- `temper/docs/plan/phase-case/notes/common.md`.
- `temper/docs/design/client/tools.md, 5.4`, the `phase:<phase>` bullet: the design names
  the words in lower case; this task widens what parses, not what is printed. Do not edit
  the design; the report says the design's bullet could say "in any case".
- What `task show` prints: its text line says the phrase-book word
  (`jig/crates/jig-client/src/words/english.rs`, `Self::Phase(..)`), its `--json` record
  `encode_phase` (`temper/crates/temper-client-tools/src/jig/render/document/parties.rs`).
  Several phrase-book words are no phase word (`Retrying`, `Due to run`, `Waiting for news`,
  `Completed`, ...): making those parse is out of scope (see Report).

## Owns and may change

Owns, in `temper/crates/temper-client-tools`:

- `src/jig/subject.rs`: the body of `pub fn phase(word: &[u8]) -> Option<Phase>`; its doc
  comment. The signature, `phase_word`, `parse`, `check`, `takes` and `live` stay as they
  are.
- `src/tests/subject.rs`: the new test and the new `UNTIL` rows below.
- `src/jig/rows.rs`: the `--until` description of `task show` (line 72) only.
- `src/tests/golden/help_task_show.txt` (the `--until` line) and `help_groups.txt` (the `ctl`
  page's bullet): regenerated, never hand-edited.

Departure from the implementer's text, stated: `docs/help/` is the lead's, and here the lead
gives its lines verbatim, so this branch changes them with the code they describe (the
help page, its reference page and `rows.rs` move together, so the trunk never says what
the code does not do). Change nothing else under `docs/help/`.

- `temper/docs/help/reference/ctl-task.md`: the `--until CONDITION` line under
  `## task show` becomes exactly the regenerated golden's line (below).
- `temper/docs/help/ctl.md`, the `task show TASK` bullet of the subjects list: the line
  `` `phase:PHASE`, a live phase; `` becomes these three lines (two-space indent, as now):

  ```
    `phase:PHASE`, a live phase in any case: `waiting`, `idle`, `due`,
    `preparing`, `claimed`, `running`, `backing-off` (or `BackingOff`),
    `active`, `closing` or `held`;
  ```

The new `rows.rs` description, verbatim:

```
Wait until it holds: ended, waiting-on-me, held, replied, phase:PHASE, or changed. PHASE is a live phase in any case: waiting, idle, due, preparing, claimed, running, backing-off (or BackingOff), active, closing or held
```

Pre-approved options:

- If a help test bounds a description's length and the line breaks it, shorten only the
  part after `changed.` to `PHASE is a live phase, in any case`, and say so in the report.
- If another golden or test (in this crate or `temper` CLI tests) holds the old `--until`
  line, regenerate or update it to the new line only, and list it in the report.
- If clippy objects to the match shape, any closure-free shape is yours (the tree bans
  `Option::map` and kin): a loop over the phases comparing with `eq_ignore_ascii_case`, or
  a lower-casing into a fixed stack buffer then the existing `match`.

## Steps

1. `subject::phase`: match a word ignoring ASCII case against each phase's lower-case word
   (`phase_word` is the list), and for `Phase::BackingOff` also against `backingoff` and
   `backing_off`. Doc comment, one line: the word in any ASCII case; `backing-off` also as
   `task show` prints it and as `--json` does. Sketch (the shape inside is yours):

   ```rust
   /// A phase's word in any ASCII case; `backing-off` also `BackingOff` and `--json`'s `backing_off`.
   #[must_use]
   pub fn phase(word: &[u8]) -> Option<Phase> { ... }
   ```

2. `src/tests/subject.rs`, beside `each_phase_word_names_one_of_jigs_phases_and_back`:

   ```rust
   /// `task show` prints a phase capitalised: that spelling parses as the lower-case word.
   #[test]
   fn the_printed_spelling_parses_as_the_lower_case_one() {
       for (word, phase, _) in PHASES {
           // the word upper-cased, and capitalised (first byte upper), each gives Some(*phase)
       }
       for word in ["BackingOff", "backingoff", "backing_off", "BACKING-OFF"] {
           assert_eq!(subject::phase(word.as_bytes()), Some(Phase::BackingOff), "{word}");
       }
       for word in ["", "unknown", "runnin", "Backing off", "backing--off"] {
           assert_eq!(subject::phase(word.as_bytes()), None, "{word}");
       }
       assert_eq!(
           subject::condition(&Subject::Task(41), b"phase:Done"),
           Err(Usage::TerminalPhase { phase: Phase::Done }),
       );
   }
   ```

   Build the upper-cased and capitalised words without `Vec` or `format!` (the tests'
   `clippy.toml`): a fixed `[u8; 16]` buffer and `make_ascii_uppercase`, or
   `skein_lib::Writer` as the test beside it does. Add to `UNTIL`:
   `(&["task", "show", "41", "--until", "phase:Running"], true)`,
   `(&["task", "show", "41", "--until", "phase:BackingOff"], true)`,
   `(&["task", "show", "41", "--until", "phase:Done"], false)`.

3. Help, in this order (`help_groups.txt` holds the `ctl` page, compiled in through
   temper-docs, so the pages change before the goldens are written): edit
   `temper/docs/help/ctl.md` as above; set the `rows.rs` line 72 description above; then
   regenerate the goldens with
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-tools --lib -E 'test(=tests::words::help_lists_groups_then_a_group_then_a_tool) | test(=tests::help::the_help_prints_as_its_golden_files)'`
   (check the test paths with `--list` first; a run that selects nothing proves nothing).
   `git diff --stat` must show only `help_task_show.txt` and `help_groups.txt` among the
   goldens, each changed only in the lines above; restore anything else the write touched
   with `git checkout --`. Last, set the `--until CONDITION` line of
   `reference/ctl-task.md` to the regenerated `help_task_show.txt` line, byte for byte.

4. Narrow tests: the new test and `a_condition_is_taken_only_where_its_subject_names_it`
   alone; then `cargo fmt`; `heavy cargo clippy -p temper-client-tools --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-client-tools --lib`; `heavy cargo nextest
   run -p temper-docs`. `phase()` widens what parses but no reader in another package calls
   it (`rg 'subject::phase'`); check that again, and if one appears, run its package's tests
   too.

5. Commit (one commit for the parse and its test, one for the help), push your branch.

## Report

As the implementer's text says, and also: the shape chosen for `phase()`; the golden files
regenerated; and, under `## Left`, that `task show` prints phrase-book words that still do
not parse as phases (`Retrying` for backing-off, `Due to run`, `Waiting for news`,
`Preparing a host`, `Claimed by a host`, `ChipExecutor active`), with
`jig/crates/jig-client/src/words/english.rs` as where they come from.
