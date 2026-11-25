# The gallery's project and settings states

The gallery's project and settings states are split from [the gallery](gallery.md).

## 1. Project pages and settings

| Name | The state |
|---|---|
| `changes` | a hand-made `ChangesPanel` delivery: three queued and four landed |
| `changes-empty` | a hand-made `ChangesPanel` delivery: nothing queued and nothing landed |
| `changes-one` | a hand-made `ChangesPanel` delivery: one queued and four landed |
| `changes-landed-only` | a hand-made `ChangesPanel` delivery, the mock's `hello`: nothing queued, something landed |
| `all-home` | the at-scale world's four projects, vsr, vsr-rs, cowsql and fabrig: three new briefs, two needs across two projects, four decisions, fabrig's main red, and What follows what, vsr-rs following vsr's repository two behind with its check passing and fabrig depending on vsr-rs; the whole story's `home.html`, and `phone-home.html` at a phone's width |
| `all-home-nothing-new` | the four projects with nothing landed since the last look, no brief new and no need: the kicker, "The last change landed SPAN ago.", "Nothing needs you." and Projects |
| `all-home-one-project` | one project read: no Projects, the headline "N changes landed in PROJECT." |
| `all-home-away` | `all-home` with Away on for vsr-rs and cowsql: the switch and the chip "Away on 2 of 4", not pressed (`../pages/README.md`, *Away*, *On All projects*) |
| `all-home-away-on` | `all-home-away` after a press, each answer Away: the switch "Away until 08:00", pressed, and the toast "Away turned on for 2 projects: agents decide within their limits." |
| `night-all-home` | `all-home` by night |
| `home-step-pair` | a project's Home under You approve: a brief, what landed, an approval's card leading with "Review on GitHub ↗", its size "+142 -18 in 4 files" under its question as `pair/home.html` draws it, and a question (`../../draft/surfaces.md`, 12, Pair) |
| `home-step-team` | a project's Home with Decided for you and a question (12, Team) |
| `home-step-stock` | a project's Home with a brief and Decided for you, no need (12, Stock) |
| `home-step-scale` | a project's Home with a question and a funds card (12, At scale) |
| `needs-list` | Needs you across two projects, three open: the first open, the others closed rows; Decided for you under them; the whole story's `needs.html`, and `phone-need.html` at a phone's width |
| `needs-list-second` | `needs-list` with its second row opened |
| `needs-tell` | `needs-list` with words written in "Or tell it something else" and Send shown |
| `needs-approval-first` | Needs you with an approval's card open, "Review on GitHub ↗" first, its size under its context as `pair/needs.html`'s second card draws it, "+142 -18 in 4 files" |
| `night-needs` | `needs-list` by night |
| `tell-area` | a project's Needs you, its `findings` document holding two areas, the page left a standing role's talk whose role holds the second: Tell temper at its foot, words written, "About PROJECT · the page you were on, ROLE", "Goes to ROLE, AREA's owner" and "File it" enabled (`../pages/needs-you.md`, 1, *Tell temper*; the whole story's `needs.html`, its foot) |
| `tell-page` | the same project's `Tell` page, the page left its Team, its triager a standing role: "Something off?", the lede, "About PROJECT · the page you were on, Team", "Goes to ROLE, PROJECT's triager, who hands it to its area's owner", "File it" and "It's a question: ask temper instead" (`phone-tell.html` at a phone's width) |
| `home-try-stopped` | a failed trial below Home's intro: "WHO didn’t answer when tried" and Try again; no change to its counts |
| `home-away` | hand-made Home deliveries: `AwayRow` on, four `Decision`s and live tasks; the mock's `home.html` in *Back from away* |
| `home-away-one` | hand-made Home deliveries: one `Decision`, the switch off: "One thing was decided for you." and the kicker's two times |
| `home-away-overturned` | hand-made Home deliveries: a row overturned, its task live, and one ended, "landed 03:12" |
| `home-away-quiet` | hand-made Home deliveries: `AwayRow` on, one card open and no `Decision` showing: the spell's kicker, "While you were away · 23:10 to now", its headline's landed count and the needs sentence, "Still needs you · 1", and no *Decided for you* (`../pages/home.md`, 1, *Decided for you*, *On Home*) |
| `home-decided` | hand-made Home deliveries: two `Decision`s of a chat and a goal, no Away row: the everyday kicker and headline, "Decided for you · 2" with each row's "Allowed:" line, and "Needs you" |
| `rail-away-off` | a project’s page after turning the switch off, with its toast open |
| `rail-away-on` | a project's page with the switch on, "Away until 08:00", and its toast open |
| `home-begin-both-left` | hand-made Home deliveries: both steps left, the repository first; first repository offered; the rail's Home, Ask temper, the hairline and How work runs alone, the phone's three tabs; the line "Two steps left. Skip them if you like; they stay in How work runs and Ask temper." |
| `home-begin-ask-left` | hand-made Home deliveries: repository adopted, "temper can change its code now."; "Tell temper how you work" left, its act staying in Ask temper (the look request `to-begin-two-steps`'s page) |
| `home-accounts-claude` | hand-made Home deliveries: Claude subscription alone; link offers ChatGPT or another plan |
| `home-accounts-both` | hand-made Home deliveries: ChatGPT plan and Claude subscription; link offers another account |
| `home-accounts-key` | hand-made Home deliveries: an API key; link offers another account |
| `home-with-goals` | `home-at-work`'s deliveries with a goal in the project, so that the rail shows Goals |
| `home-at-work` | hand-made Home deliveries: one question and three chats, the everyday request's words where the client holds its facts |
| `home-morning` | hand-made Home deliveries: `ProjectStatus` with everything: a brief above `seen`, a last look nine hours ago, three queued, needs and chats; the mock's morning |
| `home-computed` | hand-made Home deliveries: `ProjectStatus` with changes landed and no brief: the computed sentence, no lede or byline |
| `home-last-looked-none` | hand-made Home deliveries: `ProjectStatus` of a party's first look: no "last looked" and nothing new |
| `home-landing-none` | hand-made Home deliveries: `ProjectStatus` with nothing landed: "Landing: nothing has landed yet" |
| `home-landing-red` | hand-made Home deliveries: `ProjectStatus` with `red`: " · main red" in the signal's colour |
| `home-chats-sentence` | hand-made Home deliveries: no goal in the project: Chats' sentence after its rows |
| `home-needs-several` | hand-made Home deliveries: three needs, the first as a card and the others as rows |
| `home-nothing-needs` | hand-made Home deliveries: all steps done, no needs and one chat |
| `home-ready` | hand-made Home deliveries: both steps done; plain Home, its rail and tab bar whole |
| `home-adopting` | hand-made Home deliveries: first adoption in flight; no repository acts |
| `home-copying` | hand-made Home deliveries: acme/widget copying, acme/other waiting, in intent order |
| `home-copy-failed` | hand-made Home deliveries: first copy adopted, second failed with the forge’s words and Copy again |
| `home-copied` | hand-made Home deliveries: both copies adopted, with their source names and the Forgejo explanation |
| `home-copy-one` | hand-made Home deliveries: one copy adopted, with its singular explanation |
| `home-copy-offer` | hand-made Home deliveries: both addresses left unasked after a restart, with one filled Copy act |
| `home-add-repository` | hand-made Home deliveries: connected owned Forgejo with no first repository; Make, Copy and Adopt another |
| `home-add-repository-sheet` | those deliveries with Make an empty one pressed; the repository sheet open on Home |
| `home-add-repository-copies` | the same Home sheet opened at Copy a public one, with its addresses |
| `home-refused` | hand-made Home deliveries: first adoption refused; its words and retry acts |
| `home-no-repository` | hand-made Home deliveries: no repository or first offer |
| `home-after-skip` | hand-made Home deliveries: To begin skipped with steps left; plain Home with empty Needs you and Chats |
| `home-skip-refused` | hand-made Home deliveries: Skip refused; the alert under the note and Skip offered again |
| `home-begin-with-needs` | hand-made Home deliveries: To begin showing, with one question and one chat delivered and neither drawn: no Needs you and no Chats under the card |
| `home-begin-settings-left` | hand-made Home deliveries: a way of working accepted, the repository alone left; its act stays in How work runs; its Ask temper chat delivered and not drawn: no Chats under the card |
| `night-home` | `home-begin-ask-left` with the night switch pressed: the night look, the switch a sun (the look request `night-mode`'s page 01) |
| `night-ask` | `ask-first-run` with the night switch pressed: Ask temper as "Start" opens it on the first run's project, template 5 listed, at night (its page 02) |
| `night-chat-split` | `chat-draft-split` at night: a chat beside its draft (its page 03) |
| `goals-solo` | hand-made Goals deliveries: a small project's rows, one in each state, the mock's words where the client holds its facts |
| `goals-reference` | hand-made Goals deliveries: the reference's rows, a goal of a hundred tasks among them, and its ways in the lede |
| `goals-none` | hand-made Goals deliveries: "No goals yet."; the rail keeps Goals since the page is Goals |
| `goals-needs` | hand-made Goals deliveries: the filter Needs you, "needs funds" and "needs you" |
| `goals-under-way` | hand-made Goals deliveries: the filter Under way, running, planning and paused |
| `goals-ended` | hand-made Goals deliveries: the filter Ended, landed, closed and failed |
| `goals-filter-empty` | hand-made Goals deliveries: a filter no goal matches, "None." |
| `goals-more` | hand-made Goals deliveries: "More than N goals, M under way." and "More goals" |
| `home-github` | hand-made Home deliveries: GitHub connected with an accepted installation |
| `home-morning-card` | hand-made Home deliveries: three needs in the line form, the first as its card with its choices, the others as lines |
| `home-answered` | `home-morning-card` with the first answered: its line, its toast, the next as the card |
| `home-one-need` | one need: no lines and no "Answer in a row"; on a phone, "Answer" |
| `ask` | the solo world's Ask temper page before its first words: its heading, the form "Ask temper" and its empty text area |
| `ask-first-run` | the same page in the first run's world at its step 7, template 5 listed, To begin unfinished, its way of working left: the rail's Home, Ask temper, the hairline and How work runs alone, the phone's three tabs; the lede "How a change is produced and lands in PROJECT." and the ways, Agents review run and shown: the head, its line, and at the lede's right "Keep it" and the drop-down "Agents review"; its path, the foot's legend and "How does temper work?" alone; the last line; Send disabled while the words are empty |
| `ask-ways-menu` | `ask-first-run` with the drop-down open, its menu "Stock ways": "You approve", "Agents review" ticked with "current" at its right, "Draft first" |
| `ask-ways-approve` | `ask-first-run` with "You approve" chosen from the drop-down: its line, its path with "you approve" and no review, "Use this" and the drop-down "You approve" at the lede's right, no "Back" |
| `ask-ways-approve-menu` | `ask-ways-approve` with the drop-down open: "You approve" ticked, "Agents review" with "current" at its right, "Draft first" |
| `ask-ways-draft` | hand-made `ways` delivery over a repository with CI and tiers on goal branches and on main, "Draft first" chosen: its two landings, the goal branch's bar dimmer, "verify" before main |
| `ask-ways-card` | `ask-first-run` with the review's stop pressed: its card under the path, its caret at the stop, its model, what it can do and what it is asked to |
| `ask-ways-card-person` | `ask-ways-approve` with "you approve" pressed: a person's card, the role and what they decide |
| `ask-ways-yours` | hand-made `ways` delivery whose project runs no stock way, with the project's words: their `lede` as the head's line, the drop-down "PROJECT's own" alone at the lede's right, its path, "How does temper work?" alone in the foot |
| `ask-ways-yours-bare` | `ask-ways-yours` with no project words: the line "PROJECT runs settings of its own." |
| `ask-ways-yours-menu` | `ask-ways-yours` with the drop-down open: "PROJECT's own" ticked with "current" at its right, then the three ways |
| `ask-ways-refused` | `ask-ways-approve` with "Use this" refused for too few approvers: the forge, the role, how many it asks and how many can, under the foot |
| `ask-ways-member` | a member's page: the line and the path, the drop-down alone at the lede's right, "How does temper work?" alone in the foot |
| `ask-ways-kept` | after "Keep it": its toast naming the revision, Agents review shown with the drop-down "Agents review" alone at the lede's right, To begin's first step done |
| `ask-typing` | words entered; Send enabled; the rail and tabs as `ask-first-run`'s |
| `ask-sending` | the Ask temper start out; Send and "How does temper work?" disabled, words kept; the rail and tabs as `ask-first-run`'s |
| `ask-refused` | the start refused; jig's words below the composer, the words kept; the rail and tabs as `ask-first-run`'s |
| `ask-chat` | the solo world’s T10 as an Ask temper chat, conversation and proposal in its layout, its path through Ask temper |
| `ask-chat-proposal` | `ask-chat` with a pending proposal of settings: its title, lines, items and "Accept" |
| `ask-chat-proposal-deployment` | that proposal of the deployment's settings: "How temper will run your projects" and "Accept" |
| `ask-chat-proposal-two-projects` | that proposal of two projects' settings: "How temper will run tally and 2" and "Accept" |
| `ask-chat-accepted` | that proposal accepted: "✓ Accepted · new work follows it" |
| `ask-chat-rejected` | that proposal rejected: "Rejected · nothing changes", its items kept, no "Accept" and no "Change something" |
| `ask-chat-replaced` | that proposal dimmed, "Replaced by the proposal below", the later one under it |
| `ask-chat-withdrawn` | that proposal withdrawn as stale: "Withdrawn · nothing changes", its items kept, no "Accept" and no "Change something" |
| `settings` | hand-made settings deliveries: How work runs with every part of kind 7 shown |
| `settings-all-hidden` | hand-made settings deliveries: every switch of kind 7 off, the mock's `settings.html?project=hello` |
| `settings-goals` | hand-made settings deliveries: Goals shown, its ways of working with one the default |
| `settings-chat-open` | hand-made settings deliveries: an Ask temper chat open; How work runs offers no "Start" and no "Go on", and the rail's "Ask temper" opens that chat |
| `settings-drafts-off` | `settings` with "Chats can write drafts" off |
| `settings-goals-off` | `settings` with Goals hidden: no ways of working, and the rail without "Goals" |
| `settings-changes-off` | `settings` with Changes hidden: the rail without "Changes" |
| `settings-refused` | a switch pressed and refused, jig's words under its row, the switch back at its value |
| `how` | hand-made `ways` delivery over a repository with CI and tiers, Agents review current and shown: the heading "Agents review", its line, "Ask temper to change this", its path drawn with "verify" before main's bar and the legend under it, no act but "Keep it" while To begin's first step is left; What reaches you from Agents review's stock classes, "While you’re away, they are decided with the recommended choice and listed."; the foot's switches (`../pages/settings.md`, 2, *The page*) |
| `how-approve` | hand-made `ways` delivery over a repository with neither CI nor tiers, You approve current: its path with no "check" and no tier, "you approve" above the line; What reaches you from You approve's classes: no "Agents decide, then tell you", every movable kind under "Asks you", "They ask even while you’re away.", and no "Away until" |
| `how-draft` | `how` with Draft first current, Goals shown and its template giving 80: two landings, the goal branch's bar, Roles from a hand-made `roles` answer as `whole-story/how.html` draws them: the chat, Ask temper, the drafter, the coordinator, the coding agent and the status agent, each `who` saying when it runs and what it does, Budgets' row "Goals", "80" |
| `how-yours` | hand-made `ways` delivery whose project runs no stock way: "A way of its own", the way "its own", the head's line from the project's words, the drop-down "PROJECT's own" alone at the section head's right, no act |
| `how-menu` | `how` with the drop-down open: the three ways, Agents review ticked with "current" at its right |
| `how-card` | `how` with the review's stop pressed: its ring and words, its card under the drawing with its caret, its model, what it can do and what it is asked to, "Open the reviewer ›" |
| `night-how` | `how` with the night switch pressed: the path's marks by night |
| `how-reaches` | hand-made deliveries, Draft first current, its classes the whole story's `how.html`'s: wording, retries, implementation choices and test-only changes decided, then a path set, "vsr’s protocol core", asking even when away, and a new dependency and a departure Away may decide: What reaches you's three columns, "While you’re away, all but changes to vsr’s protocol core are decided with the recommended choice and listed.", and its rows, the question route and "Away until" |
| `night-how-reaches` | `how-reaches` with the night switch pressed: the columns' chips and marks by night |
| `how-rules` | `how-reaches` with its path set named "protocol core" and a rule by path naming it, as the whole story's `how.html` draws vsr's: no stock way current, "Yours" shown, the project's words' heading "Draft first, with the protocol core asking you"; on its path, on main's segment after its check and repair, the stop "you" with "if protocol core" over it; Rules by path with "+ A rule", no "Review" column, a row of the set's patterns signed off by "you", "once you sign off", and the note |
| `how-rules-card` | `how-rules` with the sign-off's stop pressed: its card, "Only for" the set's patterns, "Role" "owner", "Asks" "1 of them" and "Counts" |
| `how-team` | hand-made deliveries as the whole story's `team/how.html` draws shop's, three owners, the reader one of them: "Yours" shown, its path with "review" and "one of you"; Rules by path with "Review", a row "payments/", "an agent", "two of you", "once two have signed this head", then "everything else", "an agent", "one of you", "once signed", and the note |
| `night-how-team` | `how-team` with the night switch pressed: the table's chips by night |
| `how-follows` | `how-draft` with a hand-made `follows` document, project document and standing roles: What follows a landing's box "cowsql/vsr-rs follows cowsql/vsr", "Each landing on cowsql/vsr's main that touches src or include makes a change in cowsql/vsr-rs's main, owned by Pat.", "Each waits for Pat's word.", its check's workflow, "2 behind · check passes at" its source commit's twelve digits, its chips "code hears the landing", "an agent ports" and "code checks"; the box "Tiers on main" from the path's tier, "code runs" and "an agent repairs"; Budgets' head "a week · renews Tuesday", a standing role's row, its title, "a standing role · 28 left" and "40", then "Goals", "80" (`../pages/settings.md`, 2) |
| `night-how-follows` | `how-follows` with the night switch pressed: the chips' marks, code's square among them, by night |
| `repositories` | the Repositories tab with rows |
| `repositories-none` | the Repositories tab's quiet band |
| `forges-in-settings` | `Forges` inside Deployment's frame |
| `accounts-in-settings` | `Accounts` inside Deployment's frame, "Sign out" at its foot at a phone's width alone |
| `forges-empty` | the Forges tab's quiet band, in a world with no forge |
| `forges-setting-up` | the first run's Forges at step 6: a row setting up |
| `forges-fetching` | the first run's Forges at step 24: a row fetching Forgejo, its percent in its summary |
| `forges-fetching-bytes` | the first run's Forges at step 25: a row fetching Forgejo, its bytes in its summary |
| `forges-failed` | the first run's Forges at step 9: a set up failed, with "Set up again" |
| `forges-replacing` | the solo world's Forges, a row's Replace open on its private key |
| `forges-removing` | the first run's Forges at step 7: a row's removal asking first |
| `models` | Settings, Models: offered models by provider, what pays, default efforts, picker switches and the last line's link to Ask temper |
| `models-add` | an owner adding a model by provider and name |
| `models-off` | a model switched off, still in the list |
| `models-added` | a person's model: "added by you" and "estimated prices" |
| `models-member` | a member reads On and Off, with no switch or add form |
| `models-one-account` | Claude alone: OpenAI has its Connect line and no stock rows |
| `accounts-empty` | the Accounts tab's quiet band, in a world with no account |
| `accounts-states` | the reference world's accounts, a row in each state of a source |
| `accounts-key` | the reference world's accounts, a row's key field open |
| `accounts-removing` | the reference world's accounts, an account's removal asking first |
| `accounts-signing-in` | the reference world's accounts, a row's sign-in under way with its field for the page's address |
| `accounts-try-stopped` | a failed trial under an account's head and before Sources, its why and Try again; Sign in again goes to Connect a model |
| `accounts-program-needs-sign-in` | the solo world's accounts: a program's login not signed in on its host, with "Check again" |
| `accounts-shared` | two accounts of one person, each paying for 11 people: "Ready · Pays for 11 people" folded; opened, "Pays for Ana, Ben, … and Kai.", one person "Person N" with no read naming them (`../pages/accounts-words.md`, 1, *An account's line*) |
| `accounts-usage` | a Claude subscription with its week's window alone: "Week", 29%, "resets in 4 d 14 h · 2.0%/h · full in 1 d 11 h" (`../pages/accounts-words.md`, 1, *Usage*) |
| `accounts-usage-filling` | a Claude subscription whose 5-hour window would fill before its reset, "full in" amber, above its week at 83%, its bar amber |
| `accounts-usage-calm` | a week lasting to its reset, "about 59% at reset", and a 5-hour window that won't fill, not drawn |
| `accounts-usage-limited` | a limited window: its bar red and whole, "resets in" alone |
| `accounts-usage-model-week` | `accounts-usage-filling`'s windows and two models' weeks: Fable's, 64%, "resets in 2 d · 1.6%/h · full in 22 h 30 min", drawn under "Week" as "Fable"; Sonnet's, 0% with no rate, not drawn (the look request `usage-model-week`) |
| `builders` | the Builders tab, its hosts connected, draining and not connected |
| `builders-not-owner` | the Builders tab for a person with no role in the deployment |
| `builders-none` | the Builders tab with no host connected |
| `ask-chats-tag` | the solo world’s chats with T10 tagged Ask temper beside ordinary chats |
| `chats-solo` | the solo world's chats |
| `chats-live` | `chats-solo` under "Live" |
| `chats-closed` | `chats-solo` under "Closed" |
| `chats-no-repository` | the Chats page of a project with no repository: its band |
| `chat-new` | `NewChat` with a repository: its `.meta-line` and focused composer |
| `chat-new-no-repository` | `NewChat` with no repository |
| `chat-new-first-run-1` | `NewChat` at the first run's first screen: the playground's start |
| `chat-work-done` | the `chat-work` request's conversation, calls settled and folded, answer committed |
| `chat-work-open` | the same conversation, the call run open to all thirteen rows |
| `chat-work-running` | its first four calls settled and template 2 being checked, with no supplied outcome |
| `chat-work-writing` | the same calls settled, the supplied opening paragraph being written with its end mark |
| `chat-work-failed` | a failed charter read, its line and outcome in the signal's colour |
| `chat-work-one` | one `read_docs` call alone, "Read the page hold" |
| `chat-work-unknown` | a tool outside the words table, "read_file ×2" |
| `chat-work-separated` | text between two calls ends one run and starts another |
| `chat-work-empty` | before the first word or call, the three dots |
| `chat-work-reading` | earlier messages loading and Latest over the foot while away from the end |
| `chat-working` | a live chat at work: its three dots and "✦ Working" above the composer |
| `agent-work` | an agent's page, its last run running with a call under way, and Latest over the foot while away from the end |
| `chat-change-running` | a chat with a delegated change running: its one line |
| `chat-change-landed` | a chat with a delegated change landed: its one line with its result's words |
| `chat-change-held` | the first run's chat T1 at step 2, its change held: its one line says so |
| `chat-goal` | a chat with a goal it started: its one line |
| `chat-model-open` | a new chat, Claude alone, its picker open: the other provider's line, the efforts |
| `chat-model-both` | a new chat with Claude and a ChatGPT plan, its picker open |
| `chat-model-all` | a new chat, its picker on "All models", with a model `picker` doesn't show |
| `chat-model-two-accounts` | a new chat with two accounts of one provider, its picker open: a heading for each |
| `chat-model-sign-in-again` | a new chat whose account has stopped being usable, its picker open: the provider's own fix and no model |
| `chat-model-fixed` | a chat whose choice can't be changed: the same words, not a button |
| `chat-started-model-open` | a started chat, its picker open: "A change applies from the next turn." |
| `chat-started-model-changing` | a started chat with a change sent, its picker open: "Changing…" |
| `chat-started-model-refused` | a started chat with a change refused, its picker open: why |
| `ask-model-open` | Ask temper's page, its picker open |
| `chat-draft-chat` | a chat with a document, in Chat: its group and tool line |
| `chat-draft-split` | the same chat in Split: the strip, the pane and a tinted section |
| `chat-draft-draft` | the same chat in Draft: the composer at the foot on white |
| `chat-draft-empty` | Split chosen, nothing written: the empty pane and Write it up from this chat |
| `chat-new-draft` | `NewChat { draft: true }`: the empty pane in Split |
| `chat-start-goal` | a chat's sheet to start a goal, open |
| `chat-terminal` | the `chat-draft` chat with `terminal::Act::Enter` applied, not through a page control: the terminal, dark, DejaVu Sans Mono (the request's page 02) |
| `chat-terminal-draft` | the same with the draft beside it (page 03) |
| `chat-terminal-commands` | the same with "/" in the box: the commands' list (page 04) |
| `chat-terminal-model` | the same with `/model`'s list (page 05) |
| `chat-terminal-working` | a turn running: the running line above the box (page 06) |
| `chat-terminal-light` | the terminal in its light theme and IBM Plex Mono, its draft beside it |
| `chat-terminal-closed` | a closed chat in the terminal: the box for commands, "/new starts another chat" |
| `chat-terminal-question` | a chat whose goal asked the person: the question's list under the box |
| `chat-terminal-proposal` | an Ask temper chat's proposal in the terminal: its one line and its acts |
| `chat-start-goal-refused` | the same sheet, refused in jig's words |
| `chat-goal-linked` | a chat with its goal started: "Goal TITLE →", and its line |
| `chats-drafts` | the Chats page under Drafts: "writes away-mode.md" |
| `ask-chat-document` | an Ask temper chat in Chat, its document written |
| `ask-chat-document-split` | the same chat in Split, its pane's line and "Start over" |
| `ask-chat-draft-empty` | an Ask temper chat in Split with no document: "TN · no draft yet", Write it up from this chat, and Start over in the bar |
| `rail-new-menu` | the rail's "+ New draft or change" with its menu open: Draft, Change and Ask temper |
| `rail-person-menu` | the rail's person with its menu open |
| `rail-new-change` | a project offering no drafts, as at Pair: the rail's "+ New change", a link, no menu |
| `rail-all-projects` | a person in two projects, the switcher open: "All projects", "2 projects", first; on a wide page its acts end at "Rename PROJECT" and "New project"; at 760 px and under "Team for PROJECT" where Team shows, "How work runs for PROJECT" and "Deployment" follow, in that order (`../view-navigation.md`, 1) |
| `rail-across` | *Needs you* across projects, a need in each of two projects: the switcher names "All projects"; the rail's "Needs you 2", current, counting both, its Home to `AllHome`, and the phone's Home and "Needs 2" the same; its Work and How work runs the project last open (`../view-navigation.md`, 1, *All projects*) |
| `work` | hand-made deliveries after the whole story's `work.html`: two live goals, one with a change checking and one landed on its card; a change of its own, queued; a chat with its change; two drafts, as `work.html`'s, a design session's being written and the witness catch-up, landed and ready, with "Start a goal: WAY", WAY its project's way as `../pages/work.md`, 1, *Ways*, names it, and "Another way"; four changes landed this week |
| `work-pair` | hand-made deliveries after `pair/work.html`: chats and their changes alone, drafts not offered; no Drafts or Goals filter, the rail's "+ New change" |
| `work-drafts` | `work` under Drafts: the drafts' cards alone, the landed draft's among them |
| `work-goals` | `work` under Goals: Work's head and filters, the goals' list under them, its heading at level 2 |
| `work-changes` | `work` under Changes: Work's head and filters, the queue and what landed under them |
| `work-empty` | hand-made deliveries with nothing listed: "Nothing in flight yet.", All alone among the filters |
| `members` | Deployment's Members: the deployment's line, two projects' members with their roles |
| `members-refused` | Members with one project's read refused: its title and "Read by the project’s members." |
| `limits` | Deployment's Limits: the capacities, two waiting attempts, and two projects' tasks |
| `limits-not-owner` | Limits for a person with no role in the deployment: its one line |
| `night-work` | `work` with the night look on |
| `night-limits` | `limits` with the night look on |
| `remembered-none` | "What temper remembers about you" with none kept: its quiet band (`../pages/remembered.md`) |
| `remembered-three` | three lines, two kept by an Ask temper chat and one changed by the person, each with "Change" and "Delete" |
| `remembered-full` | twelve lines, and the full list's line under the last |
| `remembered-changing` | `remembered-three` with a line's field open, "Save" and "Cancel" |
| `remembered-refused` | `remembered-three` after a change refused as moved: its line under the row |
| `night-remembered-none` | `remembered-none` with the night look on |
| `night-remembered-three` | `remembered-three` with the night look on |
| `night-remembered-full` | `remembered-full` with the night look on |
| `night-remembered-changing` | `remembered-changing` with the night look on |
| `night-remembered-refused` | `remembered-refused` with the night look on |
| `told-not-set-up` | "Telling you" on a deployment that doesn't push: its quiet band (`../pages/told.md`) |
| `told-off` | "Telling you" with this browser off, "Turn on", and two devices under "Your devices", each with "Remove" |
| `told-on` | `told-off` with this browser on: "On since …", "Turn off", and its row marked "This browser" |
| `told-cannot` | "Telling you" on an iPhone's browser that can't be pushed to: its words and the Home Screen's line, no act |
| `told-refused` | `told-off` after "Turn on" refused as `too_many`: its line under the row |
| `night-told-not-set-up` | `told-not-set-up` with the night look on |
| `night-told-off` | `told-off` with the night look on |
| `night-told-on` | `told-on` with the night look on |
| `night-told-cannot` | `told-cannot` with the night look on |
| `night-told-refused` | `told-refused` with the night look on |

## 2. Open questions

None.
