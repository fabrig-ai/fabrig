# browser: status

2026-11-08. Where temper's code lags or departs from [browser.md](browser.md).

- **Not yet:** 2.5, *Outside a project*: a form's field name, "Code"
  on the approval page and "Project name" with no project yet, is 13 px
  on a 20.8 px line, skein's small label outweighing the column's rule;
  `look-round-22`'s `outside-ground` closes it.
- **Not yet:** 2.5, *Day and night*: at night a page shorter than the
  window shows the placeholder's day paper under the body's ground (the
  welcome at 420 by 900, a 53 px strip); `look-round-22`'s
  `outside-ground` closes it.
- **Departs:** 7.3: the gallery's tests exceed their 1 CPU s budget:
  `cargo test -p temper-client-gallery --lib`, serial, measured
  2.644 CPU s. The overrun lies in `each_state_holds_what_its_name_says`
  and `print_writes_every_page_and_the_bundle`. A gallery cost repair owns
  finding and fixing the cause (`skein/docs/foundation/testing-strategy.md`,
  8.1, question 2); only if that fails do measurements go to the user.

Sections 2.5, 7.1 and 7.2 have the day and night tokens, glow, moon/sun
switch and `night-home`, `night-ask` and `night-chat-split` gallery states;
their prints derive `data-night` from the kept look. The scoped day
palettes preserve Accounts and connection fallbacks; the stronger night
root overrides them. Embedded images and the independent terminal
palettes retain the literals permitted by the token task. No known
implementation gap remains in these rules. Shadows keep their day
colour tokens at night; the night ring tokens retain their own values.
Ask temper keeps its three equal phone tabs, its composer and model
picker raised above them by a selector browsers accept (2.5, *A selector
a browser accepts*: neither stylesheet nests `:has()` in `:has()`),
and the switch stays at the folded rail's foot or a phone chat bar's
right, with the folded rail open or closed. The `home-begin-*` states
cover two steps and the cut navigation; `home-begin-with-needs` and
`home-begin-settings-left` draw neither Needs you nor Chats under
To begin. `ask-first-run`, `ask-typing`, `ask-sending`, `ask-refused`
and `night-ask`, on `ask-first-run`, have To begin unfinished with its
way of working left, so they draw the rail's Home and Ask temper alone
and the phone's three tabs. No known implementation gap remains in
these repairs. The existing history story has the night reload row and
story 11's phone Ask row (4.6), which compares the composer's and
picker's boxes with the tabs' top at 420 px; neither adds a test
function, and no passing run of either is supplied here.
Section 5 keeps the rendered comparison, reload proof, palette refinement,
system preference and night placeholder separate.

Sections 2.5 and 7.2 have no known implementation gap in the
`ask-temper-place` repairs: the bar's tag is visible, its separator is
hidden on a phone, and the spend paragraph stays hidden. First-run
gallery worlds offer templates 1, 2 and 5; the reference world's other
ways use 2, 4, 6 and 7, leaving 5 to Ask temper. The rendered comparison and
the Ask page's later look round remain section 5's work.

Section 4.6's history story awaits Closed pressed after reload and All
pressed after Back by exact button name, each within `WITHIN`, retaining
the address, visibility, focus and call-line checks. No implementation
gap remains in those reads. The client takes its filter from the address;
Drafts arriving before Closed is section 5's deferred row movement, not
a loss of the selected filter. The runtime and whole-tier budget gaps
below remain; the supplied evidence contains no browser-run results for
this repair.

Sections 2.5 and 7.2 have the Models tab's shipped view and gallery states.
Every Settings select has the chevron rule above its scoped backgrounds
and padding. Models' add row does not wrap, its field's label stays
accessible while visually hidden, and the Provider and Add headings have
6 px bottom margins. Page 07's four-width render comparison and screenshots
belong to temper/ui after landing (section 5); the rule checks establish
no rendered equality.

Section 2.5's flow tokens have the acceptance preview's rules under the
chats marker. The page draws the case's scene and retains its fallback;
temper/ui's rendered comparison in a chat and its document is deferred
in section 5. No browser story is added for flows.

Sections 2.5 and 4.6 carry the connect panels and repository sheet.
The source chooser is excluded from the radiogroup grid; the list has
zero margins and its empty row has 14 px vertical padding. Copy public's
field has a 107 px minimum height and a browser caption. Control captions
and connection actions retain their requested type rules. The repair
account is `73fd6f4bae`; comparison of these controls remains in section 5.
The tier keeps twelve test functions: Accounts history and key re-entry
share both sign-ins' setup; sheet scrolling, search, adoption and 420 px
geometry share one test. The repaired versions have no supplied passing
runtime result. Whole-tier budget proof remains below; the full
`connect-flows` render comparison belongs to temper/ui (section 5).

Section 2.4's used, expired and unknown paragraphs are present through the
shared view. Section 4.6's cookie story expects unknown when reopening its
old link after restarting serve; no runtime result is supplied here.
Cookie fallback remains deferred in section 5.

Section 7.2's `settings-all-hidden` sample names every switch off,
separately from the drafts-on stock. The view tables hold Goals in the
rail and phone tab bar with kind 7's Goals hidden and a goal present.
The fresh-project flow's browser check is deferred in section 5.

Sections 2.5 and 7.2 have no known implementation gap in Goals opening
the Start goal sheet with its Draft field: `start-goal` draws that sheet
over Goals. The shared view draws whole person-task questions and their
choices, the cut notice and link, zero-funding words and change-only
landings. Browser, walk and render checks of these rules remain section
5's after-landing work; this plan adds no browser story.

Section 8.3's Landings link uses the underlined row-sized sheet rule, and
landings omit a zero stale-proof count while retaining a positive count.
The view and sheet tests hold these rules. Titles retain the user's
publication decision; no page offers a visibility change.

Section 8.4's public landings check accepts one queued member or none,
followed by no landing or the stock queue's merge. The tier's library
table holds all four forms and refuses malformed identifiers, phases,
times and extra text. The story keeps its other page and geometry checks.
No implementation gap remains for the request `ui-public-queue`; the
public geometry and console-expectation limits below remain separate.

Public history retains two limits the request `ui-public-history` leaves
for later (section 5), which the board's own line states (8.3): goals ended
before publication are absent; a restart empties the ended-goal history
(jig's `domain/people.md`, 13.9). A merge from before publication can still
show when its change is handed from a live goal's tree, as the plan's cut
keeps it; publication is not a cutoff on a handed change's commit.

- **Design departs:** 8.3, from the request `ui-public-2` of 2026-11-06:
  ended rows say how long ago they ended, where the request says "for how
  long", because "landed for 2 hours" would not describe the ending. The
  board shows its Landings section with a queue or a retained landing,
  where the request says only with a queue: the plan's `public-page`
  contract asks for either, and records no reason for widening that rule.

Section 2.5's terminal rules, tokens and bundled fonts and 7.2's nine
`chat-terminal-*` gallery states have no known implementation gap beyond
the departures below. The conversation's terminal ground outranks the
shell's Split and Draft grounds, including phone Split. The pane has a
thin scrollbar and no reserved gutter at every width. The quiet model
button is plain, and the running line is first inside the composer form,
before the box, with its outer margins removed. The existing view table
checks that parent and order, the plain button and Exit's retained token.
The repair accounts are `463459a7d1` (`chat-terminal`) and `078e99c4ed`
(`chat-terminal-draft`); the stylesheet and tree establish no measured
line heights, pane geometry or rendered equality. The shared shell
document and flow rules win over the request's restated strong,
code, marker and figure-margin rules, as section 2.5 requires. Temper/ui's
render comparison follows landing (section 5;
`../draft/look-requests.md`, 4 and 5). Compilation and print checks
establish no rendered equality.

- **Departs:** 2.5, *A chat's terminal*: at 760 px or less, DejaVu is 13px,
  but Plex stays 14px because `.temper-terminal-plex` has greater specificity
  than the phone's `.temper-terminal` rule. The cascade retains the desktop
  Plex size instead of the requested single phone size; temper/ui owns it.
- **Departs:** 2.5, *A chat's terminal*: adjacent local notes share the
  `.temper-terminal-note + .temper-terminal-note` margin, including help's
  code block, where `chat-terminal-font` narrows its mock selector to
  paragraphs. One note token retains uniform spacing. The shared options
  menu also has no mousemove selection handler, so hovering an option
  leaves the active keyboard choice where it is, unlike the request's
  handler. Temper/ui owns these differences from the request.
- **Not yet:** 7.2: `accounts-try-stopped` and the trial's own `welcome-ready-*`
  samples (starting, answering, answered, stopped-starting, stopped-answering,
  waiting, waited, plan and key) are not in the gallery. Its Ready samples
  cover the host's check alone; the view's tests draw the trial states.
  The model-check plan's close added the catalogue rows in section 7.2,
  but not their sample fixtures or names. `home-try-stopped` is present.
- **Not yet:** 8.4: the browser kit returns the first content quad of each text
  node, so the public geometry story cannot check every wrapped fragment.
  Its result needs a browser testing interface decision before it can retain those
  fragments. Missing rectangles remain failures.
- **Not yet:** 8.4: the intermittent missing rectangle's cause remains
  unestablished. The story retains one terminal boxed query per page and
  width, checks its truncation count, and fails on missing geometry with
  the text and a preceding measurement-stage line. Batching narrows a
  changing page's query window without making it atomic; the request
  `ui-public-story`'s timer-driven explanation remains unproved (section 5).
- **Departs:** 8.4: `browser_public` accepts the 401 resource-load console
  text throughout the walk, including its public reads, because a stream
  around sign-out can correctly answer for an ended session. The request
  `ui-public-story` permits this expectation if the sign-out flake recurs;
  it is matched by text, without a stage or resource constraint, so an
  unexpected public-read 401 with those words is accepted too (section 5).
- **Not yet:** 2.3, *A heading drawn after its page*: that a focus placed while a heading
  waits keeps its place, which the answer of 2026-11-06 to the `ui-composer-focus` task
  `composer-focus` asks, is seen by reading `temper-client-shell-web`'s `focus.rs`. No test moves the focus meanwhile:
  `browser_pages`'s `enter_starts_one_chat_and_the_composer_keeps_the_focus` holds the
  heading taking it from the main, and that module has no test of its own.
- **Not yet:** 2.5: the answer of 2026-11-06 to the `ui-rename-focus` task
  `rename-row` asks a test that a long chat's document never exceeds its window at
  1280x720 and 1000x600 with "Older", since gone (`README.md`, 5.15), pressed twice. What holds it is the sheet's
  rule (`temper-client-shell-web`'s `tests/suite/switcher.rs`,
  `scrolling_panes_contain_their_hidden_absolute_parts`), and no test measures the
  page; no reason is recorded. That test names the session's log and draft pane,
  and the session's thread with no log, which scrolls too, is not positioned. The walk
  of the request `ui-walk-3` of 2026-11-06 measured it by hand: the document at its
  window at each size, in Chat and Split, one pane scrolling, but with no earlier read
  ended (`README.status.md`, 5.15).
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
- **Not yet:** 4.2, 4.3, 4.6: the tier holds twelve test functions. Its merged
  tests 2 (with 11 and the call line), 4 (with 5 and the model picker),
  8 and 9 (welcome then ConnectModel, with Accounts history and key
  re-entry), 14 (short, Latest, unanchored and long), and 16 (sheet scroll
  and phone geometry) retain story labels for failed steps. No supplied
  passing run proves all these current merged stories in Chromium. The call line's
  retained open node, the picker's 420 px, one-press model and effort,
  outside and Escape checks, a step changing and restoring anchoring on a
  live page (4.3), and both sign-ins in one browser (4.2) await after-landing
  proof (section 5). The picker's separate render and fake-box proof at
  `1d82a937df` establishes no passing run of the merged story. Whether
  reading's four stories fit one run's forty seconds is unmeasured: apart, their wall medians add to
  25.761 s (`temper/docs/development/browser-budget.md`), each with a setup they now
  share. The long story's replies are smaller than those figures' by the bytes that
  leave room for the position story's in the fake's joint allowance, with no line
  fewer; its bytes alone changed.
- **Not yet:** 4.3: no whole-tier run on record establishes that the repaired
  tier fits its unchanged 85 CPU-second budget. The flagged run, of 17 tests, was
  129.4 CPU s, 70.8 s wall. With ctl history and the 131,072-byte window,
  all twelve final comparison runs passed, but the CPU medians remain
  20.858273 s for long, 9.178371 s for Latest and 8.399148 s for unanchored,
  above the request's targets of about 12, 6 and 5. Short rose to 6.414108 s
  from the baseline's 4.193832 s. Those four sum to 44.849900 s, leaving
  40.150100 s for the rest of the tier; the final cost awaits after-landing.
  Loads, all samples and the limits of the comparison are in
  `temper/docs/development/browser-budget.md`. Backoff failed its CPU and wall
  criteria and is absent; the smaller window alone establishes no CPU saving.
  The listener-full ctl failure is repaired by ending an idle connection while
  preserving calls, partial requests and bounds. Whether that also repairs
  `browser_public`'s earlier visibility-command deadline failure is unverified:
  that story was outside the four selected for this repair.
- **Not yet:** 4.3: no supplied manual proof checks a long chat at the person's
  1,048,576-byte bound. The passing long browser story spans three small
  windows, while the unanchored position history stays within one; neither
  substitutes for that whole-bound check (section 5).
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

Sections 2.3, 2.4 and 2.5 have `/terminal`, its parsed and canonical
`code` query, transport sign-in by cookie, the signed-out state with
Google's way back where it is given, and the approval's code, asked
time and field under the outside rules. Section 7.2's twelve
`approve-*` gallery states are present, with domain, view and shell
coverage. Section 5 keeps the rendered browser check and approval at a
public name for later; no real-browser result is supplied here.

Section 2.5's *Outside a project* has no known implementation gap: the
top bar and the column on the eight tokens and the overlay's rules under
`temper.css`'s outside marker, the placeholder's star in the signal with
no dot, and the old look's tokens, rules and `files/brand.svg` gone.
Focused tests hold every class `temper.css` selects to a `look.rs` name
and keep `#b8664e` out of the bundle's and the server's pages' sources.
Section 7.2's `signed-out`, `out-of-date`, `no-project` and
`approve-signed-out-google` are present. The rendered comparison against
the request's pages is section 5's, after landing.

- **Design departs:** 2.5, from the look request `outside-pages` of
  2026-11-08, decided by its plan's cut: `--temper-accent` and the light
  chat terminal's `--accent` take the signal, `#c2410c`, which the
  request does not draw, so that no source of the bundle names the old
  mark's `#b8664e`, as its test asks.
