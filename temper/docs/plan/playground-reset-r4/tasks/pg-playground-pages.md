+++
id = "pg-playground-pages"
part = "playground"
kind = "feature"
needs = ["pg-client-read", "pg-words"]
files = [
  "temper/crates/temper-client-domain/src/forge/demo.rs",
  "temper/crates/temper-client-domain/src/forge/form.rs",
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/forge/listed.rs",
  "temper/crates/temper-client-domain/src/tests/playground_end.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/forge/forges.rs",
  "temper/crates/temper-client-view/src/pages/accounts.rs",
  "temper/crates/temper-client-view/src/tests/playground.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/crates/temper-client-view/src/tests/demo.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/*.txt",
  "skein/crates/skein-ui-text/src/render.rs",
  "skein/crates/skein-ui-text/src/tests.rs",
  "skein/crates/skein-ui-text/src/tests/fixtures.rs",
  "skein/crates/skein-ui-text/src/tests/golden/*.txt",
]
effort = "high"
minutes = 60
+++

## Goal

The playground's pages and its way out (draft
`temper/docs/design/draft/playground-reset.md`, 6.1 "Its pages", 6.2
"The page"): while the playground runs, every page's banner is one
line, "This is the playground: a scripted model and in-memory forge.
Use the wizard to [set up temper]"; Forges lists the playground forge's
row and Accounts the Scripted model, neither offering anything to add;
"set up temper" sends `EndPlayground`, the banner then says "Leaving
the playground…" until the reset answers, and the page opens the
welcome's Forge step, `/welcome?at=forge`. The setup strip and the demo
banner's Connect links go.

Acceptance:

- with the playground running, every page (Welcome, Chats, a chat, a
  task, Inbox, Adoption, a project's settings, Forges, Connect a forge,
  Accounts, Connect a model) draws one banner region named "Playground"
  holding the paragraph "This is the playground: a scripted model and
  in-memory forge. Use the wizard to set up temper", whose last words
  are a link "set up temper" bound to the reset; no "Connect a forge"
  or "Connect a model" link in it; no setup strip on any page, ever;
- pressing "set up temper" sends one `w::Request::EndPlayground`
  (family request, deployment scope); while it is out, or while the
  read answers `resetting`, the banner says "Leaving the playground…"
  and offers nothing; a second press sends nothing;
- answered `reset`, the side reads the playground again, and once the
  read answers `not_started` the page shows
  `Address::Welcome { step: Step::Forge { forge: None } }` (pushed, not
  replaced), once; answered `end_refused`, the banner says "The
  playground was not left: not offered." with "Try again", which sends
  a new `EndPlayground`; an unknown or lost answer reads again;
- a read answering `resetting` with no `EndPlayground` of this session
  out (a reset another session or a restart left under way) sends one
  `EndPlayground`, whose answer the side then awaits as its own;
- Forges, while the playground runs: no "+ Add a forge" and no
  "Connect a forge" link; the playground's heading and its row, named
  "Playground forge · in memory · OWNER/NAME"; the paragraph "Nothing
  to add while the playground runs: set up temper to bring your own.";
  Accounts, while it runs: no connect link, the Scripted model's row as
  today, and the same paragraph;
- the tests below pass, and every existing test of the packages
  changed (the terminal's goldens updated to the new banner).

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 5).
- `temper/docs/design/draft/playground-reset.md`, 6.1, 6.2, 6.3.
- `temper/docs/design/client/README.md`, 5.1 (the banners' place) and
  5.6 (the first run's words), whose demo banner and setup strip this
  replaces.
- `temper/docs/plan/playground-reset-r4/tasks/forge-asks.md`
  (`EndPlayground`, `reset`, `end_refused`), `tasks/pg-wire.md` (the
  read's `resetting`), `tasks/pg-client-read.md` (the side this builds
  on), `tasks/pg-words.md` (the words: `PlaygroundBanner`,
  `PlaygroundSetUp`, `PlaygroundLeaving`, `PlaygroundNotLeft`,
  `PlaygroundTryAgain`, `PlaygroundForgeLine`, `PlaygroundNothingToAdd`).

## Owns and may change

Owned: the `files` above (`tests/playground_end.rs` and the view's
`tests/playground.rs` are new).

Neighbour files, edits allowed:

- `temper-client-domain/src/tests.rs`, `temper-client-view/src/tests.rs`:
  one `mod` line each.
- Tests elsewhere in `temper-client-domain`, `temper-client-view`,
  `temper/tests/client/` (`tests/suite/demo_start.rs`,
  `tests/suite/welcome.rs`, `tests/suite/forges.rs`,
  `tests/suite/accounts_page.rs`, qa files) that pinned the demo banner's
  words, its Connect links, the setup strip, or the Forges page's add
  links while the playground runs: their expectations updated to this
  task's, each named in the report.
- `temper/tools/gallery/` samples that draw the strip: the forced or
  plainly wrong expectation only.
- Any golden or expected print elsewhere in `skein/**` or `temper/**`
  that the inline link numbering (Answers) changes: the numbers only,
  each named in the report.

Pre-approved:

- `listed::setup` and `listed::Setup` go with the strip, and the strip's
  words stay in `words.rs` unused (no edit to `words.rs`); if a reader
  outside the view needs `setup`, keep the function and say so;
- the "Leaving the playground…" line replaces the banner's paragraph in
  the same region; default: the region keeps its name "Playground";
- the side's "left" signal (read answers `not_started` after this
  session ended it or saw `resetting`) is a one-shot flag on `Side`
  that `route.rs` takes to push the `Show`; if the route has a closer
  idiom (the set up's `Show` on `SettingUp`), use it.

## Steps

1. **Domain** (`forge/form.rs`, `forge/demo.rs`, `forge.rs`, `route.rs`,
   `pages.rs`):

   ```rust
   // forge::Act, beside StartDemo:
   /// The banner's "set up temper": the playground's reset (draft 6.2).
   EndPlayground,
   // pages::Act, beside Playground (the frame's binding routes it to the forge's side):
   LeavePlayground,
   // forge::demo:
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum EndFailure { NotOffered }
   pub fn end_ask(limits: &Limits) -> Option<jig_client::Ask>        // w::Request::EndPlayground, Scope::Deployment
   pub fn end_answer(value: &jig_client::Answered, limits: &Limits) -> Option<EndAnswer>
   pub enum EndAnswer { Reset, NotOffered, Unknown, EntranceRefused(c::EntranceRefusal) }
   // Side:
   pub fn demo_leaving(&self) -> bool          // an end out, or the answer Resetting
   pub fn demo_end_failure(&self) -> Option<EndFailure>
   ```

   The end's state on `demo::State`: `end: Option<Starting>` (its
   request), `end_failure`, `left: bool`, `resend: bool` (sent once per
   `resetting` answer). The route: `Act::LeavePlayground` → the forge's
   `Act::EndPlayground`; the `left` flag → `Request::Show { address:
   Address::Welcome { step: Step::Forge { forge: None } }, replace:
   false }`.
2. **View** (`frame.rs`, `root.rs`): `demo_banner` draws, when
   `root.forge().demo_leaving()`, the region with `PlaygroundLeaving`;
   else when `root.demo()` is some, the region with one paragraph:
   `PlaygroundBanner`'s text then a link `PlaygroundSetUp` bound to
   `Act::LeavePlayground`, and, with `demo_end_failure()`,
   `PlaygroundNotLeft` (why: "not offered") and a `PlaygroundTryAgain`
   button bound to the same act. `root.rs` draws `demo_banner` on every
   address (the Welcome too); the setup strip and `setup_underway` go.
3. **Forges and Accounts** (`forge/forges.rs`, `pages/accounts.rs`):
   with `root.demo()` some and not resetting, skip the add and connect
   links, draw the playground row with `PlaygroundForgeLine` (owner and
   name from the answer), and the `PlaygroundNothingToAdd` paragraph.
4. **Tests** (client domain and view unit tests):
   - `temper-client-domain/src/tests/playground_end.rs`:
     `set_up_temper_sends_one_end_playground` (a running answer; the
     act twice: one `Send` decoding to `w::Request::EndPlayground` at
     the deployment's scope);
     `reset_then_not_started_opens_the_forge_step_once` (answer
     `reset`: a read sent; read `not_started`: one `Show` of
     `/welcome?at=forge`, `replace: false`; another read: no second
     `Show`);
     `a_resetting_read_awaits_its_own_end` (no end out, read
     `resetting`: one `EndPlayground` sent, `demo_leaving()`; a second
     `resetting` read sends none);
     `a_refused_end_offers_try_again` (`end_refused`:
     `demo_end_failure() == Some(NotOffered)`, not leaving; the act again
     sends a new request under a new key);
     `an_unknown_end_reads_again`.
   - `temper-client-view/src/tests/playground.rs`:
     `every_page_has_the_one_line_banner` (for each address in the
     acceptance list: one region "Playground", its paragraph's name the
     banner's words, its link "set up temper" decoding to
     `Act::LeavePlayground`, no "Connect a forge"/"Connect a model" link
     in it);
     `leaving_shows_its_line_and_no_link`;
     `no_setup_strip_without_a_forge_or_model` (no playground, no
     forge, no model: no region "Setup" on Chats);
     `forges_and_accounts_offer_nothing_to_add_while_it_runs` (no
     "+ Add a forge", no "Connect a forge"; the row "Playground forge ·
     in memory · acme/shop"; the paragraph; Accounts: no connect link,
     the paragraph).
   - The terminal's goldens (`temper-client-shell-terminal/src/tests/golden/`):
     regenerate by that test file's write mode if it has one, else
     edit the banner lines by hand to the new words, nothing else.
   - Narrow runs per file, then the packages' focused tests.
5. Checks, once: `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view --all-targets -- -D
   warnings` (and each other package changed); focused tests of
   `temper-client-domain`, `temper-client-view`,
   `temper-client-shell-terminal`, `temper-client-world`; `heavy cargo
   check --tests` for `temper-client-tools`, `temper-client-shell-web`,
   `temper-client-gallery`, `temper-one-process-world`, one at a time.
   Commit, push.

## Report

fab's sections; `## Interfaces`: the acts, `EndFailure`, `EndAnswer`,
`end_ask`, `end_answer`, `demo_leaving`, `demo_end_failure`; every test
elsewhere whose expectation changed, by file and name.

## Answers

2026-11-01, from this task's question (the terminal prints the banner's
"set up temper" with no number, so it cannot be pressed): extend
skein-ui-text, first, in a commit of its own; the banner stays one
paragraph. `skein/docs/design/ui-text.md`, section 6, already numbers
every bound, enabled link "from 1 in preorder", wherever it sits; a
paragraph flattening its links departs from it, and the paragraph's row
in section 5's table now says so. In `render.rs`, `line` and `content`
print a bound link descendant of a paragraph (and of any role printed
by `line`) in place as `[n] TEXT`, numbered through `number` in
preorder, a disabled or unbound one `[-] TEXT`, an external link of
Markdown's as today (`TEXT <URL>`, no number); a control's own content
is unchanged. A test in `skein-ui-text`'s `tests.rs`,
`a_link_inside_a_paragraph_is_numbered_in_place` (a paragraph "Use the
wizard to " then a bound link "set up temper", and a button after it:
the line `Use the wizard to [1] set up temper`, the button `[2]`, and
the print's `numbers` naming the link's node first), then this task's own work; the
terminal's goldens then show the banner's link numbered.
