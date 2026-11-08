# README: status

2026-11-08. Where temper's code lags or departs from [README.md](README.md).

Section 5.9's run-ended activity line names the invalid contract kind or
says its result could not be read. The view's renderer table distinguishes
these endings. No known gap remains in this line.

Section 5.5's declaration refusals for name taken and not offered, and
resume's not offered, have their reason and next step in the account
outcome line. ConnectModel checks the person's own same-provider source
on name taken and leaves not offered visible. View-table rows cover
the words; the domain table covers that recovery distinction. The
joined ctl story covers a forgotten name's revival and host-check
dispatch, without establishing a real provider's login or rendered look.

- **Departs:** 5.5, from `ui-account-again`'s request for each fixable
  account-act refusal to say why and what next: the plan limits the new
  words to those three cases. Key and sign-in acts retain generic
  not-offered words because that refusal combines several core causes.
  Distinguishing them remains section 11's work.

Sections 3.5 and 5.3 have Ask temper at `/projects/<project>/ask`,
in the rail after Chats and as Ask in the phone tabs, returning to the
newest open Ask temper chat. Template 5 starts its chat; without it no
chat starts. Section 5.13 offers neither configuring chat nor Ask temper
on Settings. The opening retains the configuring page's starters and
reference, as the renamed-page draft specifies. The post-landing walk,
views and preferences remain section 11's work.

Section 3.5's settings-proposal call names distinguish failed attempts from
created proposals, including mixed outcomes. The view's
`failed_settings_proposals_do_not_claim_a_change` holds single, repeated
and mixed calls and failures after two or eight reads, with failure
descriptions and rows retained. No known gap remains in that wording.

Sections 2, 6.11 to 6.16 and 7.1 have no known implementation gap in the
TUI's command, private link file and sign-in note, host seam, automatic
local sign-in, quiet fallback, bounded rereads, expiry renewal and
suppression after a noted sign-out. The expiry is noted once the link
is live with a party. The base uses the terminal's default colours,
focus uses the accent, and the run chooses its scheme from COLORFGBG,
else OSC 11 with a 200 ms wait, else dark, before Start. Section 10's
frame goldens, host rows, simulated loop and joined-client stories are
present. Section 11 keeps the flags, design-location cleanup, terminal
and macOS checks, later surfaces and the expiry/revocation ambiguity;
no real-terminal or macOS runtime result is supplied here.

Sections 3.5, 4.6, 5.8 and 5.13 have the approvals watch and wire document,
the merge into Needs you, project and goal filters, Home and goal bands,
rail and banner totals, and Later under the change's task key. A kind
filter takes no approvals; the terminal inbox stays unchanged. Unknown,
refused or ended watches contribute no rows. The proposal's card and page
say what an owned-Forgejo approval holds against. Client and view tables
cover these states; the joined world holds the Forgejo card and files-page
link. Rendered comparison and real reviews remain section 11's later work.

Sections 3.5 and 5.5 have no known gap in bounds-refusal words above
the composer, in either shell: all ten bounds, differing shortfalls,
missing figures and refused-hello figures have view-table coverage.
Waiting-line pointers name no ctl call, and their help pages are checked.
The after-landing check and the non-waiting forge-key command pointer
remain section 11's later work.

Section 3.5, *A chat's heading is short*, has no known implementation
gap: one view function makes the first sentence or line, cut within 72
characters, for both shells. The whole numbered name and conditional
whole-title hint remain, and the crumbs take `flex: 1 1 320px`. The
focused tables cover the short forms and both shells' headings, including
a title at its text bound. The `chat-title` account is commit
`4283afdd91`; its render comparison and a possible model-written title
remain section 11's later work. No rendered-equality result is supplied.

Sections 3.5 and 5.5 have no known implementation gap in naming accounts
by email on Ready, the Model step, the model-check pane, the picker or
Accounts, including terminal words and the display/source-name fallback.
The real-login render check and browser and process stories remain
section 11's after-landing work; no real-provider result is supplied here.

Section 3.5 has no known implementation gap in drawing a Markdown flow
in a chat or its document. The page routes skein-ui's flow owner and
names under temper's limits; its host test checks the case's SVG and
the fallback at a narrow width. Temper/ui's look at both places remains
section 11's after-landing work.

Sections 3.5, 5.1 and 5.10 have no known implementation gap in Home's
single "Nothing needs you.", the three failed-link paragraphs or the
semantic-repair line "Repairing: CI failed at its updated head." The view
table checks each link paragraph in both shells, with and without a
configured sign-in way. Cookie fallback and command refusals remain
section 11's later work.

Section 3.5's terminal frame, commands, kept look, full screen, document,
questions and compact cards have no known implementation gap beyond
the departures below. Every terminal act, look restoration and closing
of an open command list advances the pages' shown count. The domain's
existing table checks the count across acts, restoration and full-screen
departure, including a list with no command words left in the box.
The redraw repair account is `703628ea00`: it corrects the Escape
account's claim that Enter already drew the terminal in its press's step.
The four request accounts are `2f653fffa3`
(`chat-terminal`), `a00892e4f2` (`chat-terminal-font`), `cfe973620f`
(`chat-terminal-draft`) and `2a62e07db5` (`chat-terminal-esc`);
compilation and print checks establish no rendered equality. Temper/ui's
render comparison follows landing (`../draft/look-requests.md`, 4 and 5;
section 11). The ground, scrollbar and foot repairs are recorded in
`browser.status.md`, 2.5. The ordinary bar's two-row title at 1000 px
remains intentional: `>_` and the status words leave the path less than
its 320 px, as the user accepted in `look-round-17`.

## Gaps and departures

- **Not yet:** `ui-ask-m1`'s `proposal-read` part was never designed,
  according to the retirement brief. Its landed run proposal read needs
  design coverage reconciliation against `jig/docs/design/domain/engine.md`,
  7.3; this is a design gap, not an absent implementation.

- **Departs:** 3.5, *A chat's terminal*: the quiet line says "N spent · of B",
  because `temper-client-view`'s `pages/terminal.rs`, `under`, reuses
  `Word::ChatbarSpend`, including the budget, rather than own spend alone.
  Temper/ui owns this remaining words gap.
- **Departs:** 3.5, *A chat's terminal*: a settings proposal's compact line
  keeps "Accept" and "Change something", with Reject on Needs you, rather
  than the terminal paragraph's "Reject…". `cards/proposal.rs`, `settings`,
  shares section 5.13's controls; the departure from `ui-chat-accept` is
  recorded under 5.13 below. Temper/ui owns their reconciliation.
- **Departs:** 3.5, *A chat's terminal*: command prefixes match case exactly,
  where the request's `commandsFor` lowercases its query; the domain and view
  share the command table's literal names. A press on the terminal's background
  has no handler to focus the box, where the request's click handler does;
  focus follows the shared web shell. Temper/ui owns both differences.
- **Design departs:** 5.1, from the request `ui-small-1` of 2026-11-06,
  item 1's unconditional used-link explanation: the engine remembers the
  last 8 used or expired links only until restart. A forgotten used link
  reads "This link isn't from this temper. Use the one its terminal
  printed." The plan chose bounded memory rather than stored history
  (jig's `protocol/client.md`, section 8).

Sections 3.5 and 5.8 have no known implementation gap in a person-addressed
report task reading as a question under Needs you, on Home or in a goal's
band. View tables cover its context, choices, recommendation, fallback and
requester link. Domain tables cover choice, reply, Undo, refusal and the
shown revision surviving a refresh; the terminal's Answer sends words too.
The focused sheet-to-answer stories check a question past the document's
text bound, its whole `TaskText` read, a choice's press and those words
reaching the resumed goal, from a chat's sheet and from Goals. Sections
3.5 and 5.8 have no known implementation gap in the whole question's
context, evidence and choices, or the cut notice and link when the read
ends short. The inbox filter remains person tasks;
role-addressed tasks and other contracts keep their ordinary cards. Browser,
walk and render checks remain section 11's after-landing work.

Sections 3.5, 5.8 and 5.14 have no known implementation gap in Goals'
Start goal sheet, its typed Draft path and following Name, its zero-funding
refusal and open-chat explanation, or counting only landed change results
in the goal's task bar and fold. Fixed-amount ways still use the funding
judgement. The funds card offers a waiting budget proposal conditionally,
otherwise closing and starting again once the pool has room. Domain and
view tables hold these rules; the gallery's `start-goal` draws the sheet.

- **Design departs:** 3.5, 5.8, from `ui-goal-asks-2`'s whole-question
  ruling, by its cut: only the first question on a goal's page draws whole;
  later bands keep the bounded first line and Answer, to price at most one
  `long_text` question per page (5.4). Needs you draws the selected question
  whole. Section 11 keeps extending the bands open.

Sections 3.5, 5.5 and 5.9 have no known implementation gap in the closed
chat's header, closed agent's taskbar or the composer's named template
default and effort. The view table covers both shells, including a turn
with unknown effort, an unoffered model and the unread tail. The joined
client story carries smith's turn bytes through decoding to `T1 · model`
in the terminal's closed header. Browser and real-model checks, the goal
page's line and latest-run metadata beyond readable turns are section 11's
later work; the phone keeps its hidden byline.

- **Design departs:** 3.5, from the request `ui-model-line`, which asks
  what the latest run used: the line takes the thread's last readable
  named completion, then the task's choice or template preview. A run
  with no committed turn, an unread last turn beyond the tail page or
  an undecodable or nameless newest turn can leave an older model or
  a preview. The plan chose the carried turn bytes, avoiding a stored
  field and wire change; neither the fallback nor `caught_up` proves the
  latest run's model. Section 11 keeps the remaining work explicit.

Sections 3.5, 5.3 and 5.13 have no known implementation gap in an
Ask temper chat's draft offer regardless of kind 7, drafts on for a
fresh project, or Goals in the bar while `Domain::has_goals` holds. The
draft offer follows the first message and retains its document and end
conditions. The view tables cover the draft offer and its act in both shells, and Goals in
the rail and phone tab bar once To begin is gone.
Stored drafts-off values still govern other chats. The fake-model journey
is `chat-draft`; the after-landing check is section 11's later work.

Section 5.5 has no known gap in the box's capacity-wait words on either
chat surface. The deployment's `owned` read selects them; before that read
answers, or when false, the multi-host words remain. A box without an
`owned` part is deferred in section 11.

Sections 3.5 and 5.13 have no known implementation gap in replacing a
proposal through "Change something", drawing live application rows from
the review under its key, naming templates and proposal activity in the
person's words, or placing Latest in flow on a phone. The joined client
world holds replacement and the Goals row through the engine; its stale
Accept is dropped before jig, whose core world checks the refusal. The
phone test asserts CSS rules, not measured geometry or a press. The
after-landing checks and ended-card row retention are section 11's later
work; no new real-model or phone press outcome is supplied.

Sections 3.5, 4.4, 4.6 and 5.2 have no known implementation gap in the
`connect-flows` shared model and forge panels, completion on Accounts or
Forges, or Settings' three-source repository sheet. Search and scroll,
plural picks, make/copy and failed-row retry and dismissal are present.
The flow paths name the chosen model way, Forges offers "+ Connect a forge",
and the sheet draws the query in its empty row, hides the completed empty
search's count and keeps the dot in More's summary. Copy public has a
browser caption over its named field; the terminal prints the field's name
once. The repair account is commit `73fd6f4bae`; it establishes no rendered
equality. Section 11 carries temper/ui's comparison of the request-owned
parts, including these controls, and the Home sheet follow-up. The browser
stories retain account history, scroll/search/adoption and 420 px sheet
geometry, with no passing runtime result supplied here (`browser.status.md`,
4.6).

- **Design departs:** 4.4, from `look-round-14`'s request for Copy public's
  `rows=4`: the field takes a 107 px minimum height in the stylesheet,
  because skein-ui carries no rows attribute. The plan's cut chose the
  four-line box by rule; its rendered height remains section 11's
  after-landing comparison.
- **Not yet:** 3.5, 5.5: no supplied record establishes a real-box walk of
  the offer, effort choice, Ask temper's chooser and connected-account
  starters with a real provider. The `model-picker` account at `1d82a937df` records the picker
  geometry at 1440, 1280, 1000 and 420 px and fake-box model and effort presses,
  with the stored choice surviving a reload. It supplies no real-provider walk.

Section 3.5's Models tab, switches, add-by-name form, source, estimated
prices, omitted count and All models' link to the project's Models tab
have no known implementation gap apart from the count's delivery below.
Provider, the name and Add share one
unwrapped row with the field's label visually hidden; its accessible name
remains. The Provider and Add section headings have 6 px bottom margins.
The client world holds switches and adding across reloads. Page 07's
render comparison at 1440, 1280, 1000 and 420 px belongs to temper/ui after
landing; it and browser and real-provider
checks remain section 11's deferred work, with no outcome supplied here.
Section 3.5's provider-list source clauses, Read again, Reading and read
refusals, and the expiring new labels on Settings and All models have no
known implementation gap. View tables and gallery states cover these
words and expiry; the client world carries reads, failures, retries and
retained names through the engine. They establish no rendered equality.
The one-process world's ModelsList row carries the key read through a
worker and hosted agent and checks one request, the new name in `me`,
and unchanged accounting. No real-provider result is supplied;
section 11 keeps that observation and the page comparison after landing.

- **Not yet:** 3.5, *The `Models` tab*: the omitted count can stay stale
  when a provider read changes that count alone, because jig does not
  mark `me` for it (`jig/docs/design/domain/engine.status.md`, 17.12).
  A fresh Frame supplies the count; the view draws what it receives.

- **Design departs:** 3.5, *The `Models` tab*, from the look request `model-picker`'s 07, for
  now and on purpose: an address under the project, `/projects/<project>/settings/models`, as
  `me`'s offer is a project's, where 07 draws `/models`; and "the default" on one row, as `me`
  names one model for a template, where 07 also draws "the default without Claude".
  The lede describes the offer and picker rather than 07's automatic-read
  explanation; the provider's source clause and Read again carry the read's
  state. The subscription's clause is "These come with temper.", rather than
  "Claude Code has no list to read: these come with temper.", as the design
  specifies. The placeholder is `model-name` rather
  than 07's hand-made `claude-haiku-5-5`, since the selected provider can differ.
  The final line keeps `.temper-settings-facts` with `.temper-models-ask`,
  rather than 07's `.temper-settings-detail`, as the design specifies; its
  link, "ask temper.", includes the final period, where 07's leaves it outside.

- **Design departs:** 3.5, 5.8, 5.11, from the greenfield target: "Pause", "Resume" and "Close" on a chat, a goal and an
  agent's task, where its goal says "Stop", which meant a goal's end and a chat's hold. The user's word, 2026-11-06, the
  request `ui-pause-close`.
- **Design departs:** 5.11, from that request, which has an agent's task's "Stop task" and "Stop this task?" become
  "Close task" and "Close this task?". That act is jig's hold, which "can't be reopened" would misstate, so the hold is
  "Pause" and "Resume", sent at once, and "Close" is `Ask::Cancel`, on the sheet the request names. Its cut decided it,
  and section 11 keeps the sheet on Pause as a candidate.
- **Not yet:** 3.5, *A person's pause*: no browser test walks a Pause, a Resume or a message that resumes. The client's
  world holds them on the engine's root, with no browser (`a_chat_pauses_resumes_and_a_message_resumes_it`,
  `a_delegate_paused_from_its_page_is_held_then_resumed`), and the check that request asks on a fake box leaves its
  proofs on a builder, outside the repository, so no record here shows it ran. A refused message is held on an answered
  refusal in the domain (`a_refused_message_to_a_paused_chat_stays_in_the_composer_s_refusal`) and by jig's all-or-none:
  no world moves a hold's revision between the read and the send. "Paused" on Home is seen by reading: the view's tests
  draw it on a Chats row (`a_paused_chat_s_row_says_paused`) and none on Home.
- **Not yet:** 3.5, `ConnectModel`, *The sign-in's tab*: only Chromium has run it, and against skein's fake issuer
  alone. Firefox's and Safari's gesture is read and not run (skein's `ui-web.md`, 14.3), and no walk on record has a
  provider's own page in the tab: the request `sign-in-new-tab` of 2026-11-06 leaves a real box to the user.
- **Not yet:** 3.5, `ConnectModel`, *Its end*: no test has a slow sign-in move on once a hidden tab's stream is opened
  again: `skein-ui-web`'s `tests/hidden.rs` holds the letting go natively, and each look of the browser tier's brings
  the page to the front within the minute (skein's `browser.md`, section 6). On the welcome, no test has a paste's end
  move on: `watch_success_replaces_a_pending_paste_with_ready` runs on `ConnectModel`, and `route.rs`'s
  `welcome_model_progress` reads the same for both. Seen by reading.
- **Design departs:** 3.5, *The sign-in under way* and `ConnectModel`, *While it waits*, from the request
  `sign-in-new-tab` of 2026-11-06, by its plan's cut and its account. The terminal draws the note's plain title,
  its muted line and "Open the sign-in page" above the folded paste, where the request has the terminal as today:
  one function draws the page for both shells. `ConnectModel`'s disclosure takes auto side margins, where the
  request's rule has none, so that it keeps its note's edges under that page's width. The note's "‹ Back" goes to
  Choose, where the request's page, made by hand from Choose's, goes to Forge.
- **Departs:** 3.5, `ConnectModel`, *The paste*: a paste answered *finishing* keeps the note's "Waiting for
  ChatGPT’s sign-in, in the new tab" and its link, and reads "Finishing your sign-in…" in the disclosure, where the
  welcome's note reads those words alone (*Finishing a paste*): `temper-client-view`'s `pages/connect_model.rs`,
  `body`, draws `sign_in_wait::waiting` while the sign-in runs, whatever its paste. No reason is recorded.
- **Not yet:** 3.5, *while it runs*, which the request `chat-reading-b` asks of a
  terminal and a browser both: no test takes the call under way from the engine, or
  draws it running in a browser or a real terminal. The client's world writes the
  run's `call` into the delivery the client hears, its joint world's decoder being
  unable to leave a call unanswered (`temper/tests/client/tests/suite/call_line.rs`,
  `outstanding`). Chromium's story opens a committed line alone, since the demo's fake
  LLM holds text pieces, not calls, and a held text ends a run
  (`temper/tests/client-browser/tests/browser_all/browser_pages.rs`, its history's test), and the real
  terminal's reads a committed line alone (`temper/tests/real-loop/tests/suite/terminal.rs`,
  `a_person_at_the_terminal_reads_a_run_of_calls`). A browser's running line is seen in
  the gallery's `chat-work-running`; a terminal's in the view's tests and that world.

Sections 5.9, *What scrolls*, and 5.15 have no known implementation gap in
the browser's agent-page log holding the column from brief through plan,
with Latest after it and the amendment's editor above it. A goal's untracked
page uses the same page. The stylesheet table holds which element scrolls;
the view tables hold the browser's nesting, the terminal's order and the
result inside the browser's log. Section 11 keeps the after-landing keys
and layout check, opening at the end, and the unqueued PgUp at a pending
read's head explicit.

- **Not yet:** 5.9, *The conversation*, and 5.15: no browser test draws an agent's running line,
  presses its "Latest" or scrolls its log, so that a press reads on to the end, the log following,
  is seen by reading. The gallery's `agent-work` draws the line and the pill, and the domain's and
  the view's tests hold what a press reads and when it binds. The request
  `ui-agent-keys` asks for no new browser test; its keys and layout rewalk
  remains section 11's after-landing work, with no outcome supplied here.
- **Departs:** 3.5, *while it runs*: a call under way with none ended draws a
  description too, its words twice ("Checking template 2", "checking template 2"),
  where the section has the line alone: `temper-client-view`'s `pages/chat_line.rs`,
  `Run::flush`, takes the description from any call under way, and the client's world
  (`an_agents_page_says_the_call_under_way`) holds it. No reason is recorded.
- **Departs:** 3.5, *a failed call* and *while it runs*: a run still out with a failed
  call keeps `temper-work-failed`, so its call under way stands in the signal's colour
  after the sheet's comma, and the line says no "1 failed" until the run ends, its row
  alone saying so (`pages/chat_line.rs`, `Run::flush`; the colour seen by reading the
  sheet). No reason is recorded.
- **Not yet:** 3.5, *the words being written* and *While it works*: no browser test
  holds the writing mark's placement for Markdown beyond paragraphs. The
  `ui-pgup` render proof at `e7bd0179d9` holds paragraphs through streaming and
  commitment at 1440 and 420 px, with focus, draft and caret kept; it does not
  settle the fence and open-marker cases below. The request `chat-reading-a`
  has the user read them on a real chat.
- **Departs:** 3.5, *the words being written*, and 5.15: a call or card drawn
  at position zero of a committed turn takes the provisional text's key,
  giving the later text another and replacing its nodes
  (`pages/chat.rs`, `committed`, `block_key`). The plan `ui-pgup` left this
  case outside its fix unless its reproduction saw it; its view test
  `a_committed_redraw_keeps_the_nodes_of_the_words_being_read` commits a text
  first. No render proof records the other order.
- **Departs:** 3.5, *the words being written*. Before its first word the page and the
  dots ask only whether the text is empty (`temper-client-view`'s `pages/chat.rs`,
  `conversation_with_times`, and `pages/chats.rs`, `working`), where `read_growing`
  holds back a marker being written: a text so far of `##`, `**` or blank lines draws
  its message with nothing in it and no "▍", and the three dots go, until its next
  word. A closed fence's code keeps its last newline (`skein-ui`'s `markdown.rs`,
  `line`), and a `<pre>` shows it, so a text ending in one has its "▍" on a line of its
  own, and the block loses that line as the turn commits: the look request `chat-work`
  left this to `read_growing`, which sets the end inside the code, after that newline.
  A last block holding a run still open, or a last line of digits or of a marker alone,
  `read` shows as the characters it is once committed, so that block changes at the
  commit, where only the "▍" should go. No reason is recorded for any; seen by reading,
  and no test reads them.
- **Design departs:** 3.5, *Starting a goal from a chat's document*, and 5.14, from
  the request `ui-chat-draft` of 2026-11-05, decided by the answers that day to its
  tasks `goal-sheet-state` and `goal-sheet`: one sheet, `ui-goal-page`'s, where the
  request draws a chat's own. So the sheet has no Budget field, its budget the
  creating bound's. It has no "Plan, then wait for me" of stock template 2:
  `shapes::set_agreed_goal_from` and `sheet::WayKind::Agreed` stand built, and nothing
  sends them, `sheet::snapshot`
  taking the task-by-task words alone. Its ways are `goals::ways`', the project's
  templates by number with their own names and lines, where the request has three in
  its own order and words. It has "When it needs you", which the request leaves out
  until the away switch exists, as it now does. Its words close with `Draft:` and
  `Chat:`, which the request doesn't name and the goal's page reads for its link back.
  A proposed goal is 5.14's toast, where the request has a line above the composer:
  `Chat.proposed` is set (`temper-client-domain`'s `pages/start_goal.rs`, `told`) and
  read by nothing. No plan holds these; section 11 keeps them.
- **Design departs:** 3.5, *A chat's document*, from that request's decision 7,
  decided by the answer that day to `goal-sheet-state`: `drafts_offered` reads kind
  7's `drafts`, true in the stock (5.13), where the request has it true while `me`
  lists a tracked template with no conversation flag. So a project can turn the offer off.
- **Departs:** 3.5 and 5.3, from the mock (`greenfield/`):
  Ask temper has its own project page, rail entry and phone tab, and
  Settings has no configuring offer. The user's draft, 2026-11-07
  (`../draft/ask-temper.md`, section 9).
- **Design departs:** 3.5, *A chat's document*, 5.13, from the mock (`greenfield/`), whose
  fresh projects have `drafts` false (`data.js`) and whose "+ Draft" follows it on every
  chat (`app.js`): a fresh project's chats write drafts, and an Ask temper chat offers
  "+ Draft" whatever `drafts` says. The user's word, 2026-11-07 (the request
  `ui-drafts-default`).
- **Design departs:** 3.5, *A chat's conversation*, from the look request `chat-work` of
  2026-11-06. A failed call's "1 failed" shares the description with the run's time, both
  in the signal's colour, where the request adds ", 1 failed" alone in it, and a document's
  write ends a run (`../draft/chat-reading.status.md`, 2): each decided by that plan's cut. The words being
  written end in a faint, inline "▍" that does not blink, where the request's overlay draws
  a block 7 px by 15 that blinks: the plan `chat-reading-a` set the mark, this request's
  landing took it at a catch-up, and no reason is recorded. Its pages' line says "Read 7
  docs, checked settings 3×" of calls that start with `settings`, where a phrase follows the
  first call, so the gallery's reads "Checked settings 3×, read 7 docs"; and its 01 gives
  the call under way an outcome, which a client is not given.
- **Departs:** 5.14: a party whose `me` gives the project no creating bound still
  sees a chat's "Start goal…" (`temper-client-view`'s `pages/chat_sheet.rs`, `bar`),
  and a press opens nothing (`pages/start_goal.rs`, `open`;
  `no_creating_or_empty_document_opens_no_sheet`), where the section has `NotOffered`
  said in jig's words. No reason is recorded.
- **Departs:** 3.5, *The Chats page*: the document read stops at
  `limits.client.domain.rows` notes, so a chat beyond those notes has no
  "writes" line and is absent under Drafts. Goal chips cover only the newest
  `rows` chats across the live and closed lists, with ties by the lower chat
  number; a row past that window has no "started goal" line. The answer of
  2026-11-05 to `ui-chat-draft/chats-drafts` keeps this one chips window,
  without raising jig's bound.
- **Design departs:** 3.5, Home, from the greenfield target, by the request `ui-small-1` of 2026-11-06: the mock's empty *Needs you* repeats the headline's "Nothing needs you."; here it says that once, in the headline.
- **Design departs:** 5.3, from the greenfield target, by the request `ui-repo-link` of 2026-11-06: the mock's
  switcher row is one button holding the name and the small words; here the small words are a sibling of the
  button, since a link can't sit inside one, and they are a link with its "↗" where the repository has an address.
- **Design departs:** 5.3, from the greenfield target, by the request
  `ui-rename-focus` of 2026-11-06: closing the rename's form leaves the project
  menu open with the focus on its summary, where the mock draws the menu closed,
  since a disclosure's fold is skein's page part's (`ui-web.md`, 7). The field does
  not select its name on opening, and no toast says "Renamed": the request names
  neither, and section 11 holds them. On a new chat or Ask temper's page the closed form
  leaves the focus on the page's own field, where the request has the opener or the
  summary, because that plan's cut keeps a page's own request. The field's hidden
  label says "Rename", where the mock's says "Project name": no one sees it, and
  that plan's cut took no new word for it.
- **Design departs:** 3.5, 5.3, 5.8, 5.13, from the greenfield target:
  the switcher keeps its inline new-project form; a chat keeps its status,
  Pause or Resume, and Close, and offers the picker only with supplied choices;
  a goal uses supplied budget units, not dollars, and offers neither the
  mock's budget field nor its earlier reports. Settings keeps unsupported
  status, funds and notification controls
  disabled with their explanation. The welcome's Claude card says
  "a config of its own" rather than the mock's "an environment of its
  own" (3.5). Home and Changes omit the mock's "main green at" time:
  their landing panel supplies a red flag, not that timestamp. A
  proposal of settings' card draws no "temper enforces" and "Agents
  follow" line, which 5.13 leaves to the review page. These
  sections retain the client's
  available acts and supplied values rather than the mock's local-only
  operations. Their extra rows and controls change the pages' lengths at
  laptop and phone widths; they are not stylesheet differences.
- **Departs:** 5.1: at 760 px and under, `temper.css` hides the accounts' and the
  playground's banners on every chat's page and `NewChat`
  (`#app:has(> .temper-session) > .temper-accounts`), where the section draws
  them there, because the request `ui-phone` of 2026-11-05 has those pages whole
  at `100dvh`, which a banner above them would cut. With no usable account "No
  model yet · Connect a model" still stands above the composer; no line says there
  that the playground is running or leaving. Seen by reading; no test holds it.
- **Departs:** 3.6, *Closed ends on one line*: a requester-closed chat hides
  every "Decision made" activity line, where the section and the request
  `close-chat` of 2026-11-06 hide the closing's alone, because `Activity::Decided`
  carries only a revision and cannot identify the closing decision (section 11).
  This keeps the quiet end line, at the cost of hiding earlier decision markers too.
- **Design departs:** 3.6, *The sheet*, from the user's request of 2026-11-06 to land
  `close-chat`, which asks no change of behaviour: a Release pressed on an uncertain
  effect before either choice closes the sheet and sends nothing, by the section's
  rule, which that plan's cut wrote (`temper-client-domain`'s `acts.rs`, `send`;
  `a_press_that_sends_nothing_closes_the_sheet`). The view draws both choices alike,
  reading no `again`, so nothing says one was pressed. A Reject pressed with no reason
  keeps its sheet open and says nothing, where `close-chat` has a confirmation that
  fails say why above its acts: no ask went, and the reason's box is on the card behind
  the sheet.
- **Not yet:** 3.6, *The sheet*: that request has each confirmation walked. A browser
  on a fake box pressed Close, Start over, a goal's Close, a Release both ways, a goal's Pause,
  and Accept and Reject on a chat's proposal of settings from *Needs you* and *Home*,
  and Accept from the chat itself; Reject from *Needs you* included a reason. Escape and
  the keeping act on Close, Start over, a goal's Close, Accept and Reject were also pressed
  by hand at each width, which no test keeps; a press while the page updates, on a Release
  alone. The client's world holds the chat's Accept and Reject's ask, with no browser
  (`temper/tests/client/tests/suite/chat_accept.rs`, a chat accepting and rejecting its own
  proposal).
  An agent's task's Close is drawn at each width by its gallery state, the walk of the
  request `ui-walk-3` of 2026-11-06 having tried the fake box for it first, and no walk
  on record presses it.
- **Not yet:** 3.5, *temper's own way, beside the page*: no journey records "See how it works"
  or "Start from it", and no test presses them on a box: the view's tests and the gallery's
  `ask-reference` and `ask-reference-draft` alone draw them. The walk of the request
  `ui-walk-3` pressed both at each width, on a first project and a fresh one, by hand. The fake model
  answers their words with its fallback, so no walk on record has an Ask temper run
  follow them.
- **Design departs:** 3.5, *Ask temper's page* and *temper's own way, beside the page*,
  from the request `configure-temper-own-way` of 2026-11-06, by the user's word that day and that
  request's plan's cut. The picture and the table's Workers and QA stand in
  `../draft/own-way.status.md`. Beside them:
  - the reference's title is level 2 at 28 px over level 3 sections at 17 px, as a chat's document
    has them, where the request's page has a small level 3 title over large level 2 sections;
  - its section 3 says of a goal's branch "whose tests run along the way", where the request has
    it checked "by CI and a review on the way": the reference's goal branches have neither
    (`../draft/reference-code-production.md`, sections 6 and 10);
  - a table's row name is bold in a cell, where the request's rule is `th[scope=row]`, since
    skein's reader draws no row header;
  - the thread's padding of 28 and 8 reaches the opening alone, at wide widths, where the request's
    rule reaches every chat; a phone keeps its 20 and 16;
  - the card's steps are a list with no marker and the pane a region of slots, where the request's
    page has an `ol`, an `article` and `section`s;
  - a phone hides the composer's hint, by the rule every session has there, where the request's page
    shows it.
- **Not yet:** 4.7: the form draws no read-only line on an adopted repository (`temper-client-view/src/forge/page.rs`).
- **Not yet:** 3.5, *The try*: the gallery has no trial past the host's check on
  Ready, and no `accounts-try-stopped` sample (`browser.md`, 7.2). The view's
  tests draw those states; `home-try-stopped` draws Home's failed line.
- **Departs:** 3.5, *The try*, 5.5: simultaneous usable accounts can lose all
  but the first trial request, and account rows share one running task's
  wait and last wait, so a stopped row can say another trial's reason or
  lose its earlier reason (`../domain/trial.status.md`, 6). No reason for
  these limits is recorded. The draft's missing "one short message" spend
  notice remains a design gap (`../draft/model-check.status.md`, 2).
- **Design departs:** 3.5, *The try*, from the look request `model-check-pane` of 2026-11-06,
  each by the cut of its landing: the steps' list has hidden state words and its accessible name,
  as Forge's has, where the request's pages have the same words, because the terminal reads
  them. A line with nothing yet keeps its "Forge" or "Model" label, as the request's words say.
- **Not yet:** 10: a walk from the sheet through planning, a live report and
  delegate tasks, Pause, Resume, a message and Close, which the request
  `ui-goal-page` of 2026-11-05 asks of the browser tier and of a box. The
  view holds Goals' link to `/chats` (`the_link_shows_with_a_tracked_template`),
  and the fake box's walk reaches an
  ended goal's report from its Goals row, in the terminal's client
  (`a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list`). The fake
  box's model (`temper-fake/src/backends.rs`, `model_document`) writes a report
  only for the first chat's goal, task 2, and has it ask, wait or finish with no
  delegates, so it can show no planning and no delegate task: the journey
  `chat-draft` walks a chat's sheet to that goal's page and back to the chat alone.
  The request `ui-final-walk` of 2026-11-05 walked a live report, Pause, Resume and
  Close there by hand, which no test keeps. Of Ask temper, that model scripts
  one proposal, the cue "Show drafts and goals for this project.", and no document,
  and it has no hold of temper's own, so the gallery alone draws those states.
- **Design departs:** 5.13, *The card in a chat*, from the greenfield target's `mock.css`, by the
  request `ui-proposal-2` of 2026-11-06: a proposal's items stand in the card's type, where the mock
  has them mono, so that a template's name reads as a person's word.
- **Not yet:** 5.13, *The card in a chat*, and 3.5's `Chat { task }`: no live turn names its
  proposal, so a live card always takes the place of one no call names, and its call stays a line of
  its own above it. A turn's calls come with no `objects` (`jig-protocol-transcript`'s
  `translate_block`) and nothing fills them, so `temper-client-view`'s `pages/chat.rs`,
  `proposal_anchor`, finds none; only the view's tests, which fill them by hand, draw a card at its call.
  Seen at each width on the walk of the request `ui-chat-accept` of 2026-11-06, its cause by reading.
  The `ui-proposal-card` plan keeps this gap: filling the core's turn record,
  archived turn and call-to-proposal mapping is separate work (section 11,
  *Calls naming their proposals*). The making line supplies the card's place.
- **Design departs:** 5.13, *The card in a chat*, from the request `ui-chat-accept` of 2026-11-06,
  by its plan's cut: that request has the fix reach both of the card's acts, reading "the unavailable
  direct Reject control" as one family with the refused Accept. The section and the mock's `chat.html`,
  the target, draw only "Accept" and "Change something", so no Reject is drawn, and a person rejects on
  the card in *Needs you*. A Reject's ask from that card goes under the proposal's own `project` all the
  same, and only the client's world sends it, by the act (section 11, *A Reject on a chat's card*).
- **Not yet:** 5.4, 5.15: no manual proof on record shows long messages reading
  whole and the over-bound composer in a browser and a real terminal. The request
  `long-words` leaves that proof after landing and adds no browser story because
  the tier is over budget. The joined client's world now reads 66 messages of
  538 and 1,149 bytes through the real engine and link on a chat and an agent's
  page (`sixty_six_long_messages_read_to_their_start`,
  `an_agents_page_reads_long_lines_through_older`); a fresh walk of the reported
  browser history remains unrecorded. The decoder's 512-byte field bound and a
  malformed answer treated as unreachable account for that chain, so its cause
  is no longer unknown.
- **Design departs:** 5.4, from the request `long-words` of 2026-11-06, by its
  plan's cut: an edit past `long_text`, 131,072 bytes, keeps only its UTF-8 prefix
  and the count of characters beyond it in the domain (`pages/composer.rs`,
  `held`). A mounted browser field keeps the full edit until its value count
  changes; a new drawing and the terminal have only the prefix, where the request
  has nothing typed dropped or kept stale. The cut bounds the client's kept edit;
  section 11 keeps retaining the whole beyond that bound open.
- **Not yet:** 5.15: the whole-bound manual reading check is not on record
  (`browser.status.md`, 4.3). The four repaired browser stories passed, with ctl
  history and the smaller window; they do not prove the person's whole-bound
  layout or repair the failed-head-read retry below.
- **Departs:** 5.15, *While a read is out*: a read that failed with the person at the head is not
  tried again on a move back within one and a half screens (skein's `ui-web.status.md`, 7).
- **Not yet:** 3.5, *A chat's bar*: on that walk "Start goal…" went once from a first
  chat's restored document, at 420, after a round trip through working notes, with no
  error and no setting, repository or goal changed in between. Eight other fresh boxes
  kept it, at each width, and `a_page_of_working_notes_does_not_hide_a_later_document`
  and `start_goal_shows_when_it_may` pass, so the cause is unknown and nothing is fixed.
- **Not yet:** 3.5, *Needs you*: an accepted proposal of settings stays the flow's
  card, the rail's count gone, where an answered card leaves the open ones and the
  mock moves it to its answered stack (`temper-client-view`'s `pages/needs.rs`). Seen
  on the walk of the request `ui-walk-large` of 2026-11-06; no test holds it.
- **Not yet:** 5.3: the rename's form asks the focus, and so does an open reply:
  `temper-client-view`'s `pages/ask.rs`, `reply_form`, calls `b.focus(0)` with no
  guard, where `pages/chat.rs`, `document.rs`, `settings.rs`, `new_chat.rs` and
  `ask_page.rs` ask only while no rename is open. So an owner who presses "Rename
  PROJECT" with a reply open on Home or Needs you meets skein-ui's
  `Builder::focus`, at most one per tree, which asserts. The `ui-rename-focus` task
  `rename-row` has opening the form on such a page not panic; no reason is recorded
  for this site. Seen by reading; no test holds it.
- **Not yet:** 3.5, *Away*, 4.6: with the `away` watch refused, ended or not
  delivered yet the switch draws off, but its act is still offered, where 4.6
  has it not offered: `temper-client-view`'s `frame.rs`, `away`, and
  `pages/settings.rs`, `away_settings`, draw their controls enabled, and
  `temper-client-domain`'s `pages/away.rs`, `act`, then sends nothing and says
  nothing, so a press does nothing. Seen by reading; no test holds it and no
  reason is recorded.
- **Not yet:** 10: the renders the request `ui-away` of 2026-11-05 asks,
  `home.html`'s away state beside the client's at 1440, 1000 and 420 wide, the
  switch on and off at each and toasts open. They are saved on a builder, outside
  the repository, so what they show is unknown here; the gallery holds their
  states (`browser.md`, 7.2).
- **Departs:** 3.5, *Away*, its *Overturn*: an `Overturn` whose outcome is no
  answer, unknown after a restart or its link lost, toasts "That was overturned
  already." and sends no message (`temper-client-domain`'s `forge/decided.rs`,
  `overturned`), where the section has those words for a refusal: if it
  committed, the row stands marked and its task has heard nothing. A message that
  fails for any reason, a lost link's too, toasts "It has ended; a new goal can
  undo it." (`pages/away.rs`, `overturn_told`), where the section has them for a
  task that ended: a live task then hears nothing, and its marked row says
  "Overturned: it asks you again" and offers no second Overturn. Seen by reading;
  no test holds either and no reason is recorded. The message's question is cut
  at a character boundary to fit 512 bytes (`forge/decided.rs`, `overturn_words`),
  where the section has it whole, because the smallest deployed composer admits
  that (5.4).
- **Design departs:** 5.8, from the request `ui-goal-page` of 2026-11-05, decided
  by its plan's cut and its proof. A goal's bar is `figure.bar` in the browser
  and `Role::Image` in the terminal, where the request has `div.bar` and
  `Role::Image` for both: skein-ui's `Role::Image` is an image, with no children.
  The planning dots are spans in a `Figure`, not `.typing`'s `<i>`s. A live goal
  with no tasks yet headlines as planning, where the request's words give "0 of 0
  tasks landed." A dependency outside the goal's rows says "waits on task N",
  where the request has its title alone, and the row has none. A goal's counts
  search at most 8 pages for its own line and are then what the page holds, where
  the request has them true past one page (section 11, *A goal's own line*).
- **Not yet:** 3.5, 4.6, 5.5: the walk the request `first-usable-model` of
  2026-11-06 asks: none on record has a real box with one account, unedited, start
  a chat that answers, or a box with no usable account say why it waits. A real
  `temper program-login` needs a person's sign-in, so such a walk starts from a login
  already signed in, or a key, and pastes the printed command only as far as the
  program's prompt. No test runs that command through `temper program-login`:
  `temper`'s `program_login/tests.rs` runs it by `sh -c` against a stub, so that it
  reaches the sign-in for that box's `programs/NAME` is seen by reading.
- **Design departs:** 5.5, *A run's wait*, from the request `first-usable-model` of
  2026-11-06, decided by its plan's cut: a Chats row says "Starting" for a chat
  that can't start, where the request has a run that isn't about to start say why,
  because a row holds no run wait and its query carries no run's check per row
  (jig's `client/domain/client.md`, 19.1). The chat's own page says why.
- **Design departs:** 3.5, *Where the focus stands on a chat*, from the request `ui-composer-focus` of 2026-11-06,
  decided by its plan's cut: a starter's or a suggested request's press asks for the composer's focus on a phone too,
  where the request has no press raise a keyboard the person did not ask for, because the tree has no width and telling
  them apart takes a kit mechanism (section 11). Whether a keyboard rises is unknown, for those and for the request's own
  answers, up after a send and down on a chat opened from a list: no run on record has a real phone, and headless
  Chromium draws no keyboard.
- **Not yet:** 3.5:
  `Board { project }` with its reordering: no
  address in `temper-client-domain`'s `address.rs`.
  5.13: a person with a project's role alone can read kinds 6 and 7,
  but cannot change them on the tab: jig's settings inventory requires
  a deployment role (`jig-domain-core`'s `documents/settings.rs`, `make`),
  so `temper-client-domain`'s `forge/ways.rs`, `act`, has no newest
  revision and sends nothing. The controls appear enabled and give no
  refusal, where the request of 2026-11-05 has a person who is no owner refused
  in jig's words, as one with a deployment role is. With no document to say a
  setting changed, that person reads them again only as the project opens or
  the link comes back. A project-scoped settings inventory needs a contract in
  jig before the tab can use it.
  5.13: Home draws neither "Show goals" nor "See the settings", so
  nothing opens the tab by them.
  5.3: the banner's "Changes", on a page that is no project's:
  the request of 2026-11-05 asked for the rail, and its mock has no banner.
  3.5, *Goals*: a project whose `me` lists no goal template opens no watch
  (`temper-client-domain`'s `pages/goals.rs`, `query`), so its page draws the
  kicker and the lede alone, with no heading and no filters, where the section
  says "No goals yet."; a project with no tracked template has a lede that
  stops at "landed work.", which the section has no words for.
  3.5, 4.6, 8: Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing.
  3.5, *Ready*: two endings leave nothing under "Model none yet"
  (`temper-client-view`'s `pages/connect_model.rs`, `welcome_end`), and no
  test holds either: a sign-in `Made` or `Renewed`, not declined, with no
  account usable, since 5.5 has no words for it and no walk has ended so
  (section 11); and any ending where `me` offers no ChatGPT plan's sign-in,
  since the function finds that way before it draws the ending's words.
  5.5: a plan's first use, its card in the conversation. 3.5, 3.6, 10:
  Delete on a project's settings, confirmed by its name, and jig's
  `RemoveProject` by `shapes::remove_project`, with the chat
  template's grace: no such act in `temper-client-domain`. 3.5, The welcome:
  for a person whose role does not hold the deployment's forges, the model
  alone deciding;
  `temper-client-domain`'s `pages/welcome.rs` `decision` reads a
  refused `forges` watch as no forge, so a usable model opens Forge.
  FORGE in "FORGE · not
  finished" as the forge's name; `forge/connect.rs` `unfinished` prints
  "FORGE" itself. "Go on" after a restart: the row's `app` is the forge
  domain's memory alone (`temper-engine-domain-forge`'s `domain.rs`
  `apps`), so a GitHub forge found unfinished after a restart offers
  "Start over" alone.
  3.5, With no project
  yet, where the document shows the welcome: its heading, words and acts; `pages/chats.rs` draws its page
  alone. There too, with a welcome or without one, `NewChat` at its address
  outside the playground's start: `temper-client-view`'s `pages/new_chat.rs`
  draws its layout where the section has it give way, its line calling the
  missing project "Project".
  4.7: the "not installed" outcome, "temper's App is not installed on
  OWNER.", with "Install on another account": `temper-service/src/family.rs`
  maps `forge::AdoptionRefusal::NotInstalled` to `ForgeReason::NoAccess`;
  the client wire has no not-installed reason, so ctl offers repository access
  and Forgejo collaborator-admin recovery rather than installation recovery.
  This gap is pinned by
  `temper-service/src/tests/setup_github.rs`,
  `unaccepted_and_other_account_adoptions_keep_the_wire_reason_gap`;
  no reason is recorded.
- **Departs:** 3.5, *The first chat*, and 5.3: the rail's "Ask temper", the
  tab bar's "Ask" and To begin's "Start" choose among the rows
  `temper-client-domain`'s `Chats::open_asking` is delivered, whose
  live watch holds a window of `limits.client.domain.rows` by latest
  activity, so an Ask temper chat not closed but past that window isn't
  seen and they open `Ask { project }`; the landings of `configure-temper-name`
  and `ask-temper-place` accepted it, raising no bound for one link.
  3.5, With no project yet, and 5.3: where the document
  shows the welcome, with no project, the banner's place for it reads "No project yet" followed by the new
  project's form, "New project" and "Create" (`temper-client-view`'s
  `frame.rs`, `switcher`), where 5.3 ends the switcher's list with that
  form and 3.5 leaves the first project to the playground's start or
  the welcome's Ready. The code records no reason; it lets a person make
  the project Ready would offer before Ready, the case of the draft's open question
  (`../draft/first-run-projects.status.md`).
  3.5, 4.6, Changes: the queue's first says "checking" at whichever step of
  its turn, and "checks started" gives when that step began
  (`temper-engine-domain-forge`'s `views/changes.rs`, `Change.since`), where
  the request of 2026-11-05 has when its checks began: nothing keeps that time.
  4.6: `changes_view` reads every `ChangeRow`'s repository with `?`, and no
  step erases a closed change's row (`../domain/after-landing.status.md`), so
  once a forge's removal erases a repository with a closed change, no project
  has the document; seen by reading, no test holds it and no reason is recorded.
  3.5, *Goals*: `goals::row` is `pub(crate)` (`temper-client-view`'s
  `pages/goals.rs`), where this section has it public, because its `Bindings`
  is the view's own and Home reads it in the same crate.
  3.5, *Needs you*: a filter's `kind` and `goal` narrow the open cards alone:
  `temper-client-view`'s `pages/needs.rs`, `main`, asks `Session::answered` by
  project, and an `Answered` keeps neither, so under such a filter the answered
  lines and the head's count hold answers the filter leaves out; no reason is
  recorded.
  3.5, *A new chat* and *A project with no repository*: `NewChat`'s line says
  the project has no repository for every `Repository` but `In`
  (`temper-client-view`'s `pages/new_chat.rs`, `main`), so also with the
  `forges` watch refused, ended or not delivered yet, where the section has
  nothing said. The request of 2026-11-05 gives that line two cases and no
  words for not knowing; no test holds it and no reason is recorded.
  3.5, *The rows*: an account's sources are a list under their level-3
  heading, each with its state and acts, and a sign-in under way a paragraph
  (`temper-client-view`'s `pages/accounts/rows.rs`, `draw`), where the section
  has a row's facts as `.setting` rows, because the rows keep their sources,
  observations and forms in document order.
- **Design departs:** 3.5, *Away*, 5.8, 5.12 and 5.13, from the request `ui-away`
  of 2026-11-05, each decided by its plan's cut, for the user to see:
  - 5.12: every clock in the browser reads in the person's zone, where the request
    asks it of Away's 08:00, because `TimeForm::Clock` is one form and read
    universal it would say "Away until 06:00". The offset is read once, at start,
    so a change of daylight saving shows at the next load, and
    `forge::away::next_eight` takes that one offset, so the night the clocks change
    its 08:00 can be an hour off. The terminal stays universal (section 11).
  - 3.5: the chip is a second button in the rail, the phone's top bar, where the
    request has it in the banner: a project's page has no banner (5.3). The
    terminal's is the rail's button, where the request has its `navigation`, which
    a project's page does not draw.
  - 3.5: "Limits" focuses the section's heading, by a count in `Pages`, and a
    browser brings it into view. `#away` is no anchor: `Address` holds none, so no
    address reaches the section.
  - 3.5: the toast names the project, where the request's words have "fabrig’s".
    "Away: agents decide within PROJECT’s limits.", "Away didn’t change. Try
    again." and "That was overturned already." are words the request does not name.
  - 3.5: a row whose task has ended says "landed…" or "ended…", overturned or not,
    its took still struck, where the request's list has "Overturned: it asks you
    again" first: a refused message leaves the row marked, and its task won't ask.
  - 3.5: Home's away form is not drawn while To begin shows, where the request has
    it with any row showing.
  - 5.8: "follows the away switch" stands on every goal whose words hold no line
    `Asks: always`, where the request has it where its requester has a switch: no
    read says that, and a requester with none has its goal told to ask
    (`../domain/forge.md`, 5.9) under those words.
  - 5.13: the tab's "Away switch" row says "Hidden" with kind 7's `away` false, and
    a project with a goal, or the party's row on, shows the switch all the same,
    since the request has any of the three show it.
- **Design departs:** 3.5, *A chat's conversation*, its cards, from the request
  `ui-chats` of 2026-11-05, which names a proposal, a held task and a question's
  answer as cards in the conversation: a chat's page holds proposals alone
  (`temper-client-domain`'s `pages/chat.rs`, `cards`), since its held delegate
  is its one line (5.7), and the answer the one line under "You" the same
  section keeps. No reason is recorded.
- **Design departs:** 5.9, and `browser.md`, 2.5, from the request `ui-chats` of
  2026-11-05, decided by its plan's cut: the request retires `look::CHAT` and
  `CHATBAR`, and they stand renamed `AGENT_PAGE` and `TASKBAR`, with `ENTRY` and
  `COMPOSER_AREA` kept, because an agent's task's page still draws them and the
  request keeps to chats' pages.
- **Design departs:** 3.5, *Needs you*, *What the client keeps*, and 5.3, from
  the request of 2026-11-05 for them, decided by its plan's cut, the request
  leaving that judgement to it. The rail's watch opens while a project's frame shows, not once signed
  in: the rail alone reads it, and opened at sign-in it would stale all three
  journeys' recordings. Bare `/inbox` is `/`, the Inbox's unfiltered address
  (`browser.md`, 2.3). The dots are a figure, not `role=img`: skein-ui's
  `Role::Image` is an image, with no children. Money reads "S of L used", not "$S
  of $L": the contract's units aren't dollars, and the request also asks the task
  page's form, which has none. Leave held takes its reason in a disclosure, as
  Reject does, where the request names no text area: `Decision::LeftHeld` needs
  one. A `Choice:` line past `CUE_BYTES` is no choice (`../domain/agent.md`, 8.2),
  where the request counts every such line, since its button could send nothing.
  And " · later" shows on *Needs you*'s mono line as on Home's, where the request
  and the mock have it on Home's card alone: the one function draws both, and no
  reason is recorded.
- **Design departs:** 3.5, *Claude*, 4.6, from the request `ui-walk-2` of 2026-11-06,
  decided by its plan's cut. `program_environment` answers the owner at any time,
  where the request has it during set up, because Connect a model draws the same
  command after set up.
- **Not yet:** 3.5, The welcome's Model step, *The command's Copy*: no browser test
  presses it, since no story in `temper/tests/client-browser` reaches the Claude panel,
  and no walk on record has. skein's `a_clipboard_write_keeps_the_press_and_reports_once`
  holds the part, and `host_tests_copy.rs` holds that the press's own report asks the
  write; no test runs the one line between them, in `carry`
  (`temper-client-shell-web/src/lib.rs`). Seen by reading. Nor does a browser show its
  word change: `the_button_reads_each_label_once_in_the_browser` holds the three names in
  the view's tree, and that a name's patch rewrites the button's text
  (`skein-ui-web/src/page/web.rs`, `refresh` and `content`) is seen by reading too.
- **Design departs:** 3.5, *Needs you*, `.rec`, from the request `ui-copy-once` of
  2026-11-06, decided by its plan's cut. The request has no button, tab or option with
  both a name and a text child in a browser, and a recommended choice's button has its
  name and the tag "recommended", which this section draws: those are other words, not
  the name twice. `no_named_control_draws_a_text_child_in_any_state`
  (`temper/tools/gallery/src/tests.rs`) excepts that word alone.
- **Design departs:** 4.6, *The program's environment*, from the request `box-small-2`
  of 2026-11-06, item 1, decided by its plan's cut, which takes `resolve` as all or
  nothing: with no `HOME` and an XDG variable placing some of the directories, not all,
  `program_environment` (`crates/temper/src/directories.rs`) knows no default and names
  all three, one at its own default among them, where the request has each only where it
  differs. The command finds the same directories either way, and no test holds this
  case: `no_home_names_all_three` gives no XDG variable. Seen by reading.
- **Design departs:** 4.6, *The words of a command for this machine*, from the request
  `box-small` of 2026-11-06, item 1, which has these words in any other command handed
  to the person for this machine: `temper forge run`'s refusal
  (`crates/temper/src/forge.rs`, `run_refusal`) says `temper forge prepare` and each
  directory whole, and the terminal client says `temper open`
  (`temper-client-shell-terminal/src/terminal.rs`), so with another `temper` first on
  `PATH` each names that one. That plan's cut names the pages' commands alone, and no
  reason is recorded. Seen by reading.
- **Design departs:** 3.5, *The flows' paths* and *Builders*, from the request
  `ui-settings-tabs` of 2026-11-05, decided by its plan's cut. The terminal
  draws no path, where the request has each flow add one: the same request
  keeps the terminal's prints' meaning, a path would shift every number on the
  flows, and the terminal's rail holds "Settings". With hosts kept and none
  connected, `.empty` stands above their "Not connected" lines, where the
  request gives none connected `.empty` and no words for that case, so that no
  state is dropped.
- **Design departs:** 3.5, *Goals*, from the request of 2026-11-05: "6 / 9" is
  on the task line (jig's `client/domain/contract.md`, 5.1), where the request
  offered the goal's document or the tree's read, and a row reads its report
  alone, not also its tasks by `parent`: each place needed the same stored
  counter, and the line needs no read per row. The lede's ways run in template
  number order, as the request's words have it, so the reference's read "task
  by task, plan, tasks, land whole", where its mock, the target, draws them the
  other way round: number order is what a project's settings decide.
- **Design departs:** 5.3, from the greenfield target, by the request
  `ui-owner-menu` of 2026-11-06, at the user's word: the rail's foot row is the
  button of a menu with "Accounts" and "Sign out", where the mock draws the
  person's name as plain text, and a browser over 760 px signs out there rather
  than on the `Accounts` tab. `greenfield/README.md` notes it.
- **Design departs:** 3.5, 5.13 and 10, from the request `ui-walk-large` of 2026-11-06:
  - 3.5: a proposal of settings shows in the Needs you of a project's owner who also
    owns the deployment, those who can decide it, by that plan's cut: a project's owner
    alone has no row of it (jig's `domain/engine.md`, 17.6, and its status).
  - 10: that request's browser walk was made by hand, at 1440 and 420 on fresh fake
    boxes, its driver thrown away as that plan's cut has it, so no test keeps it. The
    client's world holds the card, count and Accept (its story of Ask temper's
    proposal of a project's ways reaching its Needs you and its rail), and
    the process tier a person's plan's return
    (`a_plans_project_has_its_brief_on_a_return_past_the_sitting`), past a sitting a
    test's `forge.sitting_ms` sets to seconds, where that request's return is past 31
    minutes, because a real box has no fake clock.
- **Design departs:** 3.5, *Sign out*, and 5.3, from the request `ui-owner-menu` of
  2026-11-06, decided by its plan's cut. A press outside that took the focus to
  another control leaves it there, where the request returns the focus to the row:
  the row would take back what that press landed on. A project-less `Accounts` page
  keeps its "Sign out" in the banner, where the request keeps it at the tab's foot:
  that page has no such row (5.1). Over 760 px the tab's whole row leaves, "Signed in
  as NAME" and the login with "Sign out", where the request moves "Sign out" alone:
  one form holds them, and the sheet hides it, since the view knows no width (section
  11, *The person's login over 760 px*). The rail's "+ New chat" menu closes on a
  press outside too, which the request doesn't name: skein's page part gives it to
  every menu.
- **Design departs:** 3.5, *Settings*, 5.3 and 5.13, from the request `ui-settings`
  of 2026-11-05, decided by its plan's cut. "Sign out did not reach the engine."
  stands on the rail's foot as well as the `Accounts` tab, where the request moves
  it there: a person with a lost sign-out may be on any page (6.6). Kind 7's
  controls are disabled together while one is out, where the request disables the
  one pressed: they write one record, so two at once would lose one. And a
  proposal's card has no application rows for a person with no review,
  including one with no role in the deployment, where the request has their
  values for everyone: the heads omit application items, and the review and
  proposal-value read keep the deployment role because items span scopes
  (`../protocol/configuration.md`, 13.2).
- **Design departs:** 3.5, *The welcome* and *Home*, from the request `ui-welcome`
  of 2026-11-05, decided by its plan's cut. A plan's way at its address, with no
  sign-in asked, keeps its card in the panel, where the request draws a way's page
  for the key, the paste and Claude alone: the request `sign-in-new-tab` took the
  paste's page and the card's way to it, the reason that cut gave, and no plan has
  weighed the card since. A repository Home's sheet asked leaves the step's lines once in and
  reads as any in the project, "ORG/NAME is in PROJECT" and "Chats can change its
  code now.", where the request has each copy ticked and then "Chats can change its
  code now…": section 11 holds it. The sheet alerts past 16 addresses, where the request
  names no bound: the wire and the store need one (`../protocol/setup.md`, 6.7).
- **Design departs:** 3.5, *Goals*, and 5.8, from the mock's `goals.html` and
  `goal.html`, by the request `ui-goal-asks-2`, decided by its cut: a row and a goal's
  headline say "N of M tasks done.", where the mock says "landed", and a goal's task
  ended done with no change has its own "done" line. Counting landings on a row needs
  a stored and a wire field for a small word; section 11 holds it.

Sections 3.5, 5.1 and 5.12 have the browser's approval page, its root
state and actions, the asked time, signed-out card and approved read each
second. Section 7.1's native half asks and starts the take, keeping only
the code and expiry in the root. The domain and view tables, eleven
gallery states and joint-client stories cover the page and an approving
browser's party, with a non-browser refused. Sections 6.13 and 7.1 have
no known implementation gap in the TUI's ask, locally constructed
address, single-argument opener with null descriptors, `Opened` report,
fallback lines, pasted-link replacement or Enter to ask again. Golden
frames and simulated opener rows cover the lines and closing; joined
TUI stories cover approval by the browser's party, credential keeping,
the first screen and refusal followed by another ask. Browser and real
terminal checks remain section 11's after-landing work.
