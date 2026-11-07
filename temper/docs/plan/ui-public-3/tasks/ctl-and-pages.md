+++
id = "ctl-and-pages"
part = "says"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tools/src/jig/render/visibility.rs",
  "temper/crates/temper-client-tools/src/jig/rows.rs",
  "temper/crates/temper-client-tools/src/tests/jig/visibility.rs",
  "temper/crates/temper-client-tools/src/tests/golden/help_project_amend.txt",
  "temper/docs/help/reference/ctl-project.md",
  "temper/crates/temper-client-view/src/public/forge.rs",
  "temper/crates/temper-client-view/src/tests/public.rs",
  "temper/crates/temper-client-view/src/tests/public/rules.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/tests/suite/main.rs",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/client/tools.status.md",
]
effort = "high"
minutes = 45
+++

# ctl says what may show; a landing draws no zero; Landings reads as a link

## Goal

`temper ctl project amend --visibility public` is the one place a project goes public. It says what may show, in its help and with its answer. A landing's page stops ending on a bare "0 stale proofs", and the board's Landings link stops reading like a heading over an empty section.

Acceptance:

1. Made public, `project amend` prints the words on stderr, as text or as `--json`'s one JSON object a line, its stdout unchanged. Made private, or refused, it prints no such line. A key replay prints the same.
2. Its help, `temper ctl help project amend`, its golden and `temper/docs/help/reference/ctl-project.md` carry the words.
3. A landing with zero stale proofs draws no count, whether a queue merge's or a landing by turn. A landing with some still draws "2 stale proofs".
4. The board's "Landings" is still a level-2 heading holding a link to `/public/projects/<project>/landings`, and the sheet gives it the look in `notes/contracts.md`.
5. The two **Not yet** entries this closes are gone from `temper/docs/design/client/browser.status.md` (8.3) and `temper/docs/design/client/tools.status.md` (5.3, 6.2).

## Inputs

- `temper/docs/plan/ui-public-3/notes/contracts.md`: the words, ctl's table and `rows.rs`'s lines, the sheet's rule. They bind.
- `temper/docs/plan/ui-public-3/notes/common.md`.
- `temper/docs/design/client/tools.md`, 5.3 (the `project amend` row) and 6.2 (the last two bullets).
- `temper/docs/design/client/browser.md`, 8.3 (*The board*, *The landings*) and 8.4 (*Step tests*).
- `jig/docs/design/domain/people.md`, 13.3 (*A title can carry a chat's words*).

## Owns and may change

Owned: the `files` above.

Readers, surveyed, needing no edit: `temper/tests/one-process/src/public/service_checks.rs` reads the exit and prints stderr only on a failure; `temper-client-tools/src/tests/jig/replays.rs` compares a replay's stderr with the first's, and both go through `print`; `temper/tests/client-browser/tests/browser_all/browser_public.rs` reads stdout alone and never opens a landing's page. No other test draws "stale proofs" on a public page; ctl's own `landing show` line (`temper-client-tools/src/landing.rs`) is another surface and stays as it is.

Pre-approved:

- **If `the_help_prints_as_its_golden_files` or `the_project_reference_is_what_the_table_renders` is red,** regenerate with `SKEIN_CODEGEN_WRITE=1` and that test. A golden other than the amend row's that changes is a red, not stale.
- **If `Text` or the JSON encoder's bound refuses the words,** give `print` the bound they need: the existing `Limits { depth: 2, length: 128 }` holds the answer alone, and the words are 185 bytes.
- **If `unique_keys` in `tests/fixtures.rs` won't take your JSON line,** match the form the `kept:` lines print (`jig/render/outcome.rs`, `kept`), rather than changing the harness.
- **If an existing sheet test reads the old `h2` rule,** change it with the rule, and say so.

## Steps

### ctl

In `jig/render/visibility.rs`, beside `page`:

```rust
/// What making a project public says, word for word (jig's domain/people.md, 13.3).
const MAY_SHOW: &[u8] = b"Anything said in ... wrote too.";
```

The constant stays private: `render::visibility` is `pub(super)`, out of the tests' reach, so the tests hold the words in a literal of their own.

`print` builds its `stderr` from `identifier` and `json`: for `Some` it holds the words as in the contracts' table, for `None` nothing. Keep its `stdout` as it is. Change `PROJECT_AMEND`'s `prints` and `notes` to the note's lines, and regenerate the golden and the reference.

Tests in `tests/jig/visibility.rs`, with `const WORDS: &[u8]` holding the words:

- `made_public_prints_the_page_from_me`: `printed.stderr` is `WORDS` and a newline, where it asserted empty.
- `the_page_waits_for_me_to_agree`: for `--json`, stderr is exactly the `may_show` object and its newline; for text, `WORDS` and a newline. The golden check on stdout stays.
- `made_private_prints_no_page`: stderr empty, both forms.
- `a_refused_visibility_exits_2`: its stderr doesn't hold `WORDS`.
- New `the_help_says_what_the_answer_says`: `jig::rows::PROJECT_AMEND.notes` contains `WORDS`.

### A landing's zero

In `public/forge.rs`, `landing` draws its last paragraph only when `landing.stale_proofs() > 0`.

Tests in `tests/public/rules.rs` and `tests/public.rs`:

- `a_queue_merges_line_and_landing_read_as_a_landing`: `!lines.contains(&"0 stale proofs".into())`, where it asserted the opposite.
- `the_landing_pages_show_the_turn_queues_commits_and_proofs`: `only_words` allows "2 stale proofs" without requiring it, so add `assert!(has_text(&page, b"2 stale proofs"))`.

### The link

In `temper.css`, replace `#app > .temper-public-page h2 { ... }` where it stands by the three rules in the note. It is the only `h2` the public pages draw (`public/pages.rs`).

Tests:

- `temper-client-shell-web/tests/suite/main.rs`, new `the_boards_landings_link_reads_as_a_link`, in the form of `draft_section_headings_keep_the_document_line_height`: the `h2` rule holds `color: var(--gf-ink)` and no `uppercase`; the `h2 > a` rule holds `text-decoration: underline`.
- `tests/public/rules.rs`, `landings_show_below_the_goals_only_with_a_queue_or_a_landing`: where the section shows, `find(page.tree(), Role::Link, b"Landings")`'s `crate::public::href` is `/public/projects/3f2a9c0176543210/landings`, as the queue merge's test reads a link. The `page` becomes `let mut`.

### Close

Drop the two **Not yet** entries. Then narrow tests, `cargo fmt`, clippy on the three packages, and the focused tests of the three packages and their direct dependents, as your text says. Commit and push.

No browser story changes, and none runs. Stale `Prints:` words in another page are `chat-and-help`'s; leave `temper/docs/help/project.md` and `open-public-reading.md` to it.

## Report

As your text says. Name each rule you broke by hand and what failed. Say once more, for the plan's report: nothing stands between the command and the act, so ctl's words come in its help and after it.
