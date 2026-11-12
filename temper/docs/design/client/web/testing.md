# The browser tier

- **The driver is skein's.** `skein-browser` (skein's `testing/browser.md`) runs
  headless Chromium as a child and drives it over the DevTools protocol
  on a pipe, from the loop beside the engine.
- **The person's browser face is skein's** (`skein-fake-person-browser`;
  skein's `ui/web/testing.md`, 2): the same queries and acts as on the tree,
  made over Chromium's accessibility tree, and the browser's own: the
  current address, read from the navigation history, and keys pressed
  with Shift.
- **What it runs against:** `temper-box serve`, a child of the harness's
  loop, on a scratch directory, from a configuration the test writes
  with the bundle named: no Forgejo, no runner. 1's
  stories need no answer of the forge's or the LLM's, so its forge and
  its LLM endpoint are unreached, on closed loopback ports, as jig's
  tier serves (jig's `protocol/browser/testing.md`). The fake
  forge, hosted on loopback in that loop by skein's forge peer
  (`../../testing/fakes.md`, 2), and skein's fake LLM peer, as the
  process tier has them (`../../testing/process-tier.md`), join with the
  first story that needs them (2). 4's test has the forge peer
  serve only the fake's GitHub `web`. skein's fake issuer joins for
  a plan's sign-in alone (2).
- **Only what a browser alone shows.** temper's stories (what a page
  shows, what an act sends, a chat answered, a proposal accepted, a
  repository adopted, a run's live text) are the joint world's
  (`tests/client`; `../../testing/checks.md`, 3) and the real loop's, through
  the terminal shell (`../../testing/real-loop.md`); they never run again in a
  browser. A story is in this tier only when no world can show it: the
  bundle, the browser's platform (the address, history, the cookie,
  keys) or an attack that needs a browser. Each names what only the
  browser shows. What skein's browser parts prove for every application
  (the mapping of roles, a patch keeping focus, caret and scroll, the
  store, the bundler) and what jig's prove for every client of jig's
  (jig's `protocol/browser/testing.md`) are not proved again.
- **Every story checks the policy too:** skein's harness fails a test
  on a policy violation, an exception or a native dialog (skein's
  `ui/web/testing.md`, 2).

## 1. The first milestone's stories

1. **Signing in by the printed link:** the welcome shows, at
   `/welcome`, or, where the first run is over, the chats, the
   address reading `/chats` once they do (`README.md`, 2.4); a reload on a chat's
   address shows that chat, signed in by the cookie; a link spent shows
   the reason-specific signed-out paragraph (`../view.md`, 1). After a
   server restart, reopening the first printed link shows "This link
   isn't from this temper. Use the one its terminal printed." because a
   restart forgets retired link digests (jig's `protocol/client/sign-in.md`).
   That the secret leaves the address before the first request is held by
   the shell's start order (`README.md`, 2.1), and is not tested.
2. **Addresses:** skein's `back_and_forward` (skein's `ui/web/testing.md`,
   2), from the chats to a chat by its link and back; an address
   that is no page opens the inbox, and the address reads `/` once it
   shows (`README.md`, 2.3).
3. **The composer:** in the chats, Enter starts a chat with the words
   once, and Shift+Enter makes a new line in them (`look.md`). The chat opens
   with its composer focused, the focus stays there after Enter and after
   a press of "Send", and a chat opened from a list keeps its heading's
   (`../pages/README.md`).
4. **A panic:** skein's `panic` (skein's `ui/web/testing.md`, 2), which
   checks the shell's wiring of the restart part, at the test address
   `/test/panic` at which the shell panics at start, as jig's reference
   shell has it (`README.md`, 2.1).
5. **Reading back:** scroll a chat's log up; Latest appears. New words
   arriving while reading back leave the view where it is. Press Latest
   or send from the chat; the end returns, read again if the window let
   it go, and the pill goes (`../pages/chat/conversation.md`, 2).
6. **jig's attack stories that need a browser,** from
   `jig-client-fake-person-browser`, against `temper-box serve`: a
   cross-site request from another port and from another host does
   nothing; a framed page shows nothing (jig's `protocol/browser/testing.md`
   ). The rebound Host, which needs no browser, stays in the
   process tier (`../testing.md`).

Not in the tier: skein's `typing_while_patched`, which takes a second
person, and `write_refused`, which takes a test build's bounded small
store; what they check is skein's page and store, which skein's tier
proves.

## 2. Later pages

A page gets a story here only for what of its own a browser alone
shows. The inbox, a task's page and the adoption form have none: a
first load at any address is the shell's one start (`README.md`, 2.1), which 6's
tests 1 and 2 hold on a chat's address and a filter's. A procedure's
plan, its agent, the lineage bar and a fresh client at an agent's
address are the client world's (`suite::b2`, `suite::task`). The
change's steps to landed, its gates, its bounds and its ended tasks
hold nothing a browser alone shows: the client world's
`change_panel.rs` reads them. A later page that has such
behaviour brings its story with it: `ConnectModel`'s sign-in, a tab of
its own sent to the provider's address while the page stays
(`../pages/README.md`).

**`ConnectModel`'s sign-in, in a new tab.** What only a browser shows:
a press opening a tab within its gesture, the answer pointing it, the
provider's redirect followed to the engine's listener in that tab, and
the page, which stayed, drawing the end.
- **Against:** `temper-box serve` with two synthetic providers sharing an issuer,
  its authorization, token and key addresses, skein's fake issuer
  (`skein_fake_peers::oauth::Peer`), hosted on loopback in the
  harness's loop, each grant a success for one synthetic subject, and
  the provider's redirect port the kernel's (the listener's port
  varies). The fake has no TLS, so the tier's pages load with
  `internal-loopback-navigation` (3).
- **The story:** a person on `ConnectModel` presses "Sign in to Test provider";
  one more tab opens (skein's `ui/web/testing.md`, 2, Its tabs),
  which can't reach its opener by the time it's at the listener, its
  address the listener's; the page's own address reads
  `/accounts/connect?provider=17&way=sign-in&source=S`, in place of the
  entry it had; the page shows Ready ("Signed in to Test provider"); the fake
  has taken one authorization and one code exchange. A reload then shows
  Ready again and the issuer has taken no second authorization: the tab
  is pointed once.
- **The welcome's Model step,** the first thing a person meets: the same
  press holds the welcome at `/welcome?at=model`, and it moves on to Ready
  by itself, `/welcome?at=ready`, with one authorization and one code
  exchange of its own. It runs first in this story's test (6, 9), on a
  second provider of the same issuer, Welcome provider (18), with a model of
  its own: the issuer's one subject signing in twice on one provider renews
  its account, in other words than Ready's
  (`../pages/accounts.md`, 2), and a usable model leaves the welcome no sign-in to
  press. Each sign-in keeps its tab, so the story ends with two.
  The client world's `suite::second_sign_in` holds that the second provider's
  sign-in reaches Ready after the welcome's.
- **Run on Chromium alone,** the tier's one browser. Firefox and Safari
  are read: a press reaches `window.open` within its event's listener
  (skein's `ui/web/testing.md`, 3).
- **Not here:** the tab's order, the press's before the answer, a refusal,
  and the terminal's root are the client world's (`tests/client`) and the
  host's; a tab refused by the browser and the pop-up rule, skein's tier.

**`ConnectForge`'s App of your own, the pasted key,** has no story
here. Its field is a text field of the secret kind, which skein's page
maps to `<input type="password">` (skein's `ui/web/README.md`), its value read
as any text field's; HTML's value sanitization strips a password
input's line feeds, the one thing a browser alone would add, and the
client restores the PEM's armour from what is left (`../forge/deployment.md`).
The view's tests hold that the field is a text field of the secret
kind; the client domain's, that the stripped, the file's and the CRLF
forms reach one carried value with the armour's two line feeds, and
that the field is emptied; the service's, that `Paste` reads the fake's
real key in that restored form.

## 3. Its budget, outside the gate

- **A suite of its own,** outside the focused and fuzzy suites
  (`../../testing/README.md`, section 2), in its own crate
  (`tests/client-browser`, `temper-client-browser-world`), run by
  `scripts/browser.sh`, a slow tier outside temper's gate, with the nextest profile `browser-temper`,
  one test at a time, on the bundle it builds first, with the dev
  profile.
- **The tier's bundle** is the shipped page's, built apart
  (`scripts/bundle.sh`). Each address the tier loads carries the page's
  internal settings (`README.md`, 2.1), `internal-loopback-navigation=1`
  and `internal-small-window=1`, which the page honours on the tier's
  loopback origin: so a sign-in's tab (2) can go to the fake issuer on
  plaintext loopback (skein's `ui/web/README.md`, section 8), and a
  reload, a load without them, is loaded again by the tier with them.
  The tier names that bundle in its configuration, so it
  runs alike over a binary built with or without its own (`README.md`, 2.6). It holds
  the compressed forms and Chromium takes gzip on loopback, so every
  test loads the compressed module under temper's policy.
- **The tier's window:** the page's `internal-small-window` names a
  conversation's
  window of 32,768 bytes, a thirty-second of a person's `document_bytes`,
  through the same window mechanism (`../pages/chat/conversation.md`, 2). A window holds no
  less than `long_text`, so the setting sets the page's `long_text` and
  `turn_text` to the same 32,768: the largest page `read_docs` answers
  (`temper-docs`' `PAGE_BYTES`), so that a tier story's line holding one
  stays whole. The long reading
  story (6, 14) passes that bound by at least two windows' worth and
  evicts and recovers lines at both ends. The unanchored position story
  stays within one window; it checks the words holding across a read above,
  and a window dropping lines would change what that proves, so it never
  reads the long story's chat.
  Reading's replies are sized to the window: a reply's bytes ride on one
  horizontally scrollable code line, its height on its paragraphs. So the
  code lines scale with the window, and each history's bytes keep their
  share of it, while the replies' counts and paragraphs stay: the keys
  each step presses and the reply each step reads rest on them.
  The ordinary binary bundle and the journeys'
  never turn it on, and a long chat at the whole bound is read by hand.
- **A chat's history:** what a reading story reads is made through
  `temper ctl` before a page opens on it: `playground start`, `chat start`
  and `task message`, each answer waited for with
  `task show --until replied`. A face of its own signs ctl in with the printed
  link and opens the spent link, so that the server prints the reader's, as
  `public.md`, 4's walk does. A page types only what its story is about: in reading's,
  the message sent from far back. Reading's test (6, 14) holds three chats
  on one `temper box`, as few as its stories allow: T1, the short chat, which
  six messages sent through ctl then make Latest's; T2, the position story's,
  inside one window; T3, the long story's, past three. Their requests cue
  disjoint scripts, and their replies together stay within the fake's joint
  script allowance (`temper-fake`'s `SCRIPT_BYTES`), both held by a focused test
  of the tier's crate.
- **Anchoring within a step** (6, 14): reading's test serves a private
  bundle whose stylesheet, only at a page one pixel taller than the kit's
  1280 by 800, turns the browser's scroll anchoring off on the log
  (`overflow-anchor: none`) and sets the log 7 px lower. The position story
  goes to that height on its live page and back at its end, so its words stand
  on the page's own anchoring while the other stories stay on the browser's.
  That the log moved by those 7 px says the rule holds, and the next story's log
  standing where the position story's first did says it has gone.
- **The playground's test** (6, 12) runs on the built playground
  instead, `scripts/bundle.sh --playground` in its own profiles
  (`../../playground.md`, 7), served as static files with their `headers`
  by the tier's own static server, which maps them as a site's host
  would; the tier reads the world's figures from the page's frame, and
  compresses the download's with the machine's `gzip` (`../../playground.md`, 9).
  That bundle has no compressed forms (`../../playground.md`, 7).
- **A cap: at most 12 tests, and 120 seconds** on the slowest of the
  machines that build temper (the user, 10-27, temper#15). Past it,
  stories that share a setup become one test of several steps before
  any browser-only proof is dropped, and what must leave the browser
  goes to the lowest world that still shows it, named in its line of
  6; the cap stays. The tests are 6's.
- **A test of several stories** says which failed: it marks where each
  story's steps begin (the tier's `Stories`), and a failed step's report, and
  each measure read after the run, name its story. Each story's assertions
  stay as its steps, in its order. It reruns whole on a flake, and a failure
  hides the steps after it: taken, for the cap. It stands on the same forty
  seconds as any run of the harness (skein's `ui/web/testing.md`, 2): a test that
  can't fit them is two tests, never a longer run.
- **Chromium** is the machine's, at the path its owner configures
  (skein's `testing/browser.md`), which skein's, jig's and temper's browser
  tiers read from `SKEIN_TEST_CHROMIUM`. A missing Chromium or
  bundle fails the suite, and never skips it (skein's `ui/web/testing.md`,
  1).
- **The client's crates for wasm32:** the gate runs clippy for
  `wasm32-unknown-unknown` over `temper-client-domain`,
  `temper-client-view`, `temper-client-protocol-wire` and
  `temper-client-shell-web`, so a change that stops them building for
  the browser fails the gate.

## 4. GitHub's manifest page

One test, the github plan's: 6's 10, counted in 3's cap.

- **What only a browser shows:** the manifest page's form posted under
  the page's own policy, `form-action WEB` (`../forge/served.md`, 1). The
  redirect and the setup address need no browser; the real loop's
  GitHub stories follow all three steps as plain requests
  (`../../testing/real-loop.md`).
- **What it runs against:** `temper-box serve` with a GitHub forge whose
  `api` and `web` are the fake's GitHub layer, hosted on loopback in the
  harness's loop by skein's forge peer (skein's `testing/fakes/github.md`, section
  1), which serves the tier for this test alone: the test needs only
  its `web`. A GitHub forge needs a fixed
  clients' listener (`../forge/served.md`, 1), so the tier reserves a loopback
  port the kernel assigns before `temper-box serve` starts and names it in
  the configuration.
- **The test:** the signed-in browser opens the welcome's forge choice
  and presses Connect GitHub. The ordinary client supplies its return
  address. The manifest page posts its form on load to
  `WEB/settings/apps/new?state=S`, the fake's, with no policy violation.
  Its answer holds one link, Create GitHub App. Pressing that link runs
  the uninterrupted conversion, default installation and setup callback;
  setup answers with a 303 to the welcome's selected forge. The browser
  reaches that address and reads the connected state. One manifest post
  and one installation are observed. No URL is intercepted or rewritten,
  and split HTTP requests do not stand in for this click chain.
- **The advancing loop** keeps lending peers active during navigation and
  pending browser commands. A stalled browser operation is diagnosed at
  this tier before assigning it to the service or a fake. A separate
  process comparison uses the same ordinary peer configuration and
  lifecycle when needed. Evidence records route categories and statuses,
  never state/code values, token contents or a secret final address.
- **No github.com:** `web` is the fake's, and the page's policy lets
  its form post nowhere else. No test of the tier holds a real
  credential.

## 5. Forgejo's authorization

One test, the provider's (`../../protocol/openid-provider.md`, 8).

- **What only a browser shows:** jig's strict cookie, absent from a
  navigation that started on another site and present on the page's own
  exchanges, so that the `Authorize` place answers *go* for a person
  signed in.
- **What it runs against:** `temper-box serve` with jig's provider, its key
  and its client's secret made in the test's scratch directory, and one
  client whose redirect URI is a page of the harness's on the other
  loopback site (`localhost` against `127.0.0.1`), hosted in the
  harness's loop: a link to temper's `/oauth/authorize` with that
  client, and the callback, which records its query. No Forgejo: the
  process tier's story has the real one (`../../testing/process-tier.md`).
- **The test:** signed in by the printed link, the browser opens the
  harness's page and follows its link; temper's page sends `Authorize`,
  and the browser's address becomes the callback's, its `code` and the
  link's `state` there. Nothing of the token's is the browser's.

## 6. The stories

The tier's tests, each on its own `temper-box serve` (the playground's on its
built playground; 2 and 11, 13 and 14 on `temper box`, whose first run is
over). Each is one test of the steps it names; 2 and 11 share one, and 4 and
5 another; 8 and 9 share their Accounts setup, and 16 shares its sheet's
setup across desktop and phone widths. The twelve test functions in
`tests/client-browser/tests/browser_all` are:

| Stories below | Module | Test |
|---|---|---|
| 1 | `browser_sign_in` | `signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` |
| 2 and 11 | `browser_pages` | `history_keeps_the_chat_focus_and_filter_across_reload_and_back` |
| 4 and 5 | `browser_pages` | `enter_starts_one_chat_and_the_composer_keeps_the_focus` |
| 6 | `browser_attacks` | `cross_origin_requests_and_framing_cannot_act_as_the_person` |
| 7 | `browser_authorize` | `a_cross_site_authorization_returns_a_code_and_the_links_state` |
| 8 and 9 | `browser_accounts` | `the_sign_in_opens_a_tab_and_lands_on_accounts` |
| 10 | `browser_github` | `the_manifest_posts_once_and_history_waits_for_continue` |
| 12 | `browser_playground` | `story_a_walks_on_the_built_playground` |
| 13 | `browser_public` | `public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` |
| 14 | `browser_pages` | `a_chat_opens_at_its_end_and_reads_back_keeping_its_place` |
| 15 | `browser_pages` | `closing_project_rename_returns_focus_to_the_summary` |
| 16 | `browser_connect_flows` | `scrolling_the_sheet_reads_more_then_search_picks_and_adopts` |

1. **Signing in** (1, 1): the printed link signs in, a reload on a
   chat signs in by the cookie, and the link spent shows the
   signed-out words; signing out clears the cookie on later loads. The same
   chat reconnects after a server restart and remains live after rapid reloads.
2. **History** (1, 2): from the chats to a chat by its link, Back and
   Forward; a new chat's composer focused, and its filter's address and
   pressed state kept across a reload and Back. The story awaits Closed
   pressed after reloading `/chats?state=closed`, then All pressed after
   Back to `/chats`, each by its exact button name within `WITHIN`
   (skein's `ui/web/testing.md`, 2), independently of a Drafts button arriving
   before Closed. An address that is no page opens
   the inbox at `/` (1, 2). Its chat's model scripts a run of calls: the folded
   line hides its rows until a press, and stays open on its node across a reply,
   `details[open]` being the browser's, which the model doesn't hold
   (`../pages/README.md`).
4. **The composer** (1, 3), and its model picker (`../pages/README.md`): one press
   applies a model and an effort, at 420 px and 1440 px, and a press away or
   Escape closes it.
5. **A panic** (1, 4).
6. **jig's attacks that need a browser** (1, 6): a cross-site request
   from another port and from another host does nothing, and a framed
   page shows nothing, in one harness with both attacker host names.
7. **A cross-site authorization** returns its code and the link's state
   (5).
8. **The accounts page after a key's send:** the completion returns to Accounts
   with the key's row; Back restores an emptied field, Forward the chats (2).
9. **`ConnectModel`'s sign-in, in a new tab** (2), after the welcome's Model
   step's, each with one authorization and one code exchange. The replaced
   history entry returns to Accounts, and its counted heading remains through
   Back, Forward and reload while the completion message disappears.
10. **GitHub's manifest page** posts itself once, and Back shows it
    without posting again until Continue (4).
11. **The box's project across a reload:** the playground's project
    made, the Chats page Live, never Behind, with the story's chat,
    before and after a reload (`../README.md`, 2). Before that chat, at
    420 px, Home's To begin and its Start open Ask temper's page, whose
    composer and model picker end above the three tabs' top (`look.md`, *The
    project's frame*), and whose ways show their tabs, Agents review
    current; "Draft first" pressed shows its line, its path and "Use this"
    (`../pages/README.md`).
12. **The playground's story A** on the built playground, its log
    downloaded and replayed natively (`../../playground.md`, 9).
13. **Reading a public project** (`public.md`, 4), with no sign-in, its board and task
    lines readable at 1440 px and 420 px, and losing it when it is made private.
14. **Reading back** (1, 5), one test of four stories, each on its own page
    load (3): a short chat opens whole, with no reading status, both its ends
    in the log; Latest returns a chat to its end, after new words and after a
    send from far back, the view holding while it reads back; earlier words stay
    put without the browser's scroll anchoring; and a long chat opens at its end
    and reads back from the composer, past its window at both ends.
15. **A project's rename** closed by Escape, Enter and Save returns the
    focus to its summary, and never to the page's `main`.

16. **The repository sheet** (`../forge/family.md`, 4; `../forge/repositories.md`, 1): scrolling
    the last loaded row into view reads the second page without changing the
    list's bounds; search finds its repository, selection counts it, and adoption
    returns to its row. On the same deployment at phone width, a reopened
    sheet meets the viewport's bottom and keeps its primary button visible.

The cap includes all browser tests, whatever their names.

Not in the tier, each in the lowest world that shows it:
- **Models switched off and back, and added by name, across reloads:** the client world's `models::a_switch_turned_off_and_back_holds_across_a_reload` and `models::a_model_added_by_name_is_on_the_picker`.
- **A run of calls' rows and their words:** the client world's
  `suite::call_line::opened_it_holds_one_row_a_call` holds one row a call, its
  first line alone. Test 2 keeps the closed line hiding them and the open line
  across a reply.
- **Goals' link to a draft:** the view's
  `tests::goals::the_link_shows_with_a_tracked_template` holds its button
  opening the Start goal sheet with no draft. The client world's
  `suite::chat_draft::a_goal_from_goals_asks_its_person_and_the_answer_wakes_it`
  starts by a typed Draft path and answers its person task. Test 2 holds
  what a browser's history does with a link.
- **A procedure's plan, agent and lineage, and a fresh client at its address:**
  the client world's `suite::b2::roadmap_b2_through_the_task_pages` and
  `suite::task::a_delegates_page_shows_its_brief_run_and_report`.
- **A chat's change card to Landed, its closing and the Closed filter's row:**
  the client world's
  `suite::chat::a_new_chat_lands_its_delegated_change_and_closes_in_your_chats`.
  Test 2 keeps the filter's address and pressed state.
- **A key added, on the Accounts page:** the view's `tests::accounts` and the
  client world's `suite::accounts_page`. Test 8 keeps Back and Forward and the
  emptied field.
- **A landed goal's public pages:** the view's `tests::public`; an ended goal
  kept in the public index, jig's
  `suite::public::an_ended_public_goal_is_retained_by_the_public_referee`.
- **Address-codec edge rows:** `temper-client-domain`'s
  `tests::address::{a_path_that_is_no_page_is_missing,each_number_spans_its_types_range}`.
  Browser test 2 keeps the shell's replacement of a missing address with `/`.
- **Repeated loads at the watcher bound:** the client world's
  `suite::reloads::repeated_loads_release_the_previous_clients_watches`.
- **Composer bounds:** the client domain's
  `tests::pages::chats::new_chats_and_cues_keep_the_send_bound_when_receiving_grows`.
- **Provider routes and refusal pages:** the service's provider protocol test,
  with the view's `sign_in::tests::each_authorization_page_uses_the_mockup_words`.
- **A moving account's rendering:** the view's
  `tests::accounts::a_moving_account_shows_its_state_without_buttons`; the
  accounts world's `suite::moves::a_written_move_erases_the_source_and_its_account`
  holds removal.
- **A chat's day and turn clocks beside its speakers:** words and roles
  alone, which the view's tests (`temper-client-view`'s chat tests:
  order, day lines, midnight) and the client world's chat already hold.
- **GitHub's connect after the manifest** (conversion, installation,
  the forge chosen, adoption, the Model summary): plain requests and the
  client's events, in the real loop's GitHub world and the client
  world's welcome, forge and adoption stories; the manifest's post, the
  one thing a browser alone shows, stays in 10.

## 7. Open questions

- **A filter row while its project's ways arrive** (`README.md`, 2.3, `testing.md`, 6): Drafts
  appears before Closed once the ways offer drafts, moving Closed under
  a person's pointer. Default: keep the conditional Drafts button; a
  separate look decision settles whether the row should reserve its
  place. The history story reads the filter by name and pressed state,
  and makes no claim that the row stays still.

- **A phone's width in a test** (`look.md`): skein's browser kit opens pages at
  1280 by 800 and offers `Viewport` to change their CSS pixel dimensions,
  with desktop input behaviour. The public board and task story (`public.md`, 4)
  checks 1440 px and 420 px. Other phone rules remain held by their sheet
  checks and throwaway walks alone: the accounts tabs, owner menu, rail,
  chat and document panes have no retained viewport story.

- **A chat's way to the Closed filter** (`testing.md`, 6): a chat closed on the box took 8.5 s
  to show there, and no measure splits that between the chat template's 5 s of
  grace (`temper-service`'s `stock.rs`) and the box's 5 s `agent_stop_ms`. A person
  waits it, and no test of the tier does. Default: as it stands, until the wait is
  split and a request asks it shorter.

- **The box's pace in the tier** (`testing.md`, 6): the stories on `temper box` keep story A's
  paces, a reply's 1200 ms and CI's 4000 ms. None waits for a landing, and unpaced
  fakes would land in the background for CPU nobody reads. Default: paced.

- **A wait's CPU in the tier** (`testing.md`, 3): a `See` asks Chromium's accessibility tree
  again at `skein-browser`'s `poll`, 50 ms. Default: that poll stays. Repeated-miss
  backoff needs a measured saving without delaying presses or the notice of a
  change; the repaired long story's query count alone gives no reason to add it.

- **The remaining browser cost** (`testing.md`, 3, 6): another repair may prepare the
  other stories' histories outside the page, and combine stories sharing a setup
  before dropping a browser-only proof. Default: keep the CPU budget, timeouts,
  membership and cap; the four reading stories retain their layout, focus and
  scroll proofs. A whole-tier run after landing decides its remaining cost.

- **Merged stories after landing** (`testing.md`, 2, 3, 6): check the call line's
  open node across a reply, the composer's model picker, both sign-in tabs
  with the welcome first, and reading's four page loads with anchoring off
  only for the position story. Default: compile during the plan; run in
  Chromium after landing, preserving every assertion, the forty-second
  run deadline and the whole tier's 127.5 CPU-second budget, 85 raised by
  half by the user's word of 11-08, for now, to be revisited. A whole-tier
  run supplies the cost proof; separate story measurements do not.

- **Reading's test past its run's forty seconds** (`testing.md`, 3; `testing.md`, 6, story 14). Default: it
  becomes two tests, the long chat's apart, and test 15, which shares its setup
  with 4 and 5, becomes a step of theirs, so that the tier stays at twelve.

- **A long chat at the person's whole bound** (`testing.md`, 3; `../pages/chat/conversation.md`, 2): the
  small-window stories prove eviction and recovery through the same mechanism,
  but leave the 1,048,576-byte window's manual reading check separate. Default:
  keep the person's bound and check opening, position, both ends, Latest and
  sending there by hand.

- **A public page's plain text in one query** (`public.md`, 4): the page-text walk,
  apart from geometry measurement, reads a node at a time, about 0.8 CPU s
  over its seven pages. Default: read a node at a time, accepting `public.md`, 4's
  queued and unqueued landings forms, until that passes 1 CPU s, when the
  browser kit would read a page's text at once.

- **Every wrapped text fragment** (`public.md`, 4): checking all of a title's wrapped
  fragments needs a browser testing interface decision (skein's
  `testing/browser.md`, 11). Default: keep `Seen` and its first content quad;
  missing rectangles remain failures, with the text and measurement stage.

- **A public page changing during measurement** (`public.md`, 2, 4): one terminal
  boxed query avoids repeating every rectangle request for each text node,
  but is no atomic snapshot of the page. Default: keep the field, gutter,
  wrapping, navigation and overlap assertions and read a missing box's text
  and stage from its failure. A node gone between requests refuses the
  query, which fails the page assertions; that alone does not explain an
  answered query with a missing rectangle. A timer-driven read remains an
  unproved explanation.

- **Expected browser errors scoped to an act** (`public.md`, 4): the public walk's
  sign-out permits an expired stream's 401 console message. Expectations
  match text for the whole walk, so the same words from a later public
  read are accepted too. Default: keep that expectation; distinguishing
  the act or resource needs a harness interface decision.

- **A compile cache across the tier's tests** (`testing.md`, 3): each test's fresh profile
  compiles the bundle's wasm again, where a second load in one profile hits
  Chromium's own cache. Default: none, since a shared profile would change what each
  test starts from.
