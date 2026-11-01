+++
id = "cl-pg"
part = "pg"
kind = "feature"
needs = ["cl-welcome"]
files = [
  "temper/crates/temper-client-domain/**",
  "temper/crates/temper-client-view/**",
  "temper/tools/gallery/**",
  "temper/tests/client/**",
  "temper/docs/design/client/journeys/*/0*.html",
]
effort = "high"
minutes = 120
+++

## Goal

The playground's pages and its reset in the client (draft
`temper/docs/design/draft/playground-reset.md`, 6.1 and 6.2): one banner line on every page
while the playground runs, its "set up temper" sending `end_playground`, "Leaving the
playground…" until the reset answers, then the welcome's Forge step; Forges and Accounts
offer nothing to add while it runs.

- The client's playground state reads the wire's three: `not_started`, `running`,
  `resetting` (`forge::demo::DemoState` gains `Resetting`; `Resetting` no longer reads as
  `Ended`, cl-wire's interim).
- While the playground runs, every page but the welcome's draws, in the banner's place
  after the accounts' banner (today's `demo_banner` place), one line: "This is the
  playground: a scripted model and in-memory forge. Use the wizard to " followed by the
  link "set up temper" and nothing else: no Connect a model, no Connect a forge, no View.
  This holds on Accounts, Connect a model, Forges and Connect a forge too.
- "set up temper" sends `end_playground` (deployment scope, keyed, one at a time). From the
  press until the answer, and while a read says `resetting`, the line is "Leaving the
  playground…" (a status), with no link.
- `reset` → the client reads the playground again and opens `/welcome?at=forge`
  (`Request::Show { Welcome { Forge { None } }, replace: false }`), once. `not_offered` or
  `unknown` → read again and draw what the read says. A page opened while the read says
  `resetting` reads again every 2 s until it says otherwise, then, from `resetting` to
  `not_started`, opens the welcome's Forge step as `reset` does.
- While the playground runs: Forges lists the playground's forge in its own section
  (whatever the forge half said) and draws neither "+ Add a forge" nor "Connect a forge";
  Accounts lists the Scripted model in its section and draws no Connect/Add link. A
  person's own rows, if any, are listed as they are.
- "Try the playground" with a read whose `project` is 0 (the engine makes the project at
  the start, contracts T6) selects no project until the start answers `running`, then
  reads the playground again and selects its project; with a nonzero `project` (the
  committed journey's recorded read) it does exactly what it does today, asking nothing
  more.
- The committed `welcome-playground` journey replays without going stale; its `02.html`
  (the chats page, whose banner changes) is reprinted.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T6, T8;
  `notes/cl-wire.md` 1 to 3 (end_playground's outcomes, `Read::playground`'s members).
- `temper/docs/design/client/README.md`, 4.8 (the demo in the client today), 5.1 (the
  banner's place), 5.6 (the first run's words); `temper/docs/design/client/journeys.md`, 8.
- The draft, 6.1 and 6.2.

## Owns and may change

Owned: `temper/crates/temper-client-domain/`, `temper/crates/temper-client-view/` (src and
tests), `temper/tools/gallery/`. The joined client world (`temper/tests/client/`) where the
compiler requires or an expectation moves (listed in the report); the shells and tools only
where the compiler requires. The journeys' printed pages by the reprint alone (never a
recording; `notes/cl-progress.md`, request 5).

Keep, for `cl-pg-goes`: the halves, the retired playground, the setup strip and the
client's removal of the scripted account stay in the code (the banner no longer reads the
halves); this task removes none of them, so that its diff stays the playground's pages.

Options, pre-approved:

- The banner's sentence and link are one paragraph whose last run is the link; if skein's
  builder cannot put a link inside a paragraph's text, the paragraph's text then the link
  right after it, in one region named "The playground".
- The re-read period is a constant of the forge side (2 s); `forge::next_deadline` carries
  it.

## Steps

1. **Domain** (`forge/demo.rs`, `forge/family.rs`, `forge.rs`, `route.rs`, `pages.rs`):
   `DemoState::Resetting`; `family::demo` maps `w::PlaygroundState::Resetting` to it. An
   end attempt on the demo's state, `Ending { request: Token, progress, answer:
   Option<EndAnswer> }`, `pub enum EndAnswer { Reset, NotOffered, Unknown,
   EntranceRefused(c::EntranceRefusal) }`, `demo::end_ask(limits)` (`w::Request::
   EndPlayground`) and `demo::end_answer` (`w::Outcome::Reset`, `NotOffered`, `Unknown`).
   `forge::Act::EndPlayground`; a pages act `Act::LeavePlayground` routed as
   `Act::Playground` is (`route.rs`). `Side::leaving() -> bool` (an end in flight, or the
   answer `resetting`); `Side::playground_running() -> bool` (the answer running and not
   leaving). The reset's navigation pushed from `route.rs` once per reset. The start: when
   the answer's `project` is 0, `route::playground` selects nothing and, on the start's
   `Running`, reads again and selects the read's project.
   Tests (`src/tests/demo.rs`, new cases):
   `resetting_reads_as_resetting`,
   `set_up_temper_sends_one_end_and_leaves_until_its_answer`,
   `reset_reads_again_and_opens_the_welcomes_forge_step_once`,
   `not_offered_and_unknown_read_again`,
   `a_resetting_read_reads_again_every_two_seconds_then_opens_the_forge_step`,
   `a_start_with_no_project_selects_the_read_project_once_running`,
   `a_start_with_a_project_asks_nothing_more` (the journeys' path).
2. **View** (`frame.rs`, `forge/forges.rs`, `pages/accounts.rs`, `words.rs`): the banner
   (`playground_banner`, replacing `demo_banner`'s halves branch; the setup strip's branch
   stays for cl-pg-goes), on every address but `Welcome`, while
   `playground_running()` or `leaving()`; Forges' playground section from the playground
   running; "+ Add a forge", "Connect a forge" and Accounts' Connect/Add links not drawn
   while it runs. Words: `PlaygroundBanner` "This is the playground: a scripted model and
   in-memory forge. Use the wizard to ", `PlaygroundSetUp` "set up temper",
   `PlaygroundLeaving` "Leaving the playground…".
   Tests (`src/tests/demo.rs` and `src/tests/frame.rs`):
   `the_playground_banner_is_one_line_on_every_page_but_the_welcome`,
   `set_up_temper_is_the_banners_one_link`,
   `leaving_replaces_the_line_with_its_status`,
   `forges_and_accounts_offer_nothing_to_add_while_it_runs`,
   `a_persons_own_account_is_listed_beside_the_scripted_model`. Update the existing banner
   tests (`the_demo_banner_follows_the_frame_on_work_pages`,
   `a_forge_only_banner_links_to_connect_without_a_model_link` and kin) to the one line,
   or remove those whose subject (a half's wording) the banner no longer has, listing each.
3. **Gallery:** `chats-playground` (the banner) and `chats-playground-leaving`.
4. **The journeys:** `heavy cargo nextest run -p temper-client-journeys --lib` must report
   no stale journey; reprint (`heavy cargo run -p temper-client-journeys`) and commit the
   changed `NN.html` pages. Read `welcome-playground/02.html`'s diff: the banner's line
   alone changes.
5. `heavy cargo nextest run -p temper-client-domain --lib`, `-p temper-client-view --lib`,
   `-p temper-client-gallery --lib`, `-p temper-client-world --test suite` (its
   `demo_start` cases still use the world's scripted demo: keep them passing, adjusting
   only what this task's banner or reads move), `-p temper-client-shell-web --lib`, `-p
   temper-client-shell-terminal --lib`, `-p temper-client-tools --lib`. `cargo fmt`;
   clippy `--tests` and the wasm clippy of domain and view; old forms; commit and push.

## Report

- The reprinted pages, and the client-world expectations moved.
- Under `## Left`: `temper/docs/design/client/README.md` 4.8, 5.1 and 5.6 text this
  replaces.
