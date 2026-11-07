+++
id = "md-underscore"
part = "md-underscore"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui/src/markdown.rs",
  "skein/crates/skein-ui/src/tests/markdown.rs",
  "skein/docs/design/ui.status.md",
]
effort = "high"
minutes = 50
+++

# skein's Markdown reads `_` and `__` as emphasis and strong

## Goal

A draft written with underscores, `_Draft, with the user._`, shows its emphasis. skein-ui's Markdown reader takes `_` and `__` as `*` and `**` are taken, except within a word, as `skein/docs/design/ui.md`, 10, *emphasis and strong*, now says; and a delimiter closes only a run its own byte opened. This serves the look request `chat-terminal-draft`; temper's draft pane reads its first paragraph through this reader in a later task (`terminal-draft`).

Acceptance, by the tests below:

1. `_soft_` reads as `Run::Emphasis` "soft", `__strong__` as `Run::Strong` "strong", in `read` and in `read_growing`, and `_Draft, with the user._` as one emphasis run "Draft, with the user.".
2. An `_` or `__` with a word's character on both sides stays a character: `snake_case`, `read_branch … read_file`, `a__b`. A word's character is an ASCII letter or digit, or any byte at or above `0x80`. `*` within a word keeps today's reading: `un*frigging*believable` has its emphasis.
3. A delimiter closes only a run of its own byte: `*soft_` and `_soft*` are text as written; `*a _b_ c*` nests, emphasis inside emphasis being shown as today's reader shows a nested run.
4. A run still open while growing (`a _soft wo`) is emphasis to the line's end, as `a *soft wo` is; `word _` and `word __` alone at the end make no text yet, as `word *` makes none.
5. The reader's bounds (ui.md, 10, *bounded*) hold unchanged: the kept texts' tests pass as they are.
6. The `md-underscore` **Not yet** entry in `skein/docs/design/ui.status.md`, 10, is deleted.

## Inputs

- `temper/docs/plan/look-round-16/notes/common.md`: the standing rules. This task is skein's: of the request's rules, the trailer, the runs and the checks hold.
- `skein/docs/design/ui.md`, 10: *the subset*, *emphasis and strong*, *text still being written*, *a step machine*, *bounded*.
- `skein/docs/foundation/programming-model.md`, 10: the strict subset `markdown.rs` keeps.

## Owns and may change

Owns the header's files.

Departs from the implementer's text: `docs/design/` is otherwise closed, but the last commit deletes the `md-underscore` entry in `skein/docs/design/ui.status.md`. If it can't close, it stays, and goes first under `## Left`.

Reader edits allowed, each only for the expected words or a regenerated print, and each named in the report:

- A test elsewhere (`skein/crates/skein-ui-web`, `skein/crates/skein-ui-text`, `skein/tests/ui`, `jig/crates/jig-client-view`, `temper/crates/temper-client-view`, `temper/crates/temper-client-shell-terminal`, `temper/crates/temper-client-shell-web`, `temper/tools/gallery`) whose fixture holds an underscore pair outside a word and so now reads emphasis: the expected runs or words alone.
- A journey's print under `temper/docs/design/client/journeys/*/` that holds such a pair in a recorded turn or draft: reprinted by `temper-client-journeys` with no arguments, never by hand. The survey found none (`chat-draft` records `Draft, 2026-11-05, with the user.` with no underscores, and `read_branch`-like names are within words).
- A terminal golden of `temper-client-shell-terminal` that changes: regenerated as its test says (`SKEIN_CODEGEN_WRITE=1` with the owning test).

Options, each decided:

- **Where the word test lives:** a small private function beside `markers`, on the byte before and the byte after the delimiter run, `None` at the line's ends counting as no word's character.
- **`___x___`:** the reader pairs the longest delimiter it reads today: `__` then `_` as `**` then `*` are, whatever that gives; add no rule for three.
- **A delimiter of the other byte inside a run:** stays text where it pairs with nothing (`*a_b*` is emphasis "a_b", its `_` within a word).

Leave alone: `Run`, `Builder`, the web mapping and the text shell; they draw `Run::Emphasis` and `Run::Strong` already (`skein/crates/skein-ui-web/src/mapping.rs`, `text_run`; `skein/crates/skein-ui-text/src/render.rs`).

## Steps

### 1. The markers

In `skein/crates/skein-ui/src/markdown.rs`, `markers` reads `_` beside `*`: `Marker` gains the delimiter's byte, and a closing marker matches the open one only where both run and byte agree. An `_` or `__` whose neighbours are both a word's character is no marker. `inline` and the growing path need nothing else, since they follow the markers; check `read_growing`'s *a delimiter alone at the end* for `_` and `__` and extend it where it names `*` alone.

The strict subset holds: no indexing, no unchecked arithmetic, no `as`.

### 2. The tests

In `skein/crates/skein-ui/src/tests/markdown.rs`, rows of what exists, no new test:

- `paragraphs_blocks_runs_links_and_literal_fallbacks`: the literal table gains `snake_case and read_branch`, `a__b`, `*soft_` and `_soft*`, each read as written. Its input gains `_under_` and `__double__` beside `*soft*` and `**strong**`, and a check that both read as `Run::Emphasis` "under" and `Run::Strong` "double".
- `a_bold_still_open_is_bold`: rows `a __bold wo` (Strong, "bold wo") and `a _soft wo` (Emphasis, "soft wo").
- `a_delimiter_alone_at_the_end_makes_no_text`: rows `word __` and `word _`.
- `closed_blocks_read_as_read_reads_them`: rows `a __bold__`, `a _soft_`, `_Draft, with the user._`, `*a _b_ c*`, `un*frigging*believable` and `snake_case`; and `paragraphs_blocks_runs_links_and_literal_fallbacks` checks that `*a _b_ c*` reads "a ", "b" and " c" all as `Run::Emphasis`.

Break each rule once by hand (the word test, the byte match, the growing delimiter), run the test, see it fail for that reason, restore, and name each in the report.

### 3. The checks

One command at a time:

- `heavy cargo nextest run -p skein-ui --lib -E 'test(=NAME)'` for each test you changed.
- `cargo fmt`, then `python3 -I -B skein/scripts/old_forms.py`.
- `heavy cargo clippy -p skein-ui --all-targets -- -D warnings`.
- A change of meaning runs its readers' tests: `heavy cargo nextest run -p skein-ui -p skein-ui-web -p skein-ui-text -p skein-ui-tests -p jig-client-view -p temper-client-view -p temper-client-shell-terminal -p temper-client-shell-web` (`skein-ui-tests` is `skein/tests/ui`). Repair what the reader edits above allow, and name each.
- `python3 -I -B tools/citations/citations.py --base origin/main`.

### 4. Commits

- **Trailer:** `Look-request: chat-terminal-draft` on every commit, one line above the other trailers.
- The last commit deletes the status entry. Push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and:

- each rule broken and the failure seen;
- each reader test or print changed, and why;
- whether the status entry stands.
