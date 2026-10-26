# Session 06: the frame, the inbox and the chats, temper's own

temper's pages stop being jig's reference UI. A child of temper's client
root, its pages, owner `Pages` (2), holds what temper shows of jig's
work, on `jig-client` and jig's helpers alone: the frame and its
navigation, after the mockups' top bar; the inbox, home; the chats and
their composer; and a chat, its conversation, the proposals waiting in
it, decided in place, Stop and Close. Each act is one keyed request
through `jig-client`, made with jig's helpers where they fit, said
before it is sent where it cannot be undone, and naming the revision the
person saw (`client.md`, 3.5, 3.6, 5). Each page's story runs in
`tests/client` against temper's engine root from the increment that
builds it. Pages not yet temper's (a task's page, the project's pages
some stories still visit) keep showing jig's reference pages inside
temper's frame until temper 07.

## Read first

- temper's `docs/design/client.md`: 1, 3 (3.1 to 3.6), 5 (5.1, 5.3,
  5.4), 6.4, 10 (the bullets "Step tests", "Golden text" and
  "`tests/client`").
- temper's `docs/design/web/mockups/`: `inbox.html`, `chats.html`,
  `index.html` (a chat), `style.css` for the order of each page's parts;
  `docs/design/web/ux/README.md`, 4 (links by number) and 5.3 (said
  before it is sent).
- temper's `docs/design/domain/testing.md`, 6.3.
- jig's `docs/design/client/domain/client.md`, 4 and 19;
  `client/domain/root.md`, 3, 5 and 15; `client/domain/contract.md`, 3.4
  to 3.6, 5.1 to 5.4, 6.1, 7.1; `client/domain/view.md`, 4.1 (the
  reference UI's pages as functions) and 18.1 (`jig_client_view::Ui`'s
  surface, the pattern temper's pages child copies).
- skein's `docs/design/ui.md`: 3.3 (owners), 6 (bindings), 8 (roles and
  names), 14 (the outline).
- This plan's `jig/15-helpers.md` (`shapes`, `facts`, `thread`, `words`:
  their signatures; 15.5, jig's check in `jig-client-test-kit`), `jig/13-reference-ui.md` (13.2's module names in
  `jig-client-view`: `page` for `pages`) and `jig/21-ops-and-ui.md`
  (21.3's `Ui`; the page functions, view.md, 4.1's `pages::main` and
  `after_page`); `decisions.md`, 1c and 7;
  `later.md` (what these pages leave out).
- skein's `programming-model.md`: 3, 5.1, 5.2, 6.3, 7 (by number only);
  skein's `testing-strategy.md`: 2.1, 2.2, 6, 7 (by number only).

## What the design asks

- **The root's children** (`client.md`, 3.1): "temper's pages, owner
  `Pages`, which ask the client for jig's operations, reads and subjects
  only; and the forge's client side, owner `Forge`"; "`me`, watched from
  start to sign-out under a handle of its own"; "The page at its
  address ...: its handles, its windows, the composer's words, and the
  open confirmation (3.6), at most one"; "The project the person is in,
  chosen among `me`'s projects; the first at start"; "The pages' acts in
  flight, each by its key".
- **The pages** (`client.md`, 3.5, the rows `Inbox`, `Chats` and `Chat {
  task }`): their mockups, what each shows, watches and reads, and its
  acts, as the table gives them.
- **Requests and confirmations** (`client.md`, 3.6): "One act, one
  request ... A double press joins the pending one. Nothing is shown done
  before its answer"; "starting a chat is the chat template filled in
  (`shapes::chat`)"; "closing a chat is one compound request, with what
  becomes of each live delegate (`shapes::close_chat`)"; "An act that
  cannot be undone (cancelling, closing a chat, accepting, rejecting)
  says what it will do before it is sent"; "Each deciding act names the
  revision of the document the card showed."
- **The view** (`client.md`, 5.1): "a banner: temper's name; the project
  the person is in, and the others to choose from; the link's state
  ('Connecting', 'Live', 'Offline'), and 'Behind' while a page's watch
  reopens; the person's name and 'Sign out'"; "Every node a person acts
  on has a role and a name".
- **Navigation** (`client.md`, 5.3): "'Inbox', with the count of what
  needs the person from `me`; 'Chats'; then, for the project the person
  is in, 'Board' and 'Changes'; and after them 'Adopt a repository'".
  Board and Changes are `later.md`'s: this session offers neither, and
  says so in the session's report.

## Choices this plan makes

- **The pages child copies `jig_client_view::Ui`'s surface** (jig's
  `view.md`, 18.1): stepped by the root with the person's acts, the
  places gone to and what the client tells under its asker; it asks the
  client through the root (`Watch`, `Unwatch`, `Read`, `Send`), and reads
  the client only through the `&Client` the root lends it within the
  step. It keeps no copy of a document across steps.
- **Transitional, until temper 07.4** (decisions.md, 7): temper's
  `Address` keeps `Jig(address)` for the reference pages not yet temper's;
  at such an address the main is `jig-client-view`'s `main` (view.md,
  4.1's `pages::main`, in the module jig 13.2 names `page`) and
  `after_page`, inside temper's frame, and the reference UI child (asker
  65536) keeps routing as 03.6 left it. Nothing else of jig's reference
  UI is shown once 06.1 lands: its frame, banner and navigation go.
- **The names the fake person finds** (`ui.md`, 8), fixed here so that
  the stories and the goldens agree, in the mockups' words where they
  give one: the banner's status "Live", "Connecting" or "Offline", a
  button "Sign out"; the navigation's links "Inbox", "Chats", "Adopt a
  repository"; the inbox's buttons "Accept", "Reject with reason",
  "Answer", "Release task", "Leave held", "Take", "Hand back"; the
  chats' text area "Start a chat" and button "Start"; a chat's text area
  "Message", buttons "Send", "Stop", "Close"; a confirmation's dialog,
  named by what it will do, with buttons "Confirm" and "Cancel"
  (Escape), the names the reference UI's confirmation has today.
- **A deciding act's revision comes from the tree.** The binding of a
  deciding node carries the revision of the document the card was built
  from; the domain sends that one, never the document's newest (jig's
  check, `client.md`, 10). Deciding a proposal names no revision: "A
  proposal never changes once made, so deciding it names its number"
  (jig's `client/domain/contract.md`, 5.2; `Ask::DecideProposal`, jig
  `crates/jig-client-domain/src/contract/asks.rs:135`).
- **Chats:** the live chats are a watch, `tasks{query}` (requester the
  person, label the chat template's from `me`, by last activity); the
  closed ones a read of the query's ended group, a page at a time
  (contract.md, 6.1: "Live first, then ended, in two groups").
- **Golden text** for temper's pages is re-recorded by hand in the
  increment that changes a page (`client.md`, 10): `inbox.txt` in 06.2,
  `chats.txt` in 06.4, `chat.txt` in 06.5, in
  `crates/temper-client-shell-terminal/src/tests/golden/`.

## Rules that bind it

- **The meeting point:** each page's story runs in `tests/client`, the
  joint world, against temper's engine root, in the increment that
  builds the page; none runs only against `jig-client-scripted-engine`
  (README, Rules, "The meeting point"; temper's `domain/testing.md`, 6.3).
- **jig's check holds in the stories of `tests/client`** (`client.md`,
  10; temper's `domain/testing.md`, 6.3): jig 15.5's check from
  `jig-client-test-kit` runs in the world's referee from 06.1 on, beside
  `jig-client-world`'s own rules, and no story turns it off; its verdict
  is the story's.
- **No repin in this session.** temper 04.3 pinned jig past jig 15's
  helpers; no increment here names a newer jig capability. If temper
  05's repin lands first, an increment rebases onto it and repairs what
  the rebase breaks in at most 20 lines (README, Rules, "a fix the
  change ... needs"), counted with `git diff --numstat
  origin/main...HEAD -- <the changed paths outside the increment's
  Files>`; past 20 lines, a question.

## Increments

### 06.1 The pages child, temper's frame and navigation

Scope: `Owner::Pages`'s child of the root, its `me` handle, the project
the person is in, temper's `Address` (`Inbox`, `Chats`, `Chat { task }`,
`Adopt { project }`, and the transitional `Jig(address)`), routing by
asker 2 to it; temper's banner and navigation around each page, after
the mockups' top bar: the link's state ("Connecting" until `me`,
"Live", "Offline" when the link drops) and "Behind" after it while a
page's handle is behind, the page printed again at each change; the
projects to choose from; the person's name and "Sign out", a sign-out
that did not reach the engine said so and offered again (`client.md`,
3.1, 3.3, 3.4, 5.1, 5.3, 6.4, 6.6). Leaving a page closes its handles;
a client started again opens at the inbox, with nothing kept. Until
06.2 and 06.4, `Inbox` and `Chats` show jig's reference pages for
those places.
Needs: nothing beyond the session's start (temper 04).
Files: `crates/temper-client-domain/src/{pages.rs (new), address.rs,
route.rs, domain.rs, boundary.rs, limits.rs}`;
`crates/temper-client-view/src/{frame.rs (new), root.rs, words.rs,
limits.rs}`; `crates/temper-client-shell-terminal/src/limits.rs` (the
pages' limits in the sum) and `src/terminal.rs:58-62` (the status
region given to `skein_ui_text::Settings` moves to temper's banner's
link region, which holds "Behind" after the link's state, `client.md`,
6.4), with the other users of `JigRegion::Link`: `src/tests.rs:186`,
`src/tests/fixtures.rs:112, 372` and `tests/support/mod.rs:30, 127`
(`terminal.rs` alone fails `fuzzy_terminal`: "seed 1: last change
printed"); `crates/temper/src/open.rs` builds no `Settings` and does not
change; the goldens whose banner and navigation change
(`crates/temper-client-shell-terminal/src/tests/golden/*.txt`, 14 at
the base, by hand), the trees they print, built by hand in
`src/tests/fixtures.rs` (`inbox_page()` at :342), and the list of pages
`src/tests/render.rs:40-52` compares; `tests/client/src/fixtures.rs` (`inbox()` at
:118, `conversations()` at :140 and `project()` at :257 name temper's
`Address`), `tests/client/tests/stories.rs:8-21` (the banner's names:
"Signed in as Person" at :15 becomes the person's name, an expected
literal the Shape renames); the process tier's terminal script,
`crates/temper/tests/process.rs:34-63` (`[4] Project 1` at :44 and the
navigation's numbers: the person types on "Adopt a repository" in
temper's navigation) and `crates/temper/tests/support/terminal.rs`;
`tests/one-process/src/stories.rs:15-55` and `src/world.rs:209-222`
(`Address::Jig(..)` becomes temper's `Address::Jig(..)`, unchanged in
meaning). Users of temper's `Address` found by `git grep -n -w -e
'Address::Jig' -e 'Address::Forge' -- '*.rs'` in temper.
The navigation's configuration, which 06.1 removes, and its users,
unless 03.6 has moved them: the domain's `src/lib.rs:20` and
`src/tests.rs:204-218`, the shell's `src/client.rs:130` and
`src/tests/fixtures.rs:291-292`. The root `Cargo.toml`
(`jig-client-test-kit` in `[workspace.dependencies]`, which
`tests/client/Cargo.toml` takes); the new modules' declarations in
`crates/temper-client-domain/src/{lib.rs, tests.rs}` and
`crates/temper-client-view/src/{lib.rs, tests.rs}`; the fixtures the
tests below name, in each crate's `src/tests/fixtures.rs`.
Shape:
- `temper_client_protocol_wire::Owner::Pages` (number 2, since 03.5)
  given its child: the root's `owner::Owner` gains `Pages` (`asker`
  2, `of` it), and the root's match routes it to `pages::step`.
- `temper_client_domain::pages::{Pages, Event, Request, step, fire,
  next_deadline, shown}`, after `jig_client_view::Ui` (the sketch
  below); `Event::Went { address }`, `Event::Act { act }`, `Event::Told
  { told }` (what the client tells under asker 2), `Event::Link { link
  }`; `Request::Client(..)` for the client, `Request::Show { address }`.
- `temper_client_domain::Address` (the sketch below): `Inbox { filter
  }`, `Chats`, `Chat { task }`, `Adopt { project }`, `Jig(address)`,
  jig's `jig_client_view::address::Address` (transitional, removed by
  07.4). The forge's `Adopt` keeps its child
  and its form; only its address moves into temper's enum.
- `tests/client`'s referee runs jig 15.5's check (`jig-client-test-kit`,
  as jig 15.5's Shape names its entry point) on each story's sent
  decisions and the trees the person acted on, wired once in
  `tests/client/src/front.rs` and `app.rs`; `tests/client/Cargo.toml`
  takes `jig-client-test-kit`.
- `temper_client_view::frame::{banner, navigation}`: the banner's
  regions in the order of `client.md`, 5.1; the navigation's links in
  the order of 5.3, with "Inbox" and its count from `me`'s needs, the
  project's entries only when a project is chosen, "Adopt a repository"
  whatever the person's role (5.3).
- `pages::Act` (in `pages.rs`, deriving `Debug`): `Act::Project {
  project }` (one of `me`'s projects) changes the
  project the person is in; `Act::SignOut` gives the client `SignOut`;
  while `client.link().sign_out_lost`, the banner says the sign-out did
  not reach the engine and offers "Sign out" again (jig's `client.md`,
  12).
- Going to another address unwatches each handle of the page left;
  what the client tells of a handle the pages no longer hold is
  dropped. The pages keep no record: at start the address is `Inbox`.
Test shape: step tests in `crates/temper-client-domain/src/tests/pages.rs`
(new), on `crates/temper-client-domain/src/tests/fixtures.rs` (its
limits from the shipped profile as the fixtures module takes them
today); view tests in `crates/temper-client-view/src/tests/frame.rs`
(new), reading the tree with the view's fixtures (`find`, and `link_names`
below); and the
joint world. The reference step test:
```rust
#[test]
fn the_pages_watch_me_from_start_under_their_own_asker() {
    let mut pages = fixtures::pages();
    let mut out = Queue::with_capacity(16);
    pages::step(&mut pages, &fixtures::env(&fixtures::limits()), &fixtures::client(), pages::Event::Started, &mut out);
    let watches = fixtures::watches(&mut out);
    assert_eq!(watches, [(fixtures::asker(2), fixtures::ME)], "one watch of me, asked by Pages");
}
```
The reference view test:
```rust
#[test]
fn the_navigation_offers_inbox_chats_and_adoption_in_order() {
    let view = fixtures::signed_in_with_project();
    assert_eq!(fixtures::link_names(&view), ["Inbox 0", "Chats", "Adopt a repository"]);
}
```
`fixtures::link_names` (new) collects the names of the tree's
`Role::Link` nodes, beside `fixtures::find`
(`crates/temper-client-view/src/tests/fixtures.rs:155`): skein's
`TreeFace` has no `names`, and temper-client-view takes no
`skein-fake-person`. `signed_in_with_project` (new) builds the view as
the landed `built(root_forge: u16)` (:132) does.
Tests, each under 20 ms: the two above,
`an_asker_naming_pages_reaches_the_pages_child`,
`the_project_chosen_is_the_first_of_me_at_start`,
`choosing_another_project_changes_the_navigations_project`,
`the_banner_says_the_links_state_and_the_persons_name`,
`the_banner_says_connecting_until_me_then_live_then_offline`,
`a_page_behind_says_behind_after_the_links_state`,
`sign_out_gives_the_client_one_sign_out`,
`a_sign_out_that_did_not_reach_the_engine_is_offered_again`,
`leaving_a_page_unwatches_its_handles_and_drops_their_tells`,
`a_client_started_again_opens_at_the_inbox`,
`no_project_chosen_offers_no_adoption_link`,
`adopt_a_repository_is_offered_whatever_the_persons_role`; the
terminal's machine world: `a_link_state_change_prints_the_page_again`.
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(signing_in_shows_the_inbox_from_the_engine_root)'`:
the person signs in and sees temper's banner ("Live", the person's
name) and the inbox's heading against temper's engine root (under
0.5 s). And the process tier, once, with 05.1's command and the filter
`test(temper_open_signs_in_prints_the_inbox_and_exits_on_q)`: `temper
open` prints temper's frame (about 1 minute).
Don't: offer Board, Changes or Tasks in the navigation; keep jig's
banner or navigation on any page; read a document outside the step.
Size: at most 300 lines.
Removes: `jig_client_view`'s frame from temper's view (`root.rs`'s use
of jig's default arrangement), the navigation's configuration of jig's
three destinations (`temper_client_domain::navigation`).

### 06.2 The inbox, home

Scope: `Inbox { filter }`, home: what needs the person across projects,
by kind (proposals, questions, held tasks, person tasks), then updates
(results, replies); filtered by project, kind and goal; each entry's
card reading the object it needs; links by number; seen
(`client.md`, 3.5, the row `Inbox`; `inbox.html`). No deciding act yet:
06.3 adds them.
Needs: 06.1.
Files: `crates/temper-client-domain/src/pages/inbox.rs` (new),
`pages.rs`; `crates/temper-client-view/src/{pages.rs (new),
pages/inbox.rs (new)}`, `root.rs`, `words.rs`; `crates/temper-client-shell-terminal/src/tests/golden/inbox.txt`
(re-recorded by hand, with an entry of each kind and an update, as
`client.md`, 10's golden text lists it), and its tree, `inbox_page()`
(`src/tests/fixtures.rs:342`), by hand; the fixtures its tests name
(`question` among them), in each crate's `src/tests/fixtures.rs`; `tests/client/tests/stories.rs` and the
process tier's `crates/temper/tests/process.rs:34-63` (the inbox's
printed lines: `All goals` at :43 becomes temper's goal filter's name).
Shape:
- `pages::inbox::Inbox { filter: Filter, entries: Handle, cards:
  List<Card> }`: the `inbox` watch under one handle with the filter; for
  each shown entry, a read of its object (`proposal`, `question`,
  `task`, a person task, a result) under a handle of its own, read again
  when the inbox says it moved (contract.md, 7.1).
- `Filter { project: Option<u32>, kind: Option<Kind>, goal: Option<u64>
  }`, `Kind { Proposal, Question, Held, PersonTask }` (the sketch
  below).
- `Act::Filter { filter }`, `Act::Seen` (jig's `Seen` over results, one
  keyed request).
Test shape: step tests in `crates/temper-client-domain/src/tests/pages/inbox.rs`
(new), view tests in `crates/temper-client-view/src/tests/inbox.rs`
(new), the golden `inbox.txt`, and a joint-world story in
`tests/client/tests/stories.rs`. The reference joint-world story:
```rust
#[test]
fn a_question_needing_the_person_is_shown_first_in_the_inbox() {
    let mut world = fixtures::world(fixtures::SEED);
    fixtures::question(world.front(), b"Which provider first?");
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Heading { level: 2 }, b"Needs you"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"Which provider first?"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
```
(`fixtures::question` is new in `tests/client/src/fixtures.rs`, beside
`held_chat` at :180, which it follows.) Tests, each under 20 ms (unit)
or 0.5 s (world): `a_question_needing_the_person_is_shown_first_in_the_inbox`,
`the_inbox_watches_inbox_and_reads_each_entrys_object`,
`entries_show_proposals_questions_held_and_person_tasks_then_updates`,
`a_filter_by_project_shows_only_that_projects_entries`,
`a_filter_by_kind_shows_only_that_kind`,
`a_filter_by_goal_shows_only_entries_below_it`,
`an_entry_naming_a_task_links_to_its_page_by_number`,
`marking_seen_sends_one_seen_over_results`.
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(a_question_needing_the_person_is_shown_first_in_the_inbox)'`
(under 0.5 s): a question the engine root holds is shown in temper's
inbox. Nothing else is run.
Don't: decide anything on a card (06.3); count what needs the person in
the client (it is `me`'s count).
Size: at most 300 lines. Ruling 330 lets the whole inbox land at 607
production lines, as counted at 2026-10-18T08:32Z, with no split.
Removes: jig's inbox page at temper's home (the `Jig(Inbox ..)` place in
temper's navigation).

### 06.3 The inbox's acts, said before they are sent

Scope: the inbox's acts, each one keyed request: accept and reject with
a reason (a proposal), answer (a question), release and leave held (a
held task), take and hand back (a person task); accepting, rejecting
and releasing said before they are sent, from the facts the document
gives (what accepting grants and costs and the pool after it, what
releasing does); each naming the revision the card showed; a
proposable or refused act said so on its card; answered, the card
shows the outcome or the refusal in jig's words (`client.md`, 3.5,
3.6; `web/ux/README.md`, 5.3).
Needs: 06.2.
Files: `crates/temper-client-domain/src/{acts.rs (new), pages.rs,
pages/inbox.rs}`; `crates/temper-client-view/src/{confirm.rs (new),
pages/inbox.rs, bindings.rs (new), words.rs}`; tests below;
`tests/client/tests/stories.rs` (`a_held_chats_escalation_released_from_its_task_page`
moves to the inbox: its name becomes
`a_held_chats_escalation_released_from_the_inbox`, its assertion on
`Tries::NONE` kept).
Shape:
- `temper_client_domain::acts::Confirm` (the sketch below), at most one
  open on a page; `Act::Confirm` sends it, `Act::Cancel` drops it; what
  it says comes from `facts::effect(offers, act)` and
  `facts::dangerous(act)` (jig 15.1) over the card's document.
- Each deciding binding carries `(object, revision)` as the card was
  built (Choices; a proposal's, its number alone); `acts::send(.., operation, revision)` keys the
  request with the pages' seed and asks the client (`Send { key, request
  }`), keeping the key with the card that sent it.
- A card's state: open, deciding (pending, shown as asked from
  `client.pending(key)`: in flight, retrying, or parked while the link
  is not live, `client.md`, 6.7), not sent (`Outcome::NotKept`: the
  small store was not written, 6.7), refused (jig's words), unknown
  (`Outcome::Unknown`: past the engine's retention, 3.6), leaving
  (decided, by whom), after jig's `objects.md`, 2, as the reference
  UI's cards have it. Opening a confirmation drops the one open, so a
  page holds at most one (3.1).
Test shape: step tests in `crates/temper-client-domain/src/tests/acts.rs`
(new), view tests in `crates/temper-client-view/src/tests/confirm.rs`
(new), joint-world stories in `tests/client/tests/inbox.rs` (new). The
reference step test:
```rust
#[test]
fn accepting_sends_nothing_until_the_confirmation_is_confirmed() {
    let mut pages = fixtures::inbox_with_proposal(fixtures::PROPOSAL);
    let mut out = Queue::with_capacity(16);
    fixtures::act(&mut pages, &mut out, Act::Accept { proposal: fixtures::PROPOSAL });
    assert_eq!(fixtures::sent(&mut out).len(), 0, "said before it is sent");
    fixtures::act(&mut pages, &mut out, Act::Confirm);
    assert_eq!(fixtures::sent(&mut out), [fixtures::decide_accept(fixtures::PROPOSAL)]);
}
```
Tests, each under 20 ms (unit) or 0.5 s (world):
`accepting_sends_nothing_until_the_confirmation_is_confirmed`,
`the_accept_confirmation_says_what_it_grants_costs_and_leaves_in_the_pool`,
`rejecting_says_so_with_its_reason_before_it_is_sent`,
`an_act_only_proposable_says_so_and_to_whom`,
`a_decision_names_the_revision_its_card_showed`,
`a_double_press_of_confirm_sends_one_request`,
`a_page_holds_at_most_one_open_confirmation`,
`each_inbox_act_is_one_keyed_request` (a table test, its rows answer,
leave held, take and hand back),
`an_act_made_offline_shows_parked_until_the_link_is_live`,
`a_request_not_stored_says_not_sent_on_its_card`,
`an_answer_unknown_past_the_retention_says_so_on_the_card`,
`a_refused_act_shows_jigs_words_and_sends_nothing_more`,
`releasing_says_what_releasing_does_before_it_is_sent`,
`a_held_chats_escalation_released_from_the_inbox` (moved, world).
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(a_held_chats_escalation_released_from_the_inbox)'`
(under 0.5 s): the person releases a held chat from temper's inbox after
the confirmation, and the engine root's task has its tries reset, with
jig's check holding.
Don't: compute what the person may do (the document's offers say it);
send a deciding act from a tree that did not show its facts; keep the
confirmation across pages.
Size: at most 300 lines. Ruling 345: 603 production lines, the proof's two
repairs (ruling 340) among them; with the gate's repairs, it stops at 650.
Removes: nothing.

### 06.4 The chats, and starting one

Scope: `Chats`: the composer starting a chat with `shapes::chat`; the
person's chats, live then closed, newest activity first, each with its
status from jig's facts, its spend and its delegates; each a link to
its chat
(`client.md`, 3.5, the row `Chats`; `chats.html`).
Needs: 06.1.
Files: `crates/temper-client-domain/src/pages/chats.rs` (new),
`pages.rs`; `crates/temper-client-view/src/pages/chats.rs` (new),
`words.rs`; `crates/temper-client-shell-terminal/src/tests/golden/chats.txt`
(new), with its tree in `src/tests/fixtures.rs` and its entry in
`src/tests/render.rs`'s list; the stories that start a chat on jig's page, each moved to
temper's names (an expected literal the Shape renames):
`tests/client/tests/stories.rs` (`a_double_press_of_start_makes_one_chat`,
`a_chat_started_on_jigs_page_is_answered_by_the_run`, renamed
`a_chat_started_on_the_chats_page_is_answered_by_the_run`,
`signed_out_mid_request_is_made_once_after_signing_in`),
`tests/client/src/fixtures.rs:140` (`conversations()` becomes
`chats_page()`: `chats(&Driver)` at :144 keeps its name), with its users
`tests/client/tests/stories.rs:26, 45, 80` and
`tests/client/tests/fuzzy_client.rs:10`;
`tests/one-process/src/world.rs:209-222`,
`crates/temper/tests/support/person.rs:16-27, 248-255` ("First words"
and "Start a conversation" become "Start a chat" and "Start").
Shape:
- `pages::chats::Chats { live: Handle, closed: Handle, words:
  Box<[u8]>, start: Option<Key> }`: `live` watches `tasks{query}`
  (requester the person, label the chat template's label from `me`'s
  merged templates of the project the person is in, ordered by last
  activity); `closed` reads the same query's ended group a page at a
  time.
- `Act::ChatWords { words }` (bounded by the composer's bytes),
  `Act::StartChat`: `shapes::chat(me, project, words) -> Result<Ask,
  NotOffered>` (jig 15.3), one keyed request; `NotOffered` shown in
  jig's words, nothing sent; a double press joins the pending key.
- Each row from its task line (jig 22.2): its status from
  `facts::chat_status` over the task line (jig 15.1, which takes a task
  line as well as a task document), said through `jig_client::words`
  (`Word::Status`) under temper's phrasebook ("Waiting for you",
  "Writing", "Parked", "Waiting for news"), its spend and delegates.
Test shape: step tests in `crates/temper-client-domain/src/tests/pages/chats.rs`
(new), view tests in `crates/temper-client-view/src/tests/chats.rs`
(new), the golden, and the moved stories. The reference step test:
```rust
#[test]
fn starting_a_chat_sends_the_chat_template_filled_once() {
    let mut pages = fixtures::chats_page(fixtures::PROJECT);
    let mut out = Queue::with_capacity(16);
    fixtures::act(&mut pages, &mut out, Act::ChatWords { words: Box::from(*b"Hello") });
    fixtures::act(&mut pages, &mut out, Act::StartChat);
    fixtures::act(&mut pages, &mut out, Act::StartChat);
    assert_eq!(fixtures::sent(&mut out), [fixtures::chat_create(fixtures::PROJECT, b"Hello")], "one request, jig's shapes::chat");
}
```
(`fixtures::chat_create` builds its expected value from `shapes::chat`
over the fixture's `me`, jig's helper being the oracle, not temper's
code.) Tests, each under 20 ms (unit) or 0.5 s (world):
`starting_a_chat_sends_the_chat_template_filled_once`,
`the_chats_watch_live_lines_by_last_activity`,
`closed_chats_are_read_a_page_at_a_time`,
`a_chats_row_says_its_status_from_jigs_facts`,
`a_chats_row_says_its_spend_and_delegates`,
`a_chats_row_links_to_its_chat_by_number`,
`a_double_press_of_start_makes_one_chat` (moved, world),
`a_chat_started_on_the_chats_page_is_answered_by_the_run` (moved, world).
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(a_chat_started_on_the_chats_page_is_answered_by_the_run)'`
(under 0.5 s). And the one-process world, once: `cargo nextest run
-p temper-one-process-world -E
'test(a_person_signs_in_by_the_printed_link_and_a_chat_is_answered)'`
(about 1 s): the person starts the chat on temper's page through bytes,
and the run answers it.
Don't: build the `Create` by hand (jig's `shapes::chat` builds it);
filter the chats in the client beyond the query.
Size: at most 250 lines.
Removes: jig's conversations page at temper's `Chats`.

### 06.5 A chat: its conversation, the composer, Stop

**Ruling 372:** 06.5 needs temper-05.s2 and jig-17.s3 too. Its candidate
(`0650656cc`) is rebased on 05.s2's landing; then its first commit is the
repin onto jig-17.s3's landing (README, "Pins"; "What a temper repin
repairs"), with 05.s2's public-component test
(`logs/temper-05/s2-clients-unwatch.rs`, failing at jig `38ae1ba17`), which
passes with it. Then seed 7 of `a_chat_reads_then_replies_and_waits`, its
gate and the whole process tier, as ruling 370 says.

Scope: `Chat { task }`: the conversation from jig's `thread` (committed
turns, the person's words, work summaries, provisional text), its status
and spend; `task` watched, `activity` read a page at a time, `run`
watched while one is live; the composer sending a message; Stop; seen
(`client.md`, 3.5, the row `Chat { task }`; `index.html`). A task with
the chat label opens as `Chat { task }` from any link.
Needs: 06.4.
Files: `crates/temper-client-domain/src/pages/chat.rs` (new),
`pages.rs`, `acts.rs`; `crates/temper-client-view/src/pages/chat.rs`
(new), `words.rs`; `crates/temper-client-shell-terminal/src/tests/golden/chat.txt`
(new), with its tree in `src/tests/fixtures.rs` and its entry in
`src/tests/render.rs`'s list; `tests/client/tests/chat.rs` (new); the
fixtures its tests name (`chat`, `chat_page`), in each crate's
`src/tests/fixtures.rs`.
Shape:
- `pages::chat::Chat { task: u64, document: Handle, run: Option<Handle>,
  activity: Handle, thread: jig_client::thread::Thread, words:
  Box<[u8]>, pending: List<Key> }`: the `Thread` of jig 15.4, fed
  `document(&task)`, `page(activity_page)`, `older()`, `run(&run_state,
  text)`, `sending(key, by, words, place)` and `answered(key, &answer)` from
  the client's documents and tells; its header's status
  `facts::chat_status(chip, waits_on, project, me)` (jig 15.1, over the
  task document's `chip`, `waits_on` and `project`; it takes no run
  document) in `words`.
- `Act::MessageWords { words }`, `Act::Send` (`Message`, one keyed
  request), `Act::Stop` (`Stop`, one keyed request, not confirmed: it
  can be undone by releasing), `Act::Older` (the next page of
  activity), `Act::Seen` (jig's `Seen` over the conversation's turns).
Test shape: step tests in `crates/temper-client-domain/src/tests/pages/chat.rs`
(new), view tests in `crates/temper-client-view/src/tests/chat.rs`
(new), the golden, and the joint world. The reference joint-world
story, in `tests/client/tests/chat.rs`:
```rust
#[test]
fn a_message_in_a_chat_is_answered_in_its_conversation() {
    let mut world = fixtures::world(fixtures::SEED);
    world.front().answer_runs = true;
    let task = fixtures::chat(world.front(), b"Hello");
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(
        person,
        vec![
            Step::Type { find: Find::named(Role::TextArea, b"Message"), words: Box::from(*b"And then?") },
            Step::Press { find: Find::named(Role::Button, b"Send") },
            Step::See { find: Find::text(b"And then?"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"Answered by the run"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
```
Tests, each under 20 ms (unit) or 0.5 s (world):
`a_message_in_a_chat_is_answered_in_its_conversation`,
`opening_a_chat_watches_task_and_reads_its_first_activity_page`,
`a_live_run_is_watched_and_dropped_when_it_ends`,
`provisional_text_is_marked_until_its_turn_commits`,
`a_task_with_the_chat_label_opens_as_a_chat`,
`a_chats_header_says_its_status_and_spend`,
`seen_over_a_chats_turns_is_one_keyed_request`,
`leaving_a_chat_unwatches_its_task_activity_and_run`,
`stop_sends_one_stop_and_shows_it_pending_until_answered`.
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(a_message_in_a_chat_is_answered_in_its_conversation)'`
(under 0.5 s).
Don't: interleave turns and text by hand (jig's `thread` does); show a
message sent before its answer.
Size: at most 300 lines. Ruling 359: the chat lands with ruling 357's named
final-task repair at up to 650 production lines; no split. Ruling 365: up to
700, with its clippy fixes.
Removes: nothing.

### 06.6 A chat's proposals in place, and closing it

Scope: each proposal waiting in the chat's subtree (its `task`
document's `proposals`), shown after the conversation with its card:
accept or reject, said before it is sent, gone from the chat once
decided here or in the inbox (the designer's answer to the question of
2026-10-18T15:08:54Z: jig's `domain/engine.md`, 7.2, and
`client/domain/objects.md`, 10); Close, said before it is sent, as one
compound request with what becomes of each live delegate
(`shapes::close_chat`) (`client.md`, 3.5, 3.6; the story "a goal proposed in a chat, accepted
in the inbox, and gone from the chat's card", 10).
Needs: 06.3, 06.5.
Files: `crates/temper-client-domain/src/{pages/chat.rs, acts.rs}`;
`crates/temper-client-view/src/{pages/chat.rs, confirm.rs, words.rs}`;
`tests/client/tests/chat.rs`; the golden `chat.txt`.
Shape:
- The chat's cards: each `ProposalKey` in the `proposals` of the `task`
  document the chat page watches (`Subject::Task { task }`,
  `pages/chat.rs:35`), in that list's order, read once under a handle of
  the chat page's when the document first names it (`Read::Proposal`, as
  `pages/inbox.rs:153` reads one), and dropped, card and handle, when
  the document no longer names it. Each card is 06.3's (the same
  `Confirm::Accept` and `Confirm::Reject`), offering the acts its
  document offers, after the conversation's lines. A turn names no
  proposal at jig's pin: `Call.objects` is empty (`later.md`).
- `Act::Close` opens `Confirm::Close { chat }`, which says each live
  delegate's fate (cancelled, or moved to the person) and the
  template's grace; confirmed, `shapes::close_chat(me, project, chat,
  &delegates)` with one `Delegate { task, fate }` per live delegate
  (`Fate::Cancel` or `Fate::Move`, as the confirmation showed), one
  compound request (jig 15.3).
Test shape: step tests in `crates/temper-client-domain/src/tests/pages/chat.rs`,
view tests in `crates/temper-client-view/src/tests/chat.rs`, and two
joint-world stories in `tests/client/tests/chat.rs`. The reference step
test:
```rust
#[test]
fn closing_a_chat_with_a_live_delegate_is_one_compound_request() {
    let mut pages = fixtures::chat_with_delegate(fixtures::CHAT, fixtures::DELEGATE);
    let mut out = Queue::with_capacity(16);
    fixtures::act(&mut pages, &mut out, Act::Close);
    fixtures::act(&mut pages, &mut out, Act::Confirm);
    assert_eq!(fixtures::sent(&mut out), [fixtures::close_chat(fixtures::CHAT, &[fixtures::DELEGATE])], "jig's shapes::close_chat");
}
```
Tests, each under 20 ms (unit) or 0.5 s (world):
`closing_a_chat_with_a_live_delegate_is_one_compound_request`,
`the_close_confirmation_names_each_live_delegates_fate`,
`the_close_confirmation_names_the_templates_grace`,
`rejecting_a_proposal_in_place_is_said_before_it_is_sent`,
`a_proposal_waiting_in_the_chat_is_shown_after_its_conversation`,
`a_proposal_decided_elsewhere_leaves_the_chats_card`,
`a_goal_proposed_in_a_chat_accepted_in_the_inbox_leaves_the_chat`
(world), `a_closed_chat_shows_under_closed_in_the_chats` (world).
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(a_goal_proposed_in_a_chat_accepted_in_the_inbox_leaves_the_chat)'`
(under 0.5 s): the run proposes a goal in the chat, the person accepts
it in the inbox, and the chat's card leaves, the engine root holding the
goal.
Don't: chain requests for one act (contract.md, 3.4); choose a
delegate's fate the confirmation did not show; read what a call made
from its outcome's words; filter the task's proposals by proposer; drop
a card by `Thread::shows`, which needs temper's turns to show a call's
cards (`later.md`).
Size: at most 250 lines. Ruling 364: it lands whole at up to 550 production
lines with no question; past 550, it blocks with the count by file.
Removes: nothing.

## Interface sketches

`temper_client_domain::Address`, `Filter`, `Kind`, `Proposal` and
`Confirm` compile at the base as `temper_client_domain::sketch_pages`
(`logs/w9/clippy-sketches-3.txt`), and, as below, with
`Jig(jig_client_view::address::Address)`, `jig_client::contract::Proposer`,
and `Debug` and `#[must_use]` where shown, under clippy at jig
`b58a49c39` and temper `70f376c5` (06's ready check, 2026-10-17). The
pages child's surface names jig 15's types where it uses them.

```rust
// crates/temper-client-domain/src/address.rs
/// temper's addresses (client.md, 3.5). Board and Changes are later.md's.
pub enum Address {
    /// Home: what needs the person, then updates, under the filter.
    Inbox { filter: Filter },
    /// The composer starting a chat, then the person's chats, live then closed.
    Chats,
    /// One chat's conversation: a task with the chat template's label.
    Chat { task: u64 },
    /// A tracked task's page (temper 07.1).
    Task { task: u64 },
    /// The adoption form, the forge side's page (4.4).
    Adopt { project: u32 },
    /// A page of jig's reference UI: transitional, removed by temper 07.4.
    Jig(jig_client_view::address::Address),
}
/// The inbox's filter (`inbox.html`): a field that is none shows every value.
pub struct Filter { pub project: Option<u32>, pub kind: Option<Kind>, pub goal: Option<u64> }
/// What an inbox entry asks of the person.
pub enum Kind {
    /// A proposal the person may decide.
    Proposal,
    /// A question a task asked them.
    Question,
    /// A task held, waiting for the escalation's holder.
    Held,
    /// A person task addressed to them or to a role they hold.
    PersonTask,
}
```

```rust
// crates/temper-client-domain/src/acts.rs
/// A proposal, as `Decide` names it: its proposer and its number (jig's contract.md, 3.2).
pub struct Proposal { pub proposer: jig_client::contract::Proposer, pub number: u64 }
/// The page's one open confirmation (client.md, 3.6): an act said before it is sent.
pub enum Confirm {
    /// Accept: a proposal never changes once made, so it names no revision (contract.md, 5.2).
    Accept { proposal: Proposal },
    /// Reject with the reason the card's field holds.
    Reject { proposal: Proposal },
    /// Release a held task at its hold's revision.
    Release { task: u64, revision: u64 },
    /// Close a chat, with what becomes of each live delegate (`shapes::close_chat`).
    Close { chat: u64 },
}
```

```rust
// crates/temper-client-domain/src/pages.rs, after jig_client_view::Ui (jig's view.md, 18.1)
/// temper's pages: owner `Pages` (2), the asker of every handle and key they give the client.
#[derive(Debug)]
pub struct Pages { /* me's handle, the project chosen, the page, the confirmation, acts by key, seed */ }
/// To the pages, from the root.
#[derive(Debug)]
pub enum Event {
    /// The client started: watch `me`.
    Started,
    /// One of temper's addresses gone to; `Jig(..)` and `Adopt` are elsewhere for the pages.
    Went { address: Address },
    /// A person's act on one of the pages' nodes, as temper's bindings decoded it.
    Act { act: Act },
    /// What the client tells under asker 2: changed, ended, a page, pending, answered, kept.
    Told { told: jig_client::Told },
    /// The link's state, which every page shows.
    Link { link: jig_client::Link },
}
/// From the pages, for the root.
#[derive(Debug)]
pub enum Request {
    /// One of the client's events (`Watch`, `Unwatch`, `Read`, `Send`), under the pages' asker.
    Client(jig_client::Event),
    /// An address to show.
    Show { address: Address },
}
pub fn step(pages: &mut Pages, env: &Env<Limits>, client: &jig_client::Client, event: Event, out: &mut Queue<Request>);
pub fn fire(pages: &mut Pages, env: &Env<Limits>, client: &jig_client::Client, out: &mut Queue<Request>);
#[must_use]
pub fn next_deadline(pages: &Pages) -> Option<Time>;
#[must_use]
pub fn shown(pages: &Pages) -> u64;
```

## Questions to raise when reached

- The inbox's "updates" group (results, replies) has no act in the
  design's row beyond seen; if a story needs to open a result, the page
  it opens is not designed for a result (only for tasks): raise it.
- `me` names the chat template's label per project; if a person in two
  projects starts a chat with no project chosen, which project is the
  chat's is not in `client.md`, 3.5: the default this file takes is the
  project the person is in (3.1: "the first at start"); raise it if a
  story needs another.

## Done when

- 06.1 to 06.6 have landed on temper's `origin/main`.
- `git -C <worktree> grep -n -e 'First words' -e 'Start a conversation'
  -- '*.rs'` prints nothing: the stories start chats on temper's page.
- `cargo nextest run -p temper-client-world`, once, passes the
  stories named in 06.1 to 06.6.
