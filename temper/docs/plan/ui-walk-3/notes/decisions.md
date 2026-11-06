# ui-walk-3: what is already decided

Carried from the decisions and rulings kept outside this repository. **A departure listed here is not a finding:** the report names it (`D4`, `Ruling 9`, `By page, Home`) and moves on. Don't reverse one. Where a listed decision itself reads wrong in the walk, say so in the report's notes.

**Stale words, by the cut's own check on the trunk:** this list was written before several plans landed. Away and Phone are on main, so read "not started" as "landed". Where a plan that landed since (proposals in Needs you, Home's brief, the rename form, Sign out in the Owner menu, `chat-work`, `sign-in-new-tab`, `model-check`, `close-chat`, temper's own way) says otherwise, `temper/docs/design/client/README.md` is what holds, and a difference it records is not a finding.

## Rulings

1. A person's goal keeps notes of its own scope: its report is a note. ui-chat-draft's note-scope adds chats' scopes and takes none from goals.
2. One line says a run's number: ui-goals' `brief-numbers`. ui-chat-draft reuses it.
3. One Start goal sheet, built by ui-goal-page (start-domain, start-sheet). ui-chat-draft opens that sheet from a chat, the draft filled in, and adds "Start goal…" and "Goal TITLE →".
4. The sheet always writes the goal's `Draft:` and `Chat:` lines, so a goal from a chat keeps its way back.
5. The default way: the project's setting; if unset, the first way in the shared `goals::ways` order. Every list of ways uses `goals::ways`.
6. Ways are the project's templates. "Plan, then wait for me" appears only where a template provides it.
7. No typed draft path. Goals' "Start one from a draft" opens Chats filtered to Drafts.
8. The drafts setting decides "+ Draft" and Start goal: `drafts_offered` reads the setting (ui-settings' ways-session).
9. Hidden means hidden, on Home too: Home's Changes link follows the Changes setting. Goals shows in the rail and on Home when its setting is on or the project has a goal, so that a goal an agent started never disappears.
10. ui-away builds "When it needs you" on the one sheet, with `Asks: always` and its charter sentence.
11. Status row: where a project's rules name no status agent, Home says nothing of an agent; the disabled row stays as drawn.
12. Builders' words: as landed in ui-settings-tabs' design.

## Decisions

### Worth your look

1. **New projects hide Goals, Changes, Away and chat drafts.** A new per-project setting starts all four off, so a fresh project shows Home, Needs you, Chats and Settings. Why: the mock's first run shows no more than that, and the mock's accept-a-proposal turns them on. (`ui-settings`)
2. **A new stock agent writes Home's brief.** It runs when someone opens Home after 30 minutes or more away, if something landed, the last brief is over an hour old and none is running; it can only finish. Why: no timers means no spend when nobody looks, and a failed run leaves the computed sentence. (`ui-home-status`)
3. **Agents must write questions in a fixed form.** A question is one line, then context, then `Saw:` and `Choice:` lines, with one `(recommended)`; anything else shows as a plain card answered in words. Why: it needs no change to the protocol or the store. (`ui-needs`)
4. **Pause means Stop.** Pause stops running tasks at once, and the toast says so, where the mock says they finish. Why: a true "let running tasks finish" isn't built in the engine, and building it is a separate plan. (`ui-goal-page`)
5. **Answers wait 5 seconds in the tab, for Undo.** Closing the tab inside those 5 seconds sends nothing, and the question stays open. Why: the engine can't take an answer back, and an agent may already have read it. (`ui-needs`)
6. **Copies of repositories run at Ready, not during forge set up.** So there is no "Copying…" on the Model step and no "N repositories copied" on the forge's line. Why: your later ruling of one way to ask for a copy, which the engine already does. (`ui-welcome`)
7. **The rail stays on every chat, and folds to a thin strip in Split.** The mock's chat has no rail. Why: your reason from the configuring chat, that losing the rail reads as leaving the application. (`ui-chats`, `ui-chat-draft`)
8. **A goal's report is a note its own agent writes.** Goals' agents gain the note tool, and their instructions now ask for a report after planning and whenever things change. Why: no new record or read; a goal that skips it shows fallback words. (`ui-goals`)
9. **A draft lives only with its chat.** The mock's loose file drafts are out, and a draft's shown path, `docs/design/draft/NAME.md`, isn't true until a goal commits it there. Why: listing files would need a repository read nothing else uses. (`ui-chat-draft`)
10. **Agents decide while you're away through a new `decided` tool.** Implementation, retry and wording choices may be decided, dependencies wait unless ticked, and funds, sign-ins, machines, plan go-aheads and held tasks always wait. Why: the engine stays out of answering, and a question that skips the tool just waits. (`ui-away`, not started)
11. **Status, Funds and Telling you are drawn in Settings but disabled.** Each shows what temper does today, under "Not built yet: this is what temper does today." Why: a working-looking control with nothing behind it would lie, and leaving them out would break the page's shape. (`ui-settings`)
12. **Claude Code is downloaded on first use, about 290 MB, and checked less than Forgejo.** Later starts check its size only, not its digest, and there is no mirror; the real download has never been run. Why: hashing 290 MB on every command is too slow, and tests use a fake download. (`claude-always`)

### By page

### First run
- First project is "Hello" or "My project", not the mock's "Default".
- "Open Forgejo ↗" stays on Ready's forge line: it is the only door to the forge.
- The forge's pane stays on the Model step (look request); the mock's pill is on the Forge step alone.
- Fresh Home keeps To begin: no count headline, no Connected, "Skip" stays.
- States the mock doesn't draw stay: failed set up, forge cut short, declined, held, refused, paste endings, port taken.
- Claude's way adds a "Claude Code isn't signed in here yet" alert.
- No name field on Claude's way: the name is `claude-owner` on a box.
- A way with no sign-in asked keeps its card: it is how the paste is reached.
- "If this machine has no Claude Code, this fetches it first." shows on every machine.
- ChatGPT's line says "ChatGPT plan" without the email, until the account-facts work lands.
- At most 16 copy addresses; the sheet says so.
- Home's "Add a repository" sheet exists only for the "Nothing for now" state; the full sheet stays later.
- Dropped: `Mockup:` lines, GitHub's away note, GitHub's repository count, `phone.html`.
- Unproved: a real box copying from a public host.

### Home
- "overnight" is only the agent's word; the computed sentence says "since you last looked".
- No lede and no byline until a brief exists.
- "main green at 07:36" is dropped: no green time is kept; "main red" shows.
- The landed count includes merges to goal branches, so it can exceed what reached main.
- A return within 30 minutes keeps the same "last looked".
- The model's line left Home and has no new place: the mock's tab doesn't draw it.
- A project whose rules don't name the status agent gets computed words only.

### Needs you
- Undo also restores a reply's words: more than the mock.
- Leave held takes its reason in a disclosure; the mock has no such box.
- A choice past 256 bytes isn't offered as a button; a reply may be long.
- Accept, Reject, Release and Leave held send at once, with no Undo.
- Later and Answered live in the tab, and are gone on reload.
- Updates leave the page; the terminal keeps them.
- A chat's own questions in words get no card (the mock's `n-license`).
- "Work carries on" replaces the mock's "Goals carry on".

### Goals
- Ways list in number order, the reverse of the mock's: "task by task, plan, tasks, land whole".
- "Plan, then wait for me" and "One agent, one branch" aren't listed: no such template.
- "One agent, one branch" is dropped everywhere: a chat's change is that.
- "Reorder goals" is dropped; Amend's Priority still orders.
- "More goals" replaces the page, as "more closed chats" does; the headline counts rows read so far.
- A goal needing you with no report takes running's or planning's headline.
- "1 goal, 1 under way" fixes the mock's singular.

### A goal
- Task rows are links to the old task page: a held task's Release lives there.
- Dropped: "earlier reports", "goal branch last updated from main", the mock's canned agent reply.
- Dropped from the old page: What was asked, Latest activity, authority, plan tree and filter, dependency lines, model choice; `ctl task show` keeps them.
- Amend opens today's editor, restyled; the mock only has a toast.
- A live goal with no tasks says the planning heading, not "0 of 0".
- Counts stop being exact for someone with more than 8 pages of goals.
- "How it runs" says "always asks you" until Away lands.

### Chats
- Stop and Close stay on the bar: no other way to end a chat.
- Who, clock, day, spend and delegates stay in the view, hidden in the browser, for the terminal.
- The model chooser is the byline; "Manage usage" moves to its panel's foot.
- A playground chat keeps its context line, and the frame keeps its banner: the mock has no playground.
- A row says "started goal X" only for your own starts, not an agent's.
- The chat's "Started goal X" line shows the goal's state, not its way.
- Dropped: ", draft" in titles and "landed" on rows.

### A chat and its draft
- A reopened chat starts in Chat; the mode isn't kept, as the mock's is.
- Draft keeps the composer at the foot on white (look request over mock).
- No drawings: fenced blocks read as code, and agents aren't asked to draw.
- One draft per chat, up to 64 KB, but a box's own model writes about 10 KB in one call.
- Notes stay in the store after their chat, until the project goes.
- "When it needs you" is dropped from the sheet until Away exists.
- A configuring chat has "Start over" instead of "Start goal…".
- The rail's menu lands, though held: "Goal" opens a new chat with a draft; "All ways" goes to Settings.
- `Ctrl+\` is ⌘ on a Mac.

### Changes
- Only the mock's two lists: held, not-ready and paused changes stay out.
- Rows link to the task page; a change has no page of its own.
- "main green" is dropped, as on Home.
- Pulls merged by someone else are logged; pushes from outside aren't.
- A project landing by turn shows an empty Queue.
- The first row's time is when its current step began.
- The banner's "Changes" link isn't drawn.

### Settings
- "Configure temper" is the first row of How work runs; the mock's tab has no entry.
- It stays a page; no docking beside the tab.
- Later, not drawn: roles, ways to start work, rules, running work, history, the proposal review page.
- A settings proposal's card leaves out "temper enforces / Agents follow".
- The "Questions a goal can answer" control is live, off by default: setting it changes how work asks from then on.
- The four switches disable together while one is being sent.
- Sign out moves to the Accounts tab; its failure line stays on the rail's foot too.
- Forges, Accounts and Builders aren't in the mock: its language is applied, with no state dropped.
- Forges and Accounts take the new look with no project too, without tabs.
- Not drawn: repository roles, health, landing rules, Delete.
- Builders is read-only, with no capacity graph; any deployment role sees it, though its refusal says owners.
- The add-a-forge, connect-a-model and adopt pages show a path in the browser; the terminal doesn't.
- "Start over" on Add a forge returns to Forges.
- A project owner with no deployment role sees proposed settings by name, not value.
- The reference case's switches live in `project.json`, applied by a walk still to add.

### Away (not started)
- Decided rows show for 24 hours, on Home alone, at most 7.
- Overturn works only while the task lives; after that, "a new goal can undo it".
- No switch until the project has a goal, or Settings shows it.
- Notifications and quiet hours aren't built.
- Retrying a failing task (the mock's `d-split`) gets the tool's `retry` kind and nothing more.

### Phone (not started)
- `phone.html`, `mock-toggle` and `mock-panel` aren't built.
- No new breakpoint: 760, with the existing 600, 430 and 850 rules.
- Adds `viewport-fit=cover`, which the mock has and the client lacks.
- Changes isn't in the tab bar.
- With Goals shown, Settings is reached through the switcher.
- A draft on a phone is Chat and Draft only.
- No push: "To this phone's browser" stays disabled.
- A page that doesn't exist yet is reported, not built.

### Budgets
- budgets: no "Add $10" or 80% warning anywhere; amounts are plain numbers with no `$`; Release and Leave held are funds' only acts; Goals' start sheet has no budget field and the chat's has one, defaulting to the person's own limit.

### Contradictions

Paths: `PLAN/.../` is `fab/integration/PLAN/temper/docs/plan/PLAN/`; `PLAN/request.md` is `coord/ui/inputs/PLAN/request.md`; `gaps/` is `coord/ui/gaps/`.

1. **Can a person's goal write its report?** `ui-goals/.../plan.md` ("A goal's authority holds notes of its own scope"; on main) against `ui-chat-draft/.../plan.md` and `ui-chat-draft/.../tasks/note-scope.md` ("A person's goal has no note scopes"; its test `a_goal_is_filled_with_no_note_scopes`) and `ui-chat-draft/request.md` ("A goal and an advisor keep "no notes""). By reading, not run: goals started from either sheet would be refused their report. Not merged yet: `note-scope`'s fill isn't on its trunk.
2. **Two lines say a run's number.** `ui-goals/.../tasks/brief-numbers.md` (`[This task: N, in project P]`) against `ui-chat-draft/.../tasks/note-scope.md` (`[You are task T in project P]`). Each plan believes no brief says it. Likely harmless, double.
3. **Two Start goal sheets.** `ui-goal-page/request.md`, 5 against `ui-chat-draft/request.md`, 7; `ui-chat-draft/.../plan.md` says both stay. They differ in ways, words and fields; budgets above.
4. **A goal from a chat's sheet loses its way back.** The goal page's "Draft" button and "Planning from NAME" read `Draft:` and `Chat:` lines (`ui-goal-page/request.md`, 3 and 5); the chat's sheet doesn't write them (`ui-chat-draft/.../tasks/goal-sheet-state.md`).
5. **Three rules for the default way.** `ui-settings/request.md`, 2 and 6 (the setting, first way when unset, and a start preselects it); `ui-goal-page/request.md`, 5 (the setting, else highest-numbered); `ui-chat-draft/request.md`, 7 (first in its own order, no setting).
6. **Is "Plan, then wait for me" a way?** `gaps/goals.md`, recommendation 6 (no template yet) against `gaps/chats.md`, recommendation 10 (template 2 with the agreed line). So Goals and Settings omit a way the chat's sheet offers.
7. **Hand-typed draft paths.** `ui-chat-draft/request.md`, decision 4 ("No file drafts") against `ui-goal-page/request.md`, 5 (a "Draft" field, "docs/design/draft/name.md", when opened from Goals).
8. **What decides "+ Draft".** `ui-chat-draft/request.md`, decision 7 (any goal template; "No `parts`") against `ui-settings/request.md`, 4 and `ui-settings/.../tasks/ways-session.md` (`drafts_offered_reads_drafts`). If the setting wins, a fresh project offers no draft or Start goal.
9. **Hidden, but Home still shows it.** `ui-settings/request.md`, 4 ("Hiding removes the way in") against `ui-home-status/.../tasks/home-words.md` (the foot's "Changes" link, always) and `ui-home-status/.../tasks/home-goals.md` (Goals section by whether a goal exists). A goal an agent starts on a fresh project shows on Home but not in the rail.
10. **Nobody builds "When it needs you".** `ui-goal-page/request.md`, 5 builds it only if `ui-away` has landed, and it runs first; `ui-away/request.md`, Decisions, leaves it to "the goals area". The `Asks: always` line and its charter sentence go with it.
11. **"The agent always runs".** `ui-settings/request.md`, 2 and 7, and `gaps/settings.md`, recommendation 4, against `ui-home-status/.../plan.md` (no brief where a project's rules don't name it). The disabled row could say "Written by the status agent" on such a project.
12. **Builders' words.** `ui-settings/request.md`, 1 ("Where agents run." / "Builders are a separate app…") against `ui-settings-tabs/.../plan.md` ("Where agents work." / "The machines temper's agents run on…", saying no request gave any). The second is in the design.
13. **Where the mock is.** `ui-phone/request.md`, `ui-welcome/request.md` and `gaps/goals.md`, 13 ("not on main") against the others ("on main"). It is on `origin/main` now: stale words, harmless.
14. **Plan overrode its own request, openly; the plan stands.** Status agent: charter 9 not 7, kept not forgotten (`ui-home-status`). "6+ / 9+" not needed (`ui-goals`). `project.json` not `settings.json`; the sign-out failure line stays (`ui-settings`). Forges and Accounts restyled with no project (`ui-settings-tabs`). Plain numbers not `$S of $L` (`ui-needs`).

### Engine and format changes

No compatibility kept anywhere: there is no supported deployment.

**Store**
- Family 30 `LandingLog`: a ring per repository, 16 kept, each with title and branch (`ui-changes`).
- Family 36 `ProjectFacts`: landed count, last landed, the brief, a run's `running` (`ui-home-status`).
- Family 37 `Looked`: per person and project, `previous`, `latest`, `seen`, `next_seen` (`ui-home-status`).
- Family 34 `Dismissed`: its fact becomes `PersonFact`, adding `GoalFrom { project, chat, goal, at }`; bound ×8 and per person (`ui-chat-draft`).
- Family 20 `Setting`: gains kinds 6 and 7; existing stores still decode (`ui-settings`).
- Forge row: first repository becomes `Empty`, `Copies` (up to 16) or `Nothing` (`ui-welcome`).
- Stored task-line counter, `delegates_done` and `delegates_total`; old stores don't decode (`ui-goals`).
- Notes: body up to 65,536 bytes; `NoteWritten` gains `lines`, `added`, `removed`, `first` (`ui-chat-draft`).
- A person's words raised to `tasks.spec_bytes`, past `people.words`; clients gain `long_text` (`ui-chat-draft`).
- Planned: `Away` and `Decided` families, next free bytes (`ui-away`).

**Requests and protocol**
- Watch `changes`, with `ChangesSubject` and `ChangesPanel`; limits `forge.landings_kept` 16, `changes_rows` 32, `changes_landed` 8 (`ui-changes`).
- Request `Looked`, `Watch::Status`, `ProjectStatus` on the wire (`ui-home-status`).
- Status run: a procedure that delegates, under `STATUS_CODE`, kept, at most one a sitting and an hour (`ui-home-status`).
- `SetUp` takes the first repository's intent; `MakeRepository`'s address bound at 512 bytes (`ui-welcome`).
- `Task.made` on the client's document and wire (`ui-goal-page`).
- `NoteScope::Tasks { project }`, reads only (`ui-chat-draft`).
- Pages may send `ChangeSettings`, for How work runs' rows; `Read::Setting` allowed to a project's role for kinds 6 and 7 (`ui-settings`).
- `set_agreed_goal_from`; `GOAL_SCOPE` and `GOAL_AGREED` moved to the wire crate (`ui-chat-draft`).
- New addresses: `Goals`, `Changes`, `NewChat { draft }`, `Repositories`, `Builders`; `PlanFilter` gone from `Task` (several).
- Planned: `SetAway`, `Overturn`, subjects `away` and `decided`, host tool `decided` (`ui-away`).

**Stock agents and charters**
- New charter 9, the status agent: no tools but finish, 4 turns, 5 minutes; stock rules delegate to it (`ui-home-status`).
- Kind 7, a project's ways: `drafts`, `goals`, `changes`, `away`, `way`; stock all off, kept as a code constant (`ui-settings`).
- Kind 6, a project's questions, built as a setting; off by default (`ui-settings`).
- `configuring.txt` and built-in docs teach kind 7; the configuring agent may propose it (`ui-settings`).
- Charter 1's goal part gains the report words and the `Saw:`/`Choice:` sentence; reference charter 5 gains the report (`ui-goals`, `ui-needs`).
- Stock tool mask gains note; a goal's scope narrowed to its own; observers hold none (`ui-goals`).
- Charter 3 gains note and recall; a chat has its own note scope; chat and configuring instructions ask for one document (`ui-chat-draft`).
- Briefs name the task and project (`ui-goals`, `ui-chat-draft`).
- Planned: charter 1 calls `decided` before asking (`ui-away`).

**Claude and box**
- `temper init` always writes the Anthropic provider; `agent.json`'s `path` may be `"fetched"` (`claude-always`).
- Launcher at `<data>/claude-code/claude`; Claude Code 2.1.311 pinned; `DISABLE_AUTOUPDATER=1` set there; no `documents.lines` written (`claude-always`).
- `fake-claude` now stands ahead of `demo` in the fake box's models (`claude-always`).
- Box terminal says `Forge N (Forgejo)`; the hands go to that step's own log (`ui-welcome`).

**skein**
- `State::FoldsAway` (`ui-chats`); `Ctrl+\`, a keyed slot, a menu item's description (`ui-chat-draft`).

## What the user asked of the rename form and the two scroll bars

Addition, two fixes the user asked for today (via fabrig/temper). Causes are a reader's, not reproduced: reproduce first, each with a test red first.
1. The project menu's rename form should match the mock (greenfield/app.js 229-240 and its Escape at 1136; mock.css 291-296): while renaming, the open project's row itself becomes the form: a name field filling the row (min-height 34px) and a primary small Save. "Rename NAME" leaves the menu's acts while the form is open. Enter saves, Escape cancels, an empty name keeps the old one, no visible Cancel. Today: temper-client-view/src/frame.rs rename_form, and temper.css 119-121 .temper-project-rename-form, the old look's rules; remove what this replaces.
2. A dark focus ring on main after a focused control disappears (the rail's divider looks like a black line). Repro: Home, project menu, Rename Hello, Cancel: skein-ui-web page.rs (~360-376) moves focus to Role::Main with tabindex=-1, and Chrome draws focus-visible; #app > main.temper-session has no rule. temper-client-shell-web lib.rs finish can do the same after keyboard use. Wanted: (a) closing the rename form by Save or Escape returns focus to the control that opened it, or the menu's summary, not main; (b) a programmatic focus target (main, its h1, anything given tabindex=-1 only to take focus) draws no outline, in skein's sheet if it belongs there. Real controls keep their focus ring.

Addition, a third fix the user asked for (via fabrig/temper): a chat shows two scroll bars, the browser's and the conversation pane's, after pressing "Older" twice in a long chat at a laptop's window size. A reader's cause, checked on the user's recorded page at 1280x720 and 1000x600 but not by this plan: the agent message's who-line (div.temper-who, "Temper 00:26") is hidden with position: absolute and no ancestor up to body is positioned, so a who-line below the pane's visible part isn't clipped by the pane's overflow: auto and extends the document (671px in a 633px window). Fix: #app > .temper-session [role=log] { position: relative; }, which brought the page back to 633px. Any other visually hidden absolute part inside a scrolling pane has the same fault: grep for them and fix each, including the draft pane. Test: a long chat's document never exceeds its window at 1280x720 and 1000x600 with Older pressed twice.
