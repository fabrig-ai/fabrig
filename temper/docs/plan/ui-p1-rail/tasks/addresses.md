+++
id = "addresses"
part = "addresses"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-domain/src/address.rs", "temper/crates/temper-client-domain/src/lib.rs", "temper/crates/temper-client-domain/src/pages.rs", "temper/crates/temper-client-domain/src/route.rs", "temper/crates/temper-client-domain/src/tests/address.rs", "temper/crates/temper-client-domain/src/tests/pages/project_pages.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/tests/look.rs", "temper/docs/design/client/browser.status.md"]
effort = "high"
minutes = 130
+++

# Work's, Members' and Limits' addresses, and Work's three reads

## Goal

Give the client the three new addresses of the design, `Work { project, filter }`,
`Members` and `Limits`, with their paths, and make Work's five addresses (Work under
each filter, `Goals` and `Changes`) hold the goals', the changes' and the chats'
reads open together, kept across moves among them. Reserve the view's style slots
the page tasks take. Nothing is drawn for the new pages yet: `work-page` and
`deployment` draw them.

Acceptance:

1. `temper_client_domain::{Address, WorkFilter}` hold `contracts.md`'s C1, and
   `address::print`/`parse` give and read C1's paths. `/projects/3/work?show=drafts`
   reads as Drafts; `/projects/3/work?show=other` and `?x=1` read as All and print
   back as `/projects/3/work`.
2. On `Work { project, .. }`, `Goals { project, .. }` and `Changes { project }`,
   `pages().goals()`, `pages().changes()` and `pages().chats()` are all `Some` for
   that project, configured as each is on its own page today (the goals' query and
   its needs read, the changes' watch, the chats' query with its documents' notes).
   A move among these addresses of one project sends no new read and releases
   none; a move to any other page releases all three, as leaving Goals or Changes
   does today. Choosing another project on Work keeps the filter.
3. Every exhaustive match on `Address` across the workspace has its new arms, each
   doing what the nearest existing page does (below), and every package that
   matches on `Address` builds, clippy clean.
4. `temper-client-view`'s `look::NAMES` ends with the reserved names
   `temper-reserved-646` to `temper-reserved-675` (C6), and its look tests pass.
5. The focused tests of `temper-client-domain`, `temper-client-view`,
   `temper-client-tui`, `temper-client-shell-tui`, `temper-client-shell-web`,
   `temper-client-shell-terminal` and `temper-client-gallery` pass.
6. `temper/docs/design/client/browser.status.md` loses its entry
   "**Not yet:** 2.3: `/projects/<project>/work`, `/deployment/members` and
   `/deployment/limits` read as no page", whole.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md` and
  `temper/docs/plan/ui-p1-rail/notes/contracts.md`, C1, C2 and C6.
- `temper/docs/design/client/browser.md`, 2.3: the table's rows for `Work`,
  `Members` and `Limits`, and the bullet *Work's filter is its query*.
- `temper/docs/design/client/README.md`, 3.5, *Work*, its bullet *Its reads*; 3.5,
  *Goals*, its paragraph *What it reads*; 3.5, *Changes*; 5.3, the sentence
  "Choosing a project keeps the page where it can".
- The code today: `address.rs` (`print`, `parse`, `GoalsFilter`'s and the chats
  filter's queries); `pages.rs`, `went` (from "fn went(pages: &mut Pages"), its
  same-project Goals filter move, its page opening and leaving, `rail_handles` and
  the handle numbering (`pages.next`); `pages.rs`'s configuration of the Goals and
  Chats pages (where it calls `configure`, opens chats' documents on `Chats` alone,
  and asks the standing needs watch on Goals and Changes); `PageAct::Project`'s
  arm (the project's page kept on a switch).

## Owns and may change

Owns: `temper-client-domain`'s `address.rs`, `lib.rs` (the re-export), `pages.rs`
and `route.rs`, and the tests `src/tests/address.rs` and
`src/tests/pages/project_pages.rs`; `temper-client-view`'s `look.rs` (the reserved
names alone) and `src/tests/look.rs`; the one status line of acceptance 6.

Reader and neighbour files, compiler-forced arms only, each the nearest page's
behaviour:

- `temper-client-domain`: `pages/switcher.rs`, `pages/needs.rs`,
  `pages/connect_model.rs`, `pages/terminal.rs`, `pages/home.rs`,
  `pages/models.rs`, `pages/away.rs`, `sign_in_tab.rs`, `forge/trials.rs`,
  `domain.rs` and any other the compiler names. `Work` as `Goals`, `Members` and
  `Limits` as `Accounts`. In `pages/needs.rs` and `pages/home.rs` (another plan's
  files) the arm alone, nothing else.
- `temper-client-view`: `root.rs`, `frame.rs`, `choice.rs`, `shared/refusal.rs`,
  `pages/terminal.rs`, `pages/terminal/commands.rs`, `pages/connect_model.rs` and
  any other the compiler names. `Work` as `Goals`; `Members` and `Limits` as
  `Accounts`, but in `frame.rs`'s `settings_link`, where they are current as
  `Builders` is. Add no page drawing: the dispatch in `root.rs` is an `if let`
  chain, and an address with no branch draws no body until `work-page` and
  `deployment` add theirs.
- `temper-client-tui` (`view.rs`, `view/launchpad.rs`) and
  `temper-client-shell-tui` (`host.rs`): `Work` as `Goals`, `Members` and
  `Limits` as `Accounts`. These are the TUI's owner's files: arms alone.
- Tests elsewhere that list every address (`temper-client-shell-web`'s
  `host_tests.rs`, `temper-client-shell-terminal`'s, the gallery's): a row for each
  new address only where the test's table lists every address, so that it still
  covers them all.

Pre-approved options:

- **If the three reads together need more handles than `pages.next`'s numbering
  gives a page today:** number them in `went` as Home numbers its three (it opens
  home, chats and inbox in a row), keeping `Changes`' reserved rail range. Say the
  numbering in the report.
- **If the chats' rail handles (`rail_handles`, `rail_chats`) would open a second
  chats read beside Work's:** let Work's chats page serve the rail as `Chats` does,
  so that one chats query is open, and say so.
- **If a look test refuses a reserved name:** name the slots as slots 0 and 21 are
  named (`temper-reserved-N`) and adjust the test's expectation to say so.
- **If a test of another package fails only because it lists every address in
  order:** add the rows; anything else is that package's owner's, under `## Left`.

## Steps

1. **Addresses.** Add C1's variants and `WorkFilter` in `address.rs`, its doc
   comment citing `temper/docs/design/client/browser.md, 2.3`; re-export
   `WorkFilter` beside `GoalsFilter`. `print`: `Work` as `Goals` prints, its query
   `?show=drafts` for Drafts; `Members` and `Limits` as `/deployment/members` and
   `/deployment/limits`. `parse`: those paths, and Work's `show` read as the
   chats' `state` is, an unknown value or field dropped. Extend the doc comment's
   path list.
2. **Tests of the paths.** Rows in `tests/address.rs`'s
   `each_page_round_trips_through_its_path` for the four addresses; rows in the
   test beside `goals_filter_is_its_query` (or a sibling table in its shape) for
   `?show=drafts`, `?show=other`, `?x=1` and a trailing slash.
3. **Work's three reads.** In `pages.rs`'s `went`: on any of Work's five addresses,
   open the goals', changes' and chats' pages for the project as their own pages
   open them (C2); a move among them for one project keeps them, setting
   `goals.filter` from a `Goals` address, as the same-project Goals filter move does
   now; any other move leaves them as today. Wherever `pages.rs` configures the
   goals page on Goals or opens the chats' documents on Chats, do the same on Work's
   addresses. Wherever it asks the standing needs watch on Goals and Changes, on
   Work too. `PageAct::Project` on Work goes to the new project's Work with the same
   filter.
4. **Tests of the reads**, in `tests/pages/project_pages.rs`, as one table over the
   five addresses: each opens the three reads (the goals' query, the changes' watch,
   the chats' query and notes, by the requests sent); a move to each other of them
   sends no read and releases nothing; a move to `Chats { filter: All }` releases
   the goals' and changes' reads and keeps or reopens the chats' as `Chats` does
   today; choosing another project on Work keeps its filter. Break the keeping once
   by hand (reopen on every move) and see the move row fail; restore.
5. **Forced arms** across the workspace, as Owns says. Build each package the
   compiler names with `heavy cargo check -p PKG --tests`.
6. **Reserved style slots** in `look.rs`'s `NAMES`: thirty lines,
   `b"temper-reserved-646"` to `b"temper-reserved-675"`, after the last name, with
   one comment line above them in the file's manner: "Reserved for the rail, Work
   and Deployment (client/browser.md, 2.5)". No constant yet.
7. **Status:** remove acceptance 6's entry from `browser.status.md`.
8. **Check** as `notes/common.md` says, for the packages of acceptance 5. Commit
   and push. The message says why: Work's filters share their reads, so a move
   among them reads nothing again.

## Report

Follow the implementer's text. Also give: the handle numbering you chose for
Work's three reads; each package where you added forced arms and what each arm
does; the rule you broke for step 4 and the failure you saw.
