# Session 07: the task's page, adoption, the confirmations; `jig-client-view` dropped

The last of the pages temper's stories use become temper's own: a task's
page, after `task.html`, with a tracked task's header, what was asked, its
plan as a tree and its latest activity, and a held task of its plan
released after a confirmation; and the adoption form as one of temper's
pages, in temper's frame. Then temper's client shows nothing of jig's
reference UI: `jig-client-view` leaves temper's manifests, with the
reference UI's child, its addresses and its slots, and temper's main
says what the link's states not about the work need (signed out,
starting, out of date). The stories that still visit jig's pages move to
temper's, and the goldens are recorded again by hand (`client.md`, 3.5,
4.3, 4.4, 9; README, Done when, 8).

## Read first

- temper's `docs/design/client.md`: 3.5 (the rows `Task { task }` and
  `Adopt { project }`, and the bullets "A task's page" and "The forge's
  parts"), 3.6, 4.2 to 4.5, 5.2, 5.3, 6.6 (signing out), 9, 10 (the
  stories "a held task released from a goal's plan" and "a repository
  adopted from the navigation's link"; "Golden text").
- temper's `docs/design/web/mockups/task.html`; `docs/design/domain/testing.md`,
  2.5 (the one-process world's stories) and 2.6 (the process tier's).
- jig's `docs/design/client/domain/contract.md`, 5.1 (a task, a tree),
  5.2 (revisions), 5.4; `client/domain/client.md`, 8.2 (the link's
  states) and 12 (signed out).
- This plan's `temper/06-pages.md` (the pages child, `Confirm`, the
  names the fake person finds, which this session reuses),
  `jig/15-helpers.md` (`facts`, `words`); `decisions.md`, 1c and 7;
  `later.md` (an agent's task page, Board, Changes, the confirmations no
  story makes).
- skein's `programming-model.md`: 3, 5.2, 6.3 (by number only); skein's
  `testing-strategy.md`: 2.1, 2.2, 7 (by number only).

## What the design asks

- **A task's page** (`client.md`, 3.5): "a goal: its header, what was
  asked, its plan as a tree, its latest activity and history, its budget
  and authority, with Amend, Stop and Cancel"; it watches "`task`,
  `tree`, `activity`; `run` while one is live"; "A task with the chat
  label opens as `Chat { task }`. A procedure's task or a person task
  shows `task.html`'s header and plan only".
- **The forge's parts** (`client.md`, 3.5, 4.2): "a goal's issue and
  changes on its page ... say 'Shown on the forge.'"; a connector, a
  resource, a pattern or a kind in its words (4.1).
- **Releasing** (`client.md`, 3.6): "An act that cannot be undone ...
  says what it will do before it is sent, from the facts the document
  gives: ... what releasing does"; the act names the hold's revision.
- **The adoption form** (`client.md`, 4.3): "The form is a page of
  temper's, `Adopt { project }`, reached from the navigation's 'Adopt a
  repository' (5.3)"; its title "Adopt a repository into PROJECT,
  PROJECT as `me` names the project" (4.4).
- **What goes** (`client.md`, 9): "temper's client copies no client
  root, view root or native client of jig's, and arranges no page of
  jig's".
- **Signed out** (`client.md`, 6.6): "Signed out: the terminal erases
  DIRECTORY's token, the main says 'Signed out.'".

## Choices this plan makes

- **Within `later.md`:** the task's page offers release of a held task
  of its plan, the one act a present story makes. Amend, Stop and
  Cancel on a goal, the history tab, budget and authority's detail, and
  an agent's task's transcript and composer (`agent-task.html`) wait for
  temper's `client-pages` plan; until then an agent's task shows the
  header and plan only, as a procedure's task does.
- **The forge's parts on temper's pages** are drawn by
  `temper_client_view::forge`'s functions from the parts the documents
  name, with no slot: `ShowPart`, `PartsLeft` and the forge side's held
  parts (`crates/temper-client-domain/src/parts.rs`, `forge.rs`'s
  `Held`) go with the reference UI in temper-07.s1 (ruling 391).
- **The one-process world's goal** (`tests/one-process/src/stories.rs:15-46`
  sets it on jig's project's goals page) is given by a chat in 07.4, as
  the process tier's landing story gives its change
  (`crates/temper/tests/support/person.rs:251-257`, cue `@forge_chat`):
  the person starts a chat with the words `@plan Fix the adopted
  repository` on temper's Chats page, and the run's `@plan` script
  (`tests/one-process/src/stories.rs:109-112`) delegates the change as
  it does today.
- **Pages not about the work:** once jig's reference UI goes, temper's
  main says "Signed out." when the link is signed out (6.6), a phrase
  of temper's words: `jig-client`'s `words` has none for the link's
  states (its `Word` is `Phase`, `Hold`, `Refusal`, `Waiting` and
  `Status`, `crates/jig-client/src/words.rs:15-21`), and jig's are
  `jig_client_view`'s (`crates/jig-client-view/src/words.rs:67-68`), which temper-07.s1 removes. When
  the engine does not serve this client's version of jig's records, the
  main says "This temper is out of date: run the temper the engine
  runs.", also temper's words, with no jig step and no repin (the
  designer's answer to ruling 320; `client.md`, 5.1). Both mains are one
  paragraph with no heading, so the terminal's first line reads `==
  temper == Signed out` or `== temper == Out of date` (`client.md`,
  6.6); while the link is signed out or out of date the banner is
  temper's name and the link's state alone, and no navigation is
  offered (5.1). The banner keeps saying "Connecting" before `me`'s
  first delivery (5.1). The link's states are `jig-client`'s `link()`
  (jig's `client.md`, 8.2).
- **Kept work dropped** (the same answer; `client.md`, 3.4, 6.6):
  `jig-client`'s `Told::Dropped { count }`, which `route.rs:154` gives
  the reference UI's child at the base, goes to the shell in 07.4, and
  the terminal prints `temper: N requests kept in DIRECTORY were not
  sent: another version of temper made them` once.
- **temper's own cards** (the designer's answer of 2026-10-18T19:47Z
  to the question of 19:10Z; `client.md`, 1): the cards temper's pages
  show are temper's nodes after its mockups, drawn from the documents
  and `jig-client`'s `facts`, a decision's facts said in `jig-client`'s
  `words` under temper's phrasebook. No renderer, helper or phrase table
  of `jig-client-view` is called or copied. They come in temper-07.s1,
  the supervisor's to add, after jig-15.s1, with what rulings 391 and
  392 keep.

## Rules that bind it

- **jig's reference UI leaves temper in temper-07.s1:** `git -C
  <worktree> grep -n -e jig-client-view -e jig-domain-client -e jig-view
  -- '*Cargo.toml'` prints nothing at temper-07.s1's end (README, Done
  when, 8), and `git -C <worktree> grep -n -w jig_client_view -- '*.rs'`
  prints nothing.
- **The meeting point:** each page's story runs in `tests/client`
  against temper's engine root in the increment that builds it, with
  jig's check (README, Rules; temper's `domain/testing.md`, 6.3).
- **No repin in this session,** as in temper 06: an increment rebased on
  a repin of temper 05's repairs what breaks in at most 20 lines,
  counted with `git diff --numstat origin/main...HEAD -- <the changed
  paths outside the increment's Files>`; past 20 lines, a question.
  The one exception is temper-07.s1's first commit, which moves
  temper's jig pin alone to jig-15.s1's landing (the designer's answer
  of 19:47Z).

## Increments

### 07.1 The task's page

Scope: `Task { task }`, after `task.html`: for a tracked task its header
(its number, title, executor's kind, project, phase in words, who asked,
its deadline and priority), what was asked, its plan as a tree from
`tree` (chips, folded counts, held below marked), and its latest
activity, the first page of `activity`; `run` watched while one is
live; its parts in the forge's words or "Shown on the forge."; a
procedure's task, a person task and an agent's task show the header and
plan only; a task with the chat label goes to `Chat { task }`; each chip
links to its task's page (`client.md`, 3.5).
Needs: nothing beyond the session's start (temper 06).
Files: `crates/temper-client-domain/src/pages/task.rs` (new), `pages.rs`,
`address.rs` (`Address::Task { task }`); `crates/temper-client-view/src/pages/task.rs`
(new), `forge.rs` (`pub fn drawn(builder, part, words)` split out of
`part`, `:100`, which keeps its `side.contains` guard and calls it:
temper's page, holding no slot, draws a part through `drawn`),
`words.rs`; `crates/temper-client-shell-terminal/src/tests/golden/task.txt`
(re-recorded by hand on temper's page); `tests/client/tests/task.rs`
(new), `tests/client/src/fixtures.rs` (a goal with a plan, beside
`held_chat` at :185).
Shape:
- `pages::task::Task { number: u64, document: Handle, tree: Handle,
  activity: Handle, run: Option<Handle>, shape: Shape }`, the handles
  under asker 2 (a field `task` fails clippy's `struct_field_names`);
  the kind of page (`Shape`: tracked, or header and plan only) decided
  from the task document's `priority` (`jig_client_domain`'s
  `contract::documents::Task`, `documents.rs:109`: `Some` for a tracked
  task, jig's `contract.md`, "tracked | no | yes, with a priority") and
  its chip's executor (`chip.executor`), the chat label from `me`'s
  templates.
- The view: `pages::task::{header, asked, plan, latest}`, each a region
  with its level-2 heading, after `task.html`'s order; the plan a tree
  of items, one per chip, its link named by the task's number.
Test shape: step tests in `crates/temper-client-domain/src/tests/pages/task.rs`
(new), view tests in `crates/temper-client-view/src/tests/task.rs`
(new), the golden, and the joint world. The reference joint-world
story:
```rust
#[test]
fn a_goals_page_shows_its_plan_with_the_held_task_marked() {
    let mut world = fixtures::world(fixtures::SEED);
    let (goal, held) = fixtures::goal_with_held_delegate(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Heading { level: 2 }, b"Plan"), within: fixtures::WITHIN },
            Step::See { find: fixtures::plan_item(held, b"Held"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
```
(`fixtures::goal_with_held_delegate`, `task_page` and `plan_item` are
new in `tests/client/src/fixtures.rs`.) Tests, each under 20 ms (unit)
or 0.5 s (world):
`a_goals_page_shows_its_plan_with_the_held_task_marked`,
`opening_a_task_watches_task_and_tree_and_reads_activity`,
`a_procedures_task_shows_its_header_and_plan_only`,
`a_person_tasks_page_shows_its_header_and_plan_only`,
`a_live_run_on_a_tasks_page_is_watched_until_it_ends`,
`a_plans_chip_links_to_its_tasks_page_by_number`,
`a_task_with_the_chat_label_opens_as_its_chat`,
`a_forge_resource_on_the_page_is_in_the_forges_words`,
`a_forge_panel_on_the_page_says_shown_on_the_forge` (view), and
`a_forge_panel_on_a_goals_page_says_shown_on_the_forge` (world, the
story of `client.md`, 10: "a part of the forge's saying 'Shown on the
forge.'", on a goal whose task document names a forge panel).
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(a_goals_page_shows_its_plan_with_the_held_task_marked)'`
(under 0.5 s): temper's task page shows a goal's plan from temper's
engine root, its held delegate marked.
Don't: offer Amend, Stop or Cancel (`later.md`); show an agent's task's
transcript (`later.md`); compute a task's phase in words other than by
jig's `words` and temper's phrasebook.
Size: at most 300 lines. Ruling 364: it lands whole at up to 650 production
lines with no question; past 650, it blocks with the count by file.
Removes: jig's task page at temper's `Task` addresses.

### 07.2 A held task of the plan released

Scope: on a task's page, a held task of its plan offers "Release task";
pressed, the confirmation says what releasing does, from the held task's
document (its hold, its tries reset, the run started again); confirmed,
one `Release` naming the hold's revision the card showed; answered, the
item shows it running, or the refusal in jig's words (`client.md`, 3.6;
10, "a held task released from a goal's plan").
Needs: 07.1.
Files: `crates/temper-client-domain/src/{pages/task.rs, acts.rs}`;
`crates/temper-client-view/src/{pages/task.rs, confirm.rs, words.rs}`;
`tests/client/tests/task.rs`.
Shape: the plan's held item reads the held task's document under a
handle of the page's (the hold's revision and what releasing does come
from it); `Act::Release { task, revision }` opens 06.3's
`Confirm::Release { task, revision }`, sent by 06.3's `acts::send`.
Test shape: step tests in `crates/temper-client-domain/src/tests/pages/task.rs`,
the joint world in `tests/client/tests/task.rs`. The reference step
test:
```rust
#[test]
fn releasing_from_the_plan_names_the_holds_revision_the_card_showed() {
    let mut pages = fixtures::task_page_with_held(fixtures::GOAL, fixtures::HELD, fixtures::HOLD_REVISION);
    let mut out = Queue::with_capacity(16);
    fixtures::act(&mut pages, &mut out, Act::Release { task: fixtures::HELD, revision: fixtures::HOLD_REVISION });
    fixtures::act(&mut pages, &mut out, Act::Confirm);
    assert_eq!(fixtures::sent(&mut out), [fixtures::release(fixtures::HELD, fixtures::HOLD_REVISION)]);
}
```
Tests, each under 20 ms (unit) or 0.5 s (world):
`releasing_from_the_plan_names_the_holds_revision_the_card_showed`,
`the_release_confirmation_says_what_releasing_does`,
`a_moved_hold_is_refused_and_the_card_asks_to_look_again`,
`a_held_task_released_from_a_goals_plan_runs_again` (world).
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(a_held_task_released_from_a_goals_plan_runs_again)'`
(under 0.5 s): the engine root's held task has its tries reset after
the person's release from temper's task page, jig's check holding.
Don't: release without the confirmation; name the document's newest
revision in place of the card's.
Size: at most 120 lines.
Removes: nothing.

### 07.3 The adoption form as temper's page

Scope: `Adopt { project }` as one of temper's pages, in temper's frame:
the form's nodes of 4.4 in its main, its title naming the project as
`me` names it ("Adopt a repository into shop", `client.md`, 4.4 and
its example in 6.4), where it says jig's "Project: Shop" today
(`JigPhrase::Project`, `forge/page.rs:35`); jig's `after_page` no longer drawn after it; the
navigation's "Adopt a repository" going to it (`client.md`, 4.3 to 4.5,
5.2, 5.3).
Needs: nothing beyond 07.1 (it changes other files; it may land before
07.2).
Files: `crates/temper-client-view/src/{root.rs, forge/page.rs (269
lines: `project_name` at its use of `root.jig()`), forge.rs}`;
`crates/temper-client-domain/src/{route.rs, forge.rs}` (the forge's
address goes to the forge side from temper's `Address::Adopt`, as 06.1
moved it); the goldens `adopt_*.txt` (7 files at the base,
`crates/temper-client-shell-terminal/src/tests/golden/`), whose frame
lines and first line change, by hand: line 1 of each reads `==
Adopt a repository into shop == Live`, as `client.md`, 4.4's example
prints, where it says `Project: Shop`; the shell's
`src/tests/fixtures.rs:133` passes the name alone, `b"shop"`; the view's
`src/tests/forge_page.rs:31` expects "Adopt a repository into shop";
`tests/client/tests/adoption.rs` (its 6 tests keep their names and
expected values). Refresh any further copy of "Project: Shop" this
increment's title changes, and list it in the commit body.
Shape: `temper_client_view::forge::page::page(builder, form, project:
&[u8], words, jig, bindings, text_limit)`, its `jig` words and
`text_limit` kept for `outcome` (`forge/page.rs:198`, its refusals at
`:237` and `:242`), the project's name taken from `me` through the pages
child (`pages::project_name(pages, client, project) -> Option<&[u8]>`),
not from jig's reference UI. For a project `me` does not name (before
its first delivery), the title keeps today's number
(`JigPhrase::FrameProjectNumber`, `forge/page.rs:39`) in this increment;
07.4 replaces it with temper's "Adopt a repository into project N"
(`client.md`, 4.4).
Test shape: the base's form and page tests
(`crates/temper-client-view/src/tests/forge_page.rs`, 343 lines) keep
their names and expected values but the frame's lines and the title;
one new view test, on the view's own fixtures (`view` `:148`, `build`
`:155`, `find` `:159` of `src/tests/fixtures.rs`; temper-client-view
does not depend on skein-fake-person):
```rust
#[test]
fn the_adoption_page_is_titled_with_the_project_me_names() {
    let mut view = fixtures::view();
    let root = fixtures::adoption_page(fixtures::PROJECT, b"shop");
    let tree = fixtures::build(&mut view, &root);
    let title = fixtures::find(tree, Role::Heading { level: 1 }, b"Adopt a repository into shop");
    assert!(title.is_some(), "the title names the project as me names it");
}
```
(`fixtures::adoption_page`, a root on the adoption page whose `me`
names `PROJECT`, is new in `src/tests/fixtures.rs`.)
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(a_repository_adopted_from_the_navigation_link_is_held_by_the_engine)'`
(under 0.5 s): the person adopts from the navigation's link on
temper's page, and temper's engine root holds the repository. And the
one-process world, once (the default suite, no Forgejo): `cargo
nextest run -p temper-one-process-world -E
'test(an_adopted_repository_gets_its_change_pushed_through_git)'`
(about 3 s): the adoption through bytes on temper's page. Nothing else
is run: the plan uses no live service (decisions.md, 10).
Don't: change the form's fields, rules, words or outcome sentences
(4.4, 4.7); read jig's reference UI for the project's name.
Size: at most 120 lines.
Removes: `forge::page::project_name`'s read of jig's reference UI, the
`after_page` call after the form.

### 07.4 `jig-client-view` dropped

Scope: the reference UI's child of temper's root (asker 65536), its
records, places and slots, `Address::Jig`, `ShowPart` and `PartsLeft`
and the forge side's held parts, and `jig-client-view` in temper's
manifests, removed; temper's main for the pages not about the work
(signed out, out of date), the adoption's title for a project `me` does
not name, and the kept work dropped said by the terminal (`client.md`,
3.4, 4.4, 5.1, 6.6); the stories that still visit
jig's pages moved to temper's; the goldens of jig's pages removed or
recorded again by hand (`client.md`, 9; README, Done when, 8).
Needs: 07.1, 07.3 (and 06.2 to 06.6, by the session's start).
Files:
- `crates/temper-client-domain/src/{route.rs, domain.rs, address.rs,
  boundary.rs, limits.rs, owner.rs (`Owner::Jig`), parts.rs (goes),
  forge.rs (`Held`, `Event::Show`, `Event::PartsLeft` go), lib.rs}`,
  `Cargo.toml`; and the users of `Limits.jig` (ruling 385):
  `src/{acts.rs, pages/{chat,chats,inbox,task}.rs, tests/fixtures.rs,
  tests/pages/chats.rs}`;
- `crates/temper-client-view/src/{root.rs, limits.rs, words.rs,
  forge.rs, forge/{bindings,page,words}.rs}` (the outcome's refusals in
  `jig-client`'s words, `Word::refusal`, `crates/jig-client/src/words.rs:105`,
  under temper's phrasebook), `src/tests.rs` (`:29`, `:56`, `:81`,
  `:90-91` hold parts through `fixtures::part_text` and `part_tree`,
  `src/tests/fixtures.rs:118-135`: they draw through 07.1's `drawn`, and
  the test of a part not held goes with the held parts), `Cargo.toml`;
- `crates/temper-client-shell-terminal/src/{limits.rs, terminal.rs,
  client.rs, route.rs, terminal/native.rs, client/tests.rs, tests.rs}`
  (the status region given to
  `skein_ui_text::Settings`, `terminal.rs:59` and `tests.rs:93` at the
  base, wherever 06.1 left them, names temper's region alone, no region
  of jig's), `src/tests/fixtures.rs` (`:18`, `:439`, `:564`),
  `src/tests/render.rs:40-52`, `tests/support/mod.rs`, `Cargo.toml`,
  and the goldens of jig's pages: `project.txt` and `deployment.txt` go
  with their tests; `signed_out.txt` recorded again on temper's main
  (by hand), its first line `== temper == Signed out` (`client.md`,
  6.6);
- `tests/client/src/fixtures.rs:262` (`project()`, an address of jig's,
  becomes `Address::Inbox`, the stories pressing the navigation's
  "Adopt a repository" there) and its other users of jig's reference
  UI, `tests/client/src/{app,translate}.rs`, `tests/client/Cargo.toml`;
- `tests/one-process/src/stories.rs:15-46` (the goal given by a chat,
  Choices; the adoption's steps from temper's navigation),
  `tests/one-process/src/world.rs:213`, `Cargo.toml`;
- `crates/temper/tests/support/person.rs:20` and `:231-279`
  (`adopt_and_change` and `found` go to temper's addresses),
  `crates/temper/Cargo.toml`;
- the workspace `Cargo.toml` and `Cargo.lock`.
  Each of these that 06.x has already moved is changed where 06.x left
  it.
Users found by `git grep -n -w -e jig_client_view -e 'Address::Jig' -e
ShowPart -e PartsLeft -- '*.rs'` and `git grep -n jig-client-view --
'*Cargo.toml'` in temper.
Shape: no new type but temper's main for the link's states, and one
variant for the dropped count:
- `pages` shows "Signed out." or "This temper is out of date: run the
  temper the engine runs." (temper's words, one paragraph, no heading)
  when `jig-client`'s `link()` says signed out or out of date, in place
  of the page at the address; the banner then holds temper's name and
  the link's state ("Signed out", "Out of date") alone, with no
  navigation (`client.md`, 5.1).
- The adoption's title for a project `me` does not name is a row of the
  forge's words beside `Word::Title` (`forge/words.rs:65`): "Adopt a
  repository into project " and a number hole, in place of
  `JigPhrase::FrameProjectNumber` (`client.md`, 4.4).
- `Told::Dropped { count }` leaves the domain as a request of its own
  (`Request::Dropped { count }`, `boundary.rs`), which the terminal's
  `route::down` passes on (its `route::Notice::Dropped`, `route.rs:12`,
  is made nowhere at the base), and the host says as one of temper's
  lines, beside `OtherPartyKept` (`terminal.rs:31`, `:175`): `temper: N
  requests kept in DIRECTORY were not sent: another version of temper
  made them` (`client.md`, 6.6).
Each match on the root's children loses the reference UI's
arm; `Address` loses `Jig`.
Test shape: the view tests of the two mains in
`crates/temper-client-view/src/tests/frame.rs` (06.1's file), on its
fixtures; the reference test:
```rust
#[test]
fn signed_out_the_main_says_so_and_nothing_else() {
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &fixtures::signed_out());
    assert_eq!(&*fixtures::main_text(tree), b"Signed out.");
}
```
(`fixtures::signed_out`, a root whose link is signed out, and
`main_text`, the text of the main's nodes alone, beside `text` at
`src/tests/fixtures.rs:94`, are new; `fixtures::built` takes no root.)
Tests, each under 20 ms (unit) or 0.5 s (world):
`signed_out_the_main_says_so_and_nothing_else`,
`an_engine_not_serving_jigs_version_says_temper_is_out_of_date` (the
main's text is "This temper is out of date: run the temper the engine
runs.", and the banner offers no "Sign out" and no navigation),
`the_adoption_page_for_a_project_me_does_not_name_is_titled_by_its_number`
(view: "Adopt a repository into project 7"),
`kept_work_of_another_version_prints_its_line_once` (the terminal's
host, beside `another_partys_kept_work_prints_its_line_once`,
`src/client/tests.rs:10`),
`a_client_closed_on_a_tasks_page_opens_at_the_inbox_again` (world:
reloaded with nothing of the page kept, `client.md`, 3.4). The stories
moved keep their names and expected values, but the addresses and node
names temper's pages give (an expected literal the Shape renames).
Proof: the one-process world, once: `cargo nextest run -p
temper-one-process-world -E
'test(an_adopted_repository_gets_its_change_pushed_through_git)'`
(about 3 s): the person adopts on temper's adoption page and gives the
change by a chat on temper's Chats page, and the fake forge receives
the push. And the process tier, once, with 05.1's command and the
filter `test(temper_open_signs_in_prints_the_inbox_and_exits_on_q)`:
`temper open` prints temper's inbox with no page of jig's (about 1
minute). Nothing else is run.
Don't: keep an alias, a re-export or a feature flag of the reference
UI; keep a golden of a page temper no longer has.
Size: at most 140 lines added (the removals are not counted): 120,
and 20 for the title's row and the dropped count's line (the designer's
answer to ruling 320). Ruling 388: with the native replacements of the
reference UI's renderers, it lands whole, with no question; past the
stop, block with the count. Ruling 389: the stop is 2,000 production
lines added, what is left counted before more code is written, and
jig's enums said in `jig-client`'s words, not copied. Ruling 390
(interim, until the designer answers the question of 19:10Z): 07.4
lands now without the card renderers' replacement, at up to 400
production lines added; temper's calls into `jig-client-view`'s card
renderers, and `jig-client-view` in `temper-client-view`'s manifest for
them alone, stay until the step the answer gives, which meets Done
when's two greps. Ruling 391: the reference UI's
child stays with them, and what names it, since the renderers borrow it
(`Ui::borrow`); 07.4 lands now only what compiles with the child kept.
Ruling 392: so does any other use of `jig-client-view` that 07.4 cannot
drop without the child, the renderers or a change in jig (the test
transport of `jig-client-world` takes `jig_client_view::Limits`),
listed in the report; the answer's step drops them. The designer's
answer of 2026-10-18T19:47Z: 07.4 ends with what rulings 390 to 392 let
it land. The card renderers' replacement, by temper's own cards, and
the removals below go to temper-07.s1. Ruling 388's point 3 and ruling
389's points 2 and 3 no longer bind the cards, since nothing of
`jig-client-view` is copied.
Removes (in temper-07.s1, the designer's answer): the transitional code decisions.md, 7 names: the reference
UI's child, `owner::Owner::Jig` and `Address::Jig` (from temper 03.6
and 06.1), `parts.rs`,
the forge side's held parts, `jig-client-view` in temper's manifests.

## Interface sketches

None beyond temper 06's (`Address`, `Confirm`, the pages child): 07.1
adds a page module of the same shape as 06's.

```rust
// crates/temper-client-domain/src/pages/task.rs (07.1), checked at the ready check
/// A task's page: the handles under asker 2, and what kind of page the task gets.
pub struct Task { number: u64, document: Handle, tree: Handle, activity: Handle, run: Option<Handle>, shape: Shape }
/// What the task's page shows (client.md, 3.5).
pub enum Shape {
    /// A tracked task: header, what was asked, plan, latest activity.
    Tracked,
    /// A procedure's task, a person task, or an agent's task (later.md): header and plan only.
    HeaderAndPlan,
    /// Not known until the task's document comes.
    Unknown,
}
```

## Questions to raise when reached

- If the one-process world's chat cannot delegate the `@plan` change
  under the chat template's authority (07.4's Choices), the goal needs
  "Set a goal" (Board, `later.md`): raise it, with the refusal the
  engine gave.
- The design gives an agent's task `agent-task.html` (3.5), which
  `later.md` defers; if a story opens an agent's task's page and needs
  its transcript, raise it.

## Done when

- 07.1 to 07.4 and temper-07.s1 have landed on temper's `origin/main`.
- `git -C <worktree> grep -n -e jig-client-view -e jig-domain-client -e
  jig-view origin/main -- '*Cargo.toml'` prints nothing (README, Done
  when, 8).
- `git -C <worktree> grep -n -w -e jig_client_view -e 'Address::Jig'
  -- '*.rs'` prints nothing.
