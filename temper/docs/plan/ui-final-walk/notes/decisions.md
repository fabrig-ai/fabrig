# ui-final-walk: what is already decided

Carried from the decisions and rulings kept outside this repository, 2026-11-05. **A departure listed here is not a finding:** the report names it (`D4`, `Ruling 9`, or `By page, Home`) and moves on. Don't reverse one. Where a listed decision itself reads wrong in the walk, say so in the report's Notes; it stays.

**Stale words, by the cut's own check on the trunk:** this list was written before `ui-away` and `ui-phone` landed. Both are on main now, so read "not started" as "landed", and "until Away lands" or "until Away exists" as no longer holding: the sheet has "When it needs you" (Ruling 10), and a goal's How it runs follows the away switch or says "always asks you" from its `Asks: always` line. The rulings win over the list where they differ.

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

## Worth a look (`D1` to `D12`)

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

## By page

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

## A plan that overrode its own request, openly, and stands

Status agent: charter 9 not 7, kept not forgotten. "6+ / 9+" not needed. `project.json` not `settings.json`; the sign-out failure line stays. Forges and Accounts restyled with no project. Plain numbers, not `$S of $L`.
