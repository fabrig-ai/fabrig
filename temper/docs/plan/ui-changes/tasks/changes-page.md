+++
id = "changes-page"
part = "changes-page"
kind = "feature"
needs = ["changes-wire"]
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/changes.rs",
  "temper/crates/temper-client-domain/src/forge/changes.rs",
  "temper/crates/temper-client-domain/src/tests/pages/changes.rs",
  "temper/crates/temper-client-view/src/pages/changes.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/pages/task.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/changes.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
]
effort = "high"
minutes = 190
+++

# The `Changes` page

## Goal

`Changes { project }` looks and works exactly as the greenfield mock's `changes.html`, with its rows links,
on the rail after "Chats"; a change's page gains its "Changes" breadcrumb. It reads the subject `changes`
built to `notes/contracts.md`: the engine's half lands apart (`changes-view`), so samples and tests deliver
`ChangesPanel`s themselves.

Acceptance:

- `/projects/<project>/changes` round trips; the page opens its watch on arrival and closes it on leaving;
  the switcher keeps the page across projects.
- The heading, the lede and its landing's line, the queue's rows and "Landed recently" say exactly
  `temper/docs/design/client/README.md`, 3.5, *Changes*, each form with a test.
- A row's mono part is read as the design says, one read per row shown, and absent until it arrives.
- "Changes" is on the rail, current on the page, and not in the tab bar.
- A change's page's bar opens with "Changes", to its project's page.
- Gallery samples `changes`, `changes-empty`, `changes-one` and `changes-landed-only`; the terminal prints the page.
- **The proof,** in Step 8.

## Inputs

- `temper/docs/plan/ui-changes/notes/common.md`; `temper/docs/plan/ui-changes/notes/contracts.md`, *The wire*
  and *The client*.
- `temper/docs/design/client/README.md`, by `rg`, never whole: 3.5, *Changes* and its table's row; 4.6, *A
  project's changes*; 5.3's rail; 5.10's parts; 5.12.
- `temper/docs/design/client/browser.md`, 2.3.
- **The mock:** `temper/docs/design/client/greenfield/`: `changes.html`; `app.js`, `changesPage`; `data.js`,
  `changes:` (lines 259 to 270, 334 and 384); `mock.css`, `.lines`, `.mono`, `.state-running`, `.empty`, its
  phone rules. `temper/docs/plan/ui-changes/inputs/request.md`, section 1, gives each rule.
- **The models:** `temper-client-domain/src/forge/dismissed.rs` (a family watch's record and life),
  `pages/home.rs` (a page's watches), `pages/chat.rs:438` (a read per task); `temper-client-view/src/pages/home.rs`.

## Owns and may change

Owns `files`. May also change:

- **The arms `Address::Changes` asks,** each a line: `temper-client-domain/src/{departure,route}.rs`,
  `pages/{connect_model,switcher}.rs`, `temper-client-view/src/choice.rs`, their tests' exhaustive matches.
- `temper-client-domain/src/lib.rs`, `forge.rs`, `pages.rs`: `mod` and `pub use` lines.
- **Reprints:** the terminal's golden pages (`temper-client-shell-terminal`, regenerate by its `render.rs`'s
  way), and any test naming the rail's entries in order (`rg '"Chats"' temper/crates/temper-client-view/src/tests
  temper/tests/client temper/tests/client-browser`), for the new entry alone. A gallery sample's print of a change's
  page takes the breadcrumb.
- `temper/tests/client/tests/suite/*`: a new `changes.rs` world, if one is cheaper than step tests.
- **Design, its closed lines alone:** `temper/docs/design/client/README.status.md` (3.5, 4.6, 5.3, 5.10; the
  banner's stays), `client/forge.status.md` (3 and 9; 2.1 stays if a panel's own place still lags, stated),
  `client/mockups/STATUS.md`'s "Board and Changes" (Board stays).

Pre-approved:

- **Home's rules are reused as they stand:** the page's main takes `look::HOME`, with `HOME_INTRO`,
  `HOME_KICKER`, `HOME_LEDE`, `HOME_ROWS`, `HOME_WHEN` and `HOME_EMPTY`, whose rules already draw the mock's
  `.col`, `.intro`, `.lines` and `.when`: no rename, so that parallel plans on Home merge. If one must differ
  (the "and N more" row, a `li` with no link; the mono face; the running and the red colours), it's a new token
  and a rule under `/* ==== ui: changes ==== */`. If a Home rule fights the mock here, a scoped rule under that
  marker wins, and the report says which.
- **Home's "Landing:" line is not touched:** `ui-home-status` draws it, and moves to this line's function if it
  lands second. If it landed first, this page moves to its function, and the report says so.
- If `Task.goal`'s chip is not in `lineage_chips`, the row has no mono part; if the root chip is not a chat's, no
  mono part either.
- If the mock and the request differ, the request wins, and the design over both where it speaks; the report says
  each.

## Steps

1. **The address:** `Address::Changes { project }`, `print`, `parse` and the test pairs in `tests/address.rs`;
   `pages.rs`' project-of and show-across-projects arms, as `Address::Settings { .. }`; the compiler's others.
2. **The watch,** `forge/changes.rs`:

   ```rust
   pub fn watch(project: u32, limits: &Limits) -> Option<Watchable>; // of `c::Scope::Project(project)`
   pub fn panel(record: &Family, limits: Limits) -> Option<w::ChangesPanel>; // by `wire::decode_changes_panel`
   ```
3. **The page,** `pages/changes.rs`:

   ```rust
   pub enum State { Loading, Shown(Box<w::ChangesPanel>), Ended }
   pub struct Changes { pub project: u32, /* its handle, its reads, one per row shown */ }
   impl Changes {
       pub(super) fn new(project: u32, serial: &mut u64, limits: forge::Limits, out: &mut Queue<Request>) -> Self;
       pub(super) fn leave(&self, out: &mut Queue<Request>);
       pub(super) fn told(&mut self, told: &Told, client: &Client, limits: forge::Limits, out: &mut Queue<Request>) -> bool;
       pub fn state(&self) -> &State;
       /// Where a listed task came from, once its read has arrived.
       pub fn from(&self, client: &Client, task: u64) -> Option<From<'_>>;
   }
   pub enum From<'a> { Goal(&'a [u8]), Chat(u64) }
   ```

   Each delivery opens the reads of tasks newly shown and closes the ones gone. Until the first document, the
   page is its kicker; a refused or ended watch adds the forge's "Shown on the forge." (`README.md`, 4.2), and
   it isn't opened again while the page stands.
4. **The view,** `temper-client-view/src/pages/changes.rs`, called from `root.rs` as Home is:

   ```rust
   pub(crate) fn main(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, project: u32, times: &mut Times);
   /// The landing's line, Home's too in time: its parts, each saying if it is the signal's.
   pub(crate) struct LinePart { pub words: Box<[u8]>, pub red: bool }
   pub(crate) fn landing_line(panel: &w::ChangesPanel, words: &Words, times: &mut Times) -> Box<[LinePart]>;
   /// The name the heading and the lede say.
   fn branch_named(panel: &w::ChangesPanel) -> Option<&[u8]>;
   ```

   `branch_named`: the one branch *name* among queues with rows, whichever repositories; none with rows,
   `landing` unless empty; else `None`, the "landing" and "its branch" forms. A section per `(repository, branch)`
   with rows; one alone is plainly "Queue". Several queues' `more` is one count: its "and N more" is the last
   section's last row. The queued count, the rows and `more`. Words go in `words.rs`; "queued 2nd" by a small
   ordinal (2nd, 3rd, 4th, 11th, 21st, 22nd, 23rd), "Three" by words two to nine, digits from ten.
5. **The rail and the crumb:** `frame.rs`: "Changes" after "Chats" in the rail alone, so `entries` takes whether
   it draws it; `pages/task.rs`: on a change's page (a forge panel's subject, its `project`), the bar's first
   link, "Changes".
6. **The look:** `look.rs` tokens and `temper.css` rules under the marker. Compare `mock.css`, by the request's
   figures, at both widths.
7. **Samples and tests:**
   - gallery: `changes` (`data.js`' fabrig, three queued, four landed), `changes-empty` (nothing landed),
     `changes-one`, `changes-landed-only` (`data.js`' hello); `worlds.rs`/`samples.rs` deliver their
     `ChangesPanel` for `w::Watch::Changes`; `names.txt` and `tests.rs` by `notes/common.md`.
   - `temper-client-domain`: the watch's life; reads bounded by the rows, opened and closed; `Ended` isn't opened
     again; the switcher; `from`, goal, chat and neither.
   - `temper-client-view/src/tests/changes.rs`: each heading form (0, 1, 3, 9, 10, 12; two branches); each line form;
     `main red` with nothing queued; a first row's words and link; `queued 2nd`, an ordinal's table; "and N more" not a
     link; "Empty."; landed's clock; the rail's entry current; no tab-bar entry; the crumb.
   - `temper-client-shell-terminal`: a print of the page, in `fixtures`' way.
8. **The proof:** render the gallery's page against the mock's at 1440, 1000 and 420 wide: `changes.html` against
   `changes`; `changes.html?project=hello` against `changes-landed-only`; a project with nothing landed against
   `changes-empty` (the mock's: `data.js`' state, or a local copy of the state in localStorage). `chromium
   --headless=new --no-sandbox --disable-gpu --allow-file-access-from-files --screenshot=OUT.png
   --window-size=W,H --virtual-time-budget=3000 file://...`, and `notes/common.md`, *Render proofs*. Screenshots,
   side by side by name, go in `~/src/fabrig/ui-changes/` on your builder, outside any worktree, never committed.
   Fix what differs but the fonts.
9. **Check:** the narrow tests, then `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p
   temper-client-gallery -p temper-client-shell-terminal`, `-p temper-client-world`; `heavy cargo check -p temper-client-browser-world
   --tests`; fmt, old forms, clippy; the citations.
10. Commit and push.

## Report

As the implementer's text has it, with the builder holding the screenshots, what you compared and each remaining
difference from the mock, by page and width.
