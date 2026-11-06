# browser: status

2026-11-06. Where temper's code lags or departs from [browser.md](browser.md).

- **Not yet:** 4.3, *The tier's window*: the tier's bundle holds the whole `document_bytes`,
  and `reading_tier`'s replies are sized for it.
- **Not yet:** 4.3, *A chat's history*: `browser_pages`'s reading stories type their history
  in the page, and `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` takes
  a second tab for its new words.
- **Not yet:** 2.3, *A heading drawn after its page*: that a focus placed while a heading
  waits keeps its place, which the answer of 2026-11-06 to the `ui-composer-focus` task
  `composer-focus` asks, is seen by reading `temper-client-shell-web`'s `focus.rs`. No test moves the focus meanwhile:
  `browser_pages`'s `enter_starts_one_chat_and_the_composer_keeps_the_focus` holds the
  heading taking it from the main, and that module has no test of its own.
- **Not yet:** 2.5: the answer of 2026-11-06 to the `ui-rename-focus` task
  `rename-row` asks a test that a long chat's document never exceeds its window at
  1280x720 and 1000x600 with "Older" pressed twice. What holds it is the sheet's
  rule (`temper-client-shell-web`'s `tests/suite/switcher.rs`,
  `scrolling_panes_contain_their_hidden_absolute_parts`), and no test measures the
  page; no reason is recorded. That test names the session's log and draft pane,
  and the session's thread with no log, which scrolls too, is not positioned. The walk
  of the request `ui-walk-3` of 2026-11-06 measured it by hand: the document at its
  window at each size, in Chat and Split, one pane scrolling, but with no older read
  ended (`README.status.md`, 3.5).
- **Not yet:** 2.5: a numbered list read from Markdown
  (`temper/docs/design/draft/chat-reading.md`, 6) shows no numbers in a card of a
  chat's thread: `temper.css` makes `.temper-card:not(.temper-task-card) li` there
  `display: flex`, under no `ul`, and an item that is no `list-item` draws no
  marker. A proposal's reason reads there (`temper-client-view`'s `cards.rs`,
  `document`). On a document page `.temper-doc-inner ul` has no `ol` beside it, so
  a numbered list takes the browser's margins and padding. Both are seen by reading
  the sheet: no browser test draws a numbered list, the request `chat-reading-d`
  having the user read one on a real chat and leaving the sheet to the look's owner.
- **Not yet:** 2.5, *temper's own way*: no render has checked the picture as an `ol`: the walk
  of the request `ui-walk-3` opened the reference at each width and read its title, flow and
  line, and names nothing of the list's margins. `configure-temper-own-way`'s render check read the numbers as a `ul` item's words, and numbered lists reached skein's
  reader after it, so the `ol` takes `.temper-doc-flow ol`'s padding and the browser's margins, and
  `.temper-doc-flow ul` now matches nothing, left for whoever next edits that rule. Seen by
  reading; the view's tests read the terminal's print, which says `1. Design draft` either way.
- **Departs:** 2.5: `Ctrl+\` keeps Chat → Split → Draft on a phone too,
  with Split displayed as Draft. The 2026-11-05 answer to
  `ui-chat-draft/document-pane` accepts this cycle without viewport state;
  only the visible buttons omit Split (`README.md`, 3.5).
- **Design departs:** 2.5, from the request `ui-phone` of 2026-11-05, decided by
  its plan's cut and its sweep. A chat's foot stands 16 px in, where the
  request's check has the mock's 12: the look request `configure-temper-name`
  moved it for every chat, and the request keeps look requests made. A chat's
  foot and the toast take the home bar's inset, which the request asks of the
  tab bar alone: a covered page leaves them under it. "Every tab and primary
  button at least 44 tall" holds where the mock draws no shorter: the welcome's
  primaries stand 40 px, Home's acts 34, and a goal's primary outside its
  choices, the sheet's among them, 40, as the mock's `.btn` and `.btn.small` do.
  The mock's `box.html` draws a terminal, what `temper box` prints, and no page
  of the client's draws it.
- **Design departs:** 2.5, from the request `close-chat` of 2026-11-06, decided by
  its plan's cut: a confirmation sheet's acts stand 44 px on a phone, where the
  request's overlay has 38, to keep a phone's target. So at 420 px the sheet stands
  6 px taller and 3 px higher than the request draws it.
- **Departs:** 7.2: the gallery's comparison pages do not hold one copy of
  the greenfield scenario. `home-morning` is Hello with no goals or Away
  switch, `home-with-goals` has the Solo world's goals, and `home-away`
  has four decisions still open to Overturn, where the mock's first is
  already landed. `needs-one` has no goal templates; `goals-reference`
  adds the large goal; chats and settings use the Reference and Solo
  worlds' parties, accounts and templates. Their counts, words, available
  tabs and resulting page lengths differ because the gallery draws those
  domains (`tools/gallery/src/samples.rs`), not the mock's stored state.
- **Not yet:** 7.2: `goal-sending` has one pending message and "Being sent",
  but no earlier accepted message or next words in the composer.
  `goal-toast` answers Pause without delivering a paused task document, so
  its toast says "Paused" while the header and acts still read running and
  Pause (`tools/gallery/src/samples.rs`, `goal_domain`).
- **Not yet:** 2.2: no test shows a head waiting for jig's mapping's room
  ended by the stall deadline: `temper-service`'s
  `a_call_past_the_mounts_room_waits_until_its_peer_closes` ends it by its
  peer's close.
- **Departs:** 2.5, *The first paint*: handing over fades in 160 ms, where this section
  has 200: `loading.css` takes `transition: opacity .16s` from the request
  `bundle-loading`'s own stylesheet, whose message says 200 ms, because
  `../draft/look-requests.md`, 3 lands a request's rules as its stylesheet has them. The
  loader removes the placeholder at 200 ms all the same. `#boot` also sets
  `letter-spacing: normal`, beyond that stylesheet, since an arriving `temper.css` would
  otherwise move its words by -0.18 px.
- **Departs:** 2.5, *The first paint*: "no byte for 15 s" holds only once the module's
  answer has begun: a stall before that answer draws the mark and an empty bar, with no
  line (skein's `ui-web.status.md`, 4.3).
- **Departs:** 4.3, 4.6: the tier holds 14 tests, two past the cap, and 4.6 names
  12. `browser_call_line.rs`'s `a_run_of_calls_opens_to_its_rows`, from the request
  `chat-reading-b`, has no line there, and its file's header names the story, not what
  only a browser shows. The request `browser-trim` has every test read against the
  tier's rule, but that plan's calls were made before this test reached its trunk, and
  no reading of it is recorded. `browser_accounts.rs`'s
  `the_welcome_sign_in_opens_a_tab_and_moves_on_to_ready`, which 4.2 names at the
  request `sign-in-new-tab`, has no line there either. 4.3 has stories that share a
  setup become one test first: the call line's and test 14 both start a chat on
  `temper box`, though it scripts its own model's turns, and the welcome's and test 9
  both sign in on `Tier::accounts_sign_in`. No reason is recorded for either.
- **Not yet:** 4.3: no run on record shows the tier as it stands within its budget
  (`tools/tiers/budgets.toml`, `browser-temper`, 85 CPU s): its total with the call
  line's and the welcome's sign-in's tests is unknown. The `browser-trim` plan's three
  runs, at `970302c050`, before they reached its trunk, took 78.557098, 73.801674 and
  57.755442 CPU s over 12
  tests, on one slot of a shared machine whose load rose from 6 to 31, where its request
  has a slot as idle as can be had. Those totals are in `tools/tiers/design.md`, 7.2's
  scope, which live sampling of the same tests read higher (that document's section 8).
  The `bundle-loading` plan's run of 13 tests, one fewer than the tier holds, with the
  placeholder on every page but the playground's, took 83.069911 CPU s with one red, against 61.048855 with
  none before it, and no figure splits the page's cost from the machine's load
  (that plan's figures on the browser tier, at `668f7cd3e5`). The red is
  `browser_public`'s `a_visitor_reads_a_public_project_and_loses_it_when_made_private`:
  `temper ctl`'s `project amend --visibility public` outlasts its ten-second deadline
  with the page at Ready and the placeholder gone, in that run and alone. Focused tests
  of the bundle's answers, the mapping's room and ctl's wait for its `Me` pass and don't
  reproduce it; the listener's eight connections, which a page's parallel files share,
  are a hypothesis, no cause observed (the same figures, on the ctl wait).
  Its repair waits on the tier's run after landing (the user, 2026-11-06), and no later
  run is recorded here.
- **Not yet:** 4.6: the request `browser-trim` has a person's page slower to settle as
  the answer most wanted. Settled chat and task pages stood still and no product cause
  was found, but a chat closed on the box took 8.539 s to reach the Closed filter, a
  wait that plan located and did not split, and test 2 has since lost the step (section
  5).
- **Design departs:** 4.3, 4.6, from the request `browser-trim` of 2026-11-06, decided
  by its plan's cut. Goals' link's target is held by the view's
  `the_link_shows_with_a_tracked_template`, where the request has the client world,
  because the view is the lowest tier that shows it. Story A's download is compressed
  by the machine's `gzip`, where the request looks in the playground for a missing cache
  or a needless rebuild alone, because `skein_http::gzip::encode` in the dev profile
  took 4.85 CPU s of the story; so the printed figure's bytes changed (`../playground.md`,
  9).
- **Departs:** 7.2: `start-goal` is Chats with no sheet and no filter,
  `Chats { filter: All }`, since that is where Goals' entry goes and `OpenStart`
  opens nothing without both a draft and a chat; `start-goal-draft` and
  `start-goal-refused` draw the same sheet a chat's "Start goal…" opens.
  The rulings of 2026-11-05 on the request `ui-goal-page` send the entry to a
  Drafts filter (`README.status.md`, 3.5, *Goals*, 5.14).
- **Not yet:** 7.1: a `home-*` state's title is its headline, then
  "temper", where the shell's is "Home · temper" (`README.md`, 5.1):
  `tools/gallery/src/lib.rs`, `heading`, reads the level-1 heading, not
  the view's title.
- **Departs:** 2.5: Home uses the greenfield mock's own scale, as the
  project's frame and Goals do, where the general main's scale says
  14/12: its column is 720 px, body 15 px, headline 32 px (25 px on a
  phone), facts 13.5 px and byline 12 px. The greenfield target is the
  user's look (`greenfield/README.md`); `temper.css` scopes these rules
  under `temper-home`.
- **Departs:** 2.3: `?forge=elsewhere` still reads, as A Forgejo
  elsewhere's Details (`temper-client-domain`'s `address.rs`
  `forge_choice`), words for a forge added to the configuration by hand,
  which names none (`forge.status.md`). 7.2: `google-sign-in`,
  `forge-details`, `forge-setting-up-middle` and `welcome-setup-failed` are
  states the gallery holds (`tools/gallery/names.txt`,
  `tools/gallery/src/samples.rs`) that the table does not list.
