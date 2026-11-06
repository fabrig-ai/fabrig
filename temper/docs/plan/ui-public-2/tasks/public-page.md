+++
id = "public-page"
part = "public-page"
kind = "feature"
needs = ["public-records"]
files = [
  "temper/crates/temper-client-view/src/public.rs",
  "temper/crates/temper-client-view/src/public/**",
  "temper/crates/temper-client-view/src/tests/public.rs",
  "temper/crates/temper-client-view/src/tests/public/**",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-protocol-wire/schema/public-v1.schema",
  "temper/crates/temper-client-protocol-wire/schema/public-v1.fields",
  "temper/crates/temper-client-protocol-wire/src/generated/public_v1.rs",
  "temper/crates/temper-client-protocol-wire/golden/public/**",
  "temper/crates/temper-client-protocol-wire/src/public.rs",
  "temper/crates/temper-client-protocol-wire/src/tests/public.rs",
  "temper/crates/temper-client-protocol-wire/src/tests/public/**",
  "temper/crates/temper-engine-domain-forge/src/public.rs",
  "temper/crates/temper-engine-domain-forge/src/public/**",
  "temper/crates/temper-engine-domain-forge/src/tests/public.rs",
  "temper/tests/codecs/tests/suite/fuzzy/public_wire.rs",
  "temper/tests/engine/forge/tests/suite/public.rs",
  "temper/tests/engine/forge/src/public.rs",
  "temper/tests/client-browser/tests/browser_all/browser_public.rs",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/client/forge.status.md",
]
effort = "high"
minutes = 100
+++

# A stranger reads a plain column, in sense, with no link to a forge

## Goal

The public pages become one plain column of rows in the client's look, with nothing overlapping from 320 to 1440 px. An ended goal says how it ended, a commit is seven plain digits, each empty page says so, and no page links to a forge. The public records stop carrying forge addresses.

Acceptance:

- Every page and word in `temper/docs/design/client/browser.md`, 8.3 holds, with the words in `temper/docs/plan/ui-public-2/notes/contracts.md`, *What a page shows*.
- temper's `public-v1.schema` is as the contracts note has it, with its generated code, goldens and `public-v1.fields` in agreement. A test holds that the schema has no string field at all.
- A commit held as 20 bytes and 12 zero bytes prints seven digits and no padding, on the change page and the landing page, as plain text. A focused test holds it.
- No node on any public page has an external address, except the banner's "Sign in".
- `temper/tests/client-browser/tests/browser_all/browser_public.rs` matches the new pages and compiles. It isn't run.
- A throwaway render shows a board of live, landed and failed goals with nothing overlapping or overflowing at 320, 360, 420, 600, 760, 1024 and 1440 px. Your report has the measurements and the paths of screenshots at 1440 and 420 (*Render proof*, below).
- The **Not yet** line naming 8.3 and 8.4 in `temper/docs/design/client/browser.status.md` and the one naming 11.1, 11.3 and 11.5 in `temper/docs/design/client/forge.status.md` are gone. The other new line in `browser.status.md`, *three things remain*, stays.

## Inputs

- `temper/docs/design/client/browser.md`, 8.3 and 8.4.
- `temper/docs/design/client/forge.md`, 11, 11.1, 11.3 and 11.5.
- `temper/docs/plan/ui-public-2/notes/contracts.md`, binding.
- `temper/docs/plan/ui-public-2/notes/common.md`.
- The model for a row: `temper/crates/temper-client-view/src/pages/chats.rs`, `row`, and `/* ==== ui: chats ==== */` in `temper.css`. Take its spacing and rules, not its monospace.
- `temper/docs/design/client/greenfield/mock.css`: `.col`, `.headline`, `.lede`, `.section-title`, `.empty`. The mock has no public page, so this is its type, not a layout to copy.

## Owns and may change

Owns the files in the header.

Readers and neighbours:

- **`temper/crates/temper-client-domain/src/tests/public.rs`, `temper/crates/temper-engine-domain/src/tests/public.rs`, and any other reader of the four changed records:** the removed fields' lines alone.
- **`temper/crates/temper-client-protocol-wire/src/lib.rs`:** `ADDRESS_BYTES`, if nothing reads it any more. If panels still read it, it stays.

Options, each decided:

- **Regenerating:** `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`. Delete any golden it leaves behind. `public-v1.fields` is compared by `temper-client-protocol-wire/src/tests/public.rs`: if it has no writer, edit it to match.
- **The fuzzy test's pinned lengths** in `public_wire.rs` follow the new shapes, and its `address()` helper goes.
- **`addresses_only`** in `temper-client-protocol-wire/src/tests/public/shape.rs` becomes a test that no record holds a `bytes` or `text` field. Keep what it checks of the rest.
- **`repository_address` stays:** `views/changes.rs` reads it, and `public/landings.rs` still orders and groups its internal `Queue` by it. Only `public_pull` and the wire's fields go. `maximal_proofs`, and any test that existed to fill a 2048-byte address, shrink to what still bites.
- **`Word::PublicCancelled` (3021)** is renamed `Word::PublicLanded`, saying "landed": `cancelled` now reads `PublicEnded`. `Word::PublicChange` (3013) keeps its id and becomes `(&[b"Its change: ", b""], &[Name])`. New words take the next free ids.
- **Reuse:** `Word::RowSpent`, `Word::ToolSeparator` (" · "), `TimeForm::Span` and `TimeForm::Ago`.
- **A title's row with a long unbroken title:** `overflow-wrap: anywhere` on the row's line.
- **`look::PLAN`, `PLAN_ROW` and `CHANGE`:** the public pages stop using them. Private pages keep them, so leave their rules alone.

Not this task: titles (`notes/common.md`), `temper-client-domain`'s reader, the lineage's words, priority's ordering, `clients.public`'s figures, the private pages, the gallery, and `temper-client-tools/src/landing.rs`, whose `hex` prints 64 digits to the terminal. That one is a separate defect with no link, and the plan's report names it.

## Steps

### 1. The records

Change temper's schema per the contracts note and regenerate. Then `temper-client-protocol-wire/src/public.rs`'s limits, `temper-engine-domain-forge/src/public.rs` and `public/{change,landing,landings}.rs`, and their tests. Remove the assertions on addresses: `temper/tests/engine/forge/tests/suite/public.rs`, 55 asserts a run's address, and becomes an assertion on the failed proof's state.

### 2. The words

In `words.rs`'s public block:

```rust
Word::PublicBoardLede => (&[b"What temper is doing on this project, as it happens. Read-only."], &[]),
Word::PublicNoGoals => (&[b"No goals yet."], &[]),
Word::PublicNothingLanded => (&[b"Nothing has landed yet."], &[]),
Word::PublicNoProjects => (&[b"No public projects yet."], &[]),
Word::PublicLanded => (&[b"landed"], &[]),
Word::PublicChange => (&[b"Its change: ", b""], &[Name]),
```

### 3. The view

New tokens in `look.rs`, named in its `NAMES` and in `tests/look.rs`: `PUBLIC_PAGE` (`temper-public-page`), `PUBLIC_INTRO`, `PUBLIC_LEDE`, `PUBLIC_ROWS`, `PUBLIC_ROW`, `PUBLIC_LINE`, `PUBLIC_FACTS`, `PUBLIC_SECTION`. `build_public` puts `PUBLIC_PAGE` on the main beside `MAIN` and `PAGE`.

In `public/lines.rs`, one way to draw a row, for the board, the task page and the delegates:

```rust
pub(super) fn row(&mut self, b: &mut Builder, line: &PublicLine, executor: bool);
```

```
Item PUBLIC_ROW
  Paragraph PUBLIC_LINE    link "Goal 7169df3c", a space, title, "…" where cut
  Paragraph PUBLIC_FACTS   [an agent ·] running · for 14 minutes [· 126 spent]
  (the goal's tree) List PUBLIC_ROWS with Item PUBLIC_ROW below it, then "2 lines folded" as PUBLIC_FACTS
```

- The space and every " · " are text in the page, not layout: a line's words must never touch, which is how "its change on the forge789ada..." came about.
- The phase's word depends on `line.goal()`, per the contracts note.
- A live row's time is `changed` in `TimeForm::Span` with `Word::PublicFor`. An ended row's is `ended` (or `changed` if it has none) in `TimeForm::Ago`.
- Priority isn't drawn. The executor is drawn on the task page's own row alone.

In `public/pages.rs`:

- **Board:** `PUBLIC_INTRO` with the h1 and the lede, then the rows or `PublicNoGoals`, then folded and as-of. Below them, only if `landings` holds a queue or a landing, a `PUBLIC_SECTION` with an h2 holding the "Landings" link, the queues, and `PublicNothingLanded` if it has no landing. The top-of-page "Landings" link goes.
- **Task:** as 8.3 has it, its forge lines drawn with `ToolSeparator`: `CI at its head · Passed`, `Gate 3 · review · blocking · Approved`.
- **Landings:** `PublicNothingLanded` when it has none, its lines separated likewise. Its landed commits stay links to the public landing page, now seven digits.
- **Projects:** `PublicNoProjects` when empty.

In `public/forge.rs`, `commit` and `hex` go, along with `external`'s uses here. A `short(hash: &[u8; 32]) -> [u8; 7]`-shaped helper is the only printer of a commit. `pull` and `run` links go. `external` stays for the banner's sign-in.

### 4. The look

In `temper.css`, a new `/* ==== ui: public ==== */` marker placed before `/* ==== ui: welcome ==== */`, never at the end of the file, holding every new rule:

- `:root:has(#app > .temper-public-page)` sets the `--gf-*` tokens and the body, as the chats and needs blocks each do.
- `#app:has(> .temper-public-page)` sets `--temper-mono: var(--gf-sans)` and its neighbours as `#app:has(> .temper-welcome)` does, so that nothing is monospace, the banner included.
- `#app > .temper-public-page`: the column, max 720 px, as `.temper-chats-page`. The rows' rules as `.temper-chats-rows` and `.temper-chats-row`, with a nested tree indented and without a second heavy rule. `PUBLIC_FACTS` muted, normal wrapping, no `nowrap`, no grid.
- Under 760 px, only the page's padding changes, as `.temper-chats-page`'s does. The rows are the same at both widths.

### 5. Tests

In `temper/crates/temper-client-view/src/tests/public.rs`, beside its helpers, with a helper that reads a whole paragraph's text so that touching words fail:

```rust
#[test] fn a_goals_row_shows_phase_time_and_spend_and_a_tasks_no_spend() {}   // exact text of each second line
#[test] fn an_ended_goal_reads_landed_failed_or_ended_and_a_task_done_failed_or_ended() {}  // and "cancelled" appears nowhere
#[test] fn an_ended_row_says_how_long_ago() {}
#[test] fn the_board_says_what_it_is_and_when_it_has_no_goals() {}
#[test] fn landings_show_below_the_goals_only_with_a_queue_or_a_landing() {}  // and "Nothing has landed yet." on the section and the page
#[test] fn a_commit_is_seven_plain_digits_and_no_padding() {}  // 20 bytes + 12 zeros; merge over head; landed over candidate
#[test] fn no_public_page_links_to_a_forge() {}  // every page: no external but the sign-in
#[test] fn a_lines_words_never_touch() {}  // "CI at its head · Passed", a queue's line, a landing's line
```

Update `only_words`' allowed lists, and `records.rs`'s fixtures, which give `change()` and `landing()` a 40-digit commit. A focused test for `no string field` lives in `shape.rs`. Each new test bites, as the implementer's text says.

### 6. The story

In `browser_public.rs` (compiled, not run):

- Remove `"its change on the forge"` from the forbidden list and its comment. Add `"https://"` and `"spent \u{b7} of"`.
- `Page::Task`'s advance goes to `{board}/landings` by address, since the demo has no queue and the board shows no Landings link. Keep its `see(heading(b"Landings"))` and its `Step::Address`.
- The `Page::Landings` assertion becomes `["temper", "Landings", "Nothing has landed yet."]`.
- The `Page::Empty | Page::Malformed` assertion becomes `["temper", "Public projects", "No public projects yet."]`.
- Check the board's and task's `see`s still find what they name.

### 7. Render proof

Throwaway, untracked, gone before the push. Build a `Reader` from fixtures for a board with live, landed and failed goals with tasks, a long unbroken title, and a queue. Render it the way `temper/tools/gallery/src/lib.rs`, `page`, makes a whole document, with the bundle's stylesheets. Drive Chromium over CDP:

- Use `Emulation.setDeviceMetricsOverride` for each width, since `--window-size` won't go below 500 px.
- Handle `Page.javascriptDialogOpening`.
- At each width, measure that `document.documentElement.scrollWidth <= innerWidth`, and that no two rows' or a row's text boxes intersect.

Save `public-board-1440.png` and `public-board-420.png` in the directory of your report's path, and name both paths in the report. Look at them yourself before you report: the request's point is how they read.

### 8. Close

Remove the two **Not yet** lines. Then:

- `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, and clippy on every package you changed.
- `heavy cargo nextest run -p temper-client-view -p temper-client-protocol-wire -p temper-engine-domain-forge -p temper-engine-domain -p temper-client-domain -p temper-codecs-world -p temper-engine-forge-world`
- `heavy cargo check -p temper-client-browser-world -p temper-one-process-world -p temper-client-shell-web --tests`

Commit and push.

## Report

As the implementer's text says, plus: the widths' measurements, the two screenshots' paths, each test's rule broken by hand and the failure seen, and that the story was compiled and not run.
