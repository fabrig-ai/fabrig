+++
id = "chat-title"
part = "chat-title"
kind = "feature"
needs = ["node-hint"]
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 75
+++

# A chat's heading is its first sentence, short

## Goal

Land the look request `chat-title`. A chat's heading shows its title's short form: the first sentence, or the first line where that is shorter, cut on a word with "…" past 72 characters. The whole title stays in the heading's name, and shows on hover where it is cut. The terminal's heading takes the same short form. The bar's crumbs yield to its controls.

Acceptance, by the tests below and by reading the rule. No browser runs, here or in any step:

1. `pages::chat::short_title` follows `temper/docs/design/client/README.md`, 3.5, *A chat's heading is short*, *its words* and *its length*, to the letter. One focused table test holds each case of Steps, 3.
2. A chat's level-1 heading is named "T9 · TITLE", the title whole, in both shells. The browser shows `short_title(TITLE)`, the terminal "T9 · " and it, so its first line is `== T9 · SHORT == …`.
3. Where the short form isn't the title, the heading's hint is the title, in both shells. A title shown whole has no hint.
4. A chat whose title is at its bound, `long_text` bytes with no sentence end, builds in both shells.
5. Only a chat's heading changes: the person's message, the task card, the Chats rows and an agent's page (`pages/agent.rs`) keep the title whole, and the stored title is untouched.
6. `#app > .temper-session .temper-crumbs` has `flex: 1 1 320px;`, in its rule where it stands, under `/* ==== ui: chats ==== */`. No other rule changes.
7. Every commit carries `Look-request: chat-title`. The last one deletes the `look-round-15` **Not yet** entry in `README.status.md` and carries the account (Steps, 5).

## Inputs

- `temper/docs/plan/look-round-15/notes/common.md`: the standing rules, the request and how to read it.
- `temper/docs/plan/look-round-15/notes/contracts.md`, both parts: `Builder::hint`, `short_title`, `Heading::Chat`, binding.
- `temper/docs/design/client/README.md`, 3.5: *A chat's bar* and *A chat's heading is short*.
- `temper/docs/design/client/journeys.md`, 9: the `chat-title` paragraph.
- `temper/docs/design/draft/look-requests.md`, 3 to 5: the landing, the trailers and the account.
- `skein/docs/design/ui.md`, 3.2, *a hint*; `skein/docs/design/ui-web.md`, 6, *a hint*.

## Owns and may change

Owns the header's files.

Departs from the implementer's text: `docs/design/` is otherwise closed, but the last commit deletes the entry naming `look-round-15` in `temper/docs/design/client/README.status.md`. If it can't close, it stays, and goes first under `## Left`.

Reader edits allowed:

- `temper/crates/temper-client-view/src/limits.rs`: only if acceptance 4 fails without it, one line raising the per-node `text` figure by the short form's bytes (72 characters of at most 4 bytes, the ellipsis and "T9 · "), with a comment in the file's manner naming the chat's heading. The survey reckons the bound already holds it: today's heading holds its name and the whole title, and the short form is far smaller than `long_text`.
- A test elsewhere in temper (`temper-client-shell-terminal`, `temper-client-tools`, `temper/tools/gallery`, `temper/tests/client`) that reads a chat's heading text and whose fixture title has more than its first sentence: the expected words alone. Name each in the report.
- A print or golden your change makes stale, by its own tool: `temper-client-journeys again` for a journey (none should be: no recorded journey's first prompt runs past its first sentence), the gallery's or terminal's goldens by their means. Name each.

Leave alone:

- `pages/agent.rs`, `pages/chats.rs` and every other place that shows a title: only a chat's heading is short.
- `words.rs`: `Word::ChatTitle` serves both of the heading's words as it is. A draft beside this one, `ui-wait-words/wait-words`, is changing `words.rs`, `pages/session.rs`'s `over` and a row of `tests/chat.rs`'s `every_chats_run_wait_says_why_above_the_composer_without_working_dots`; keep to the heading's lines of `session.rs` and to the two tests below in `tests/chat.rs`.
- `temper-client-shell-web/tests/suite/main.rs`: no stylesheet test for one declaration; the render check after landing reads it.
- skein: `node-hint` gave what this task needs.

Options, each decided:

- **A first line with a carriage return:** `\r` is white space, dropped at the short form's end, so "Fix it\r\nthen" is "Fix it".
- **Invalid UTF-8:** the title arrives validated; count a character at each byte that isn't a continuation byte (`b & 0xc0 != 0x80`) and never cut between a lead byte and its continuations, as `crate::choice::shown` does.
- **Where the terminal's numbered words come from:** `Word::ChatTitle` with `Hole::Name` of the short form, as `whole` is made with the title.

## Steps

### 1. The short form

In `temper/crates/temper-client-view/src/pages/chat.rs`, beside `main`:

```rust
/// A chat's heading is at most this many characters, its ellipsis included.
pub(crate) const SHORT_TITLE: usize = 72;

/// A chat's heading shows its title's short form (client/README.md, 3.5, A chat's heading is short).
pub(crate) fn short_title(title: &[u8]) -> Box<[u8]>
```

The crate's strict subset holds: no indexing, no unchecked arithmetic, no `as`; `get`, `checked_*`, `skein_lib::Writer` as `crate::choice::shown` uses them.

### 2. The heading

- `pages/session.rs`: `Heading::Chat` takes the contract's four fields. `top` names the heading `whole`, shows `short` in `Shell::Browser` and `numbered` in `Shell::Terminal`, and calls `b.hint(hint)` where there is one, in both shells.
- `pages/chat.rs`, `main`: make `short` by `short_title(task.chip.title.bytes())`; `whole` as today; `numbered` by `Word::ChatTitle` with the task's number and `short`; `hint` the title where `short` differs from it.

### 3. The tests

In `temper/crates/temper-client-view/src/tests/chat.rs`:

- **New, the one new test:** `a_chats_heading_is_its_first_sentence_or_line_cut_on_a_word`, directly after `the_session_heading_keeps_its_name_and_shows_the_title_in_the_browser`. A table of `(title, short)` over `crate::pages::chat::short_title`. Its rows, each with the literal it expects:
  - a short title shown whole: "Ship the update";
  - a sentence: the request's prompt, "okay let's design a small http server in python, using std lib. It should show a page with details about the moon cycle, where are we now in the cycle and when the next full moon will happen", is "okay let's design a small http server in python, using std lib.";
  - "?" and "!" end one: "Why does it fail? It ran yesterday." is "Why does it fail?";
  - a line shorter than its sentence: "Fix the build\nthen the docs. And more" is "Fix the build";
  - a carriage return: "Fix it\r\nthen" is "Fix it";
  - a `.` inside a word ends none: "Read 3.5 in lib.rs first. Then" is "Read 3.5 in lib.rs first.";
  - white space first: "  \n\tIndented first line\nsecond" is "Indented first line";
  - exactly 72 characters with no sentence end, shown whole: "Show the moon cycle on one page, with where we are now and the next moon";
  - past 72, cut on a word, with a space as its 72nd character: "Show the moon cycle on one page, with where we are now and the next one again soon" is "Show the moon cycle on one page, with where we are now and the next one…", 72 characters;
  - a cut just after a comma drops it: "Please write a small http server in python for the moon cycle page, using the standard library" is "Please write a small http server in python for the moon cycle page…";
  - a long word: 100 × "x" is 71 × "x" and "…";
  - characters, not bytes: 100 × "é" is 71 × "é" and "…", and 72 × "é" is whole;
  - nothing: "" is "".
- **Rows of what exists:** `the_session_heading_keeps_its_name_and_shows_the_title_in_the_browser` takes the title as a column, beside the shell's view, the shown words and the hint. Its rows, each in both shells (`fixtures::page_view`, the terminal's; `fixtures::browser_page_view`):
  - "Ship the update": named "T9 · Ship the update", showing "T9 · Ship the update" and "Ship the update", no hint (today's two rows);
  - the request's prompt: named "T9 · " and the whole prompt, showing "T9 · okay let's design a small http server in python, using std lib." and the same without "T9 · ", the hint the whole prompt;
  - a title at its bound, `long_text` bytes of words with no sentence end: it builds, and its hint is the title.

  Give the chat its title as `configuring_task` and `deliver_chat` do. Keep the test's name: it still says what the rows check.

Break each rule once by hand (the sentence end's following white space, the line feed, the 72-character window, the comma, the hint's condition, the terminal's numbered words), run the test, see it fail for that reason, restore, and name each in the report.

### 4. The stylesheet

In `temper/crates/temper-client-shell-web/temper.css`, the chats area's rule `#app > .temper-session .temper-crumbs { display: flex; … }` gains `flex: 1 1 320px;` after `min-width: 0;`. The request replaces no rule. Then, one command at a time:

- `heavy cargo nextest run -p temper-client-view --lib -E 'test(=NAME)'` for each of the two tests.
- `cargo fmt`, then `python3 -I -B temper/scripts/old_forms.py`.
- `heavy cargo clippy -p temper-client-view -p temper-client-shell-web --all-targets -- -D warnings`.
- `heavy cargo nextest run -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-tools -p temper-client-gallery -p temper-client-journeys -p temper-client-world`. A chat heading's words change meaning, so this set is also its readers'.
- `python3 -I -B tools/citations/citations.py --base origin/main`.

### 5. Commits and the account

- **Trailers:** `Look-request: chat-title` on every commit, one line above the other trailers.
- **The last commit** deletes the status entry and carries the account in its message, one table, `| Item | Result |`, as `temper/docs/design/draft/look-requests.md`, 5, a row each:
  - the overlay's rule, `#app > .temper-session .temper-crumbs { flex: 1 1 320px; }`: landed, in its rule where it stands;
  - page 01's heading text, the first sentence: landed, `pages/chat.rs`'s `short_title`;
  - page 01's `aria-label`, "T3 · " and the whole prompt: landed, the name as it was;
  - page 01's `title`, the whole prompt on hover: landed through skein's hint (`node-hint`); differs in that a title shown whole has none, as it would show nothing more;
  - the state of a first sentence past 72 characters: landed, cut on a word;
  - the state of a prompt with no sentence end: landed, its first line under the same limit;
  - the state of a short prompt: landed, whole, with no hint;
  - the narrow width: the same rule, which wraps below 320 px as the request says; nothing else changed;
  - the terminal's heading: landed, "TN · " and the short form, a heading still;
  - the person's message, the task card, the stored title and an agent's page: whole, unchanged;
  - prints: which journey, gallery or terminal print changed, or that none did;
  - a model-written title: not this request's; the request names it as a later request of the user's own;
  - the render check: it follows landing, owned by temper/ui, and compiling and print checks establish no rendered equality.
- Push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and:

- each rule broken and the failure seen;
- whether `limits.rs` changed, and the figure that made it;
- each reader file you edited beyond the header's, and each print regenerated;
- whether the status entry stands.

Name no browser run: none is to be made.
