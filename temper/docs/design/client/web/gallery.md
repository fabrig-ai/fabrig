# The gallery

`temper-client-gallery` (`bin/temper-client-gallery`, calling
`crates/temper-client-gallery-tool`), a program of temper's workspace, prints the browser's pages in named states into a
directory, as static documents, for screenshots beside the mockups
and a person's eye. It is a tool, not a test of the pages: what
the pages do is the worlds' (`testing.md`), and the gallery draws what the
view draws, never changing a view to suit a sample. It stays for the
states no journey reaches (`journeys.md`): a journey's print, replayed
from the engine's own answers, is where the look is tuned.

## 1. What it prints

- **One command:** `cargo run -p temper-client-gallery -- OUT`. It
  reads no configuration, environment, clock or network, and starts no
  server.
  `OUT` is replaced whole when it holds the gallery's `index.html`,
  made when it is missing, and refused otherwise.
- **Each state's page** as `OUT/NAME.html`: the view's tree for the
  state, made a page by skein's `Page` with the shell's look (`Look`
  with `look::NAMES`, primary input `Control`) and its prefix, its
  links given the hrefs the shell gives them (`README.md`, 2.3), by the view's one
  href function, which the shell's host calls too. The page's root is
  the document's `#app`, as skein's `Dom` mounts it; dialogs carry
  `open`, since a static document has no top layer. The title is the
  shell's: the page's heading, then "temper". A state whose root's kept
  look is night, its switch pressed, prints `data-night` on its `html`, as
  the shell sets it (`look.md`, *Day and night*).
- **The bundle's stylesheets and files,** under `OUT/bundle/`, made by
  skein's bundler from the stylesheets and files `scripts/bundle.sh`
  passes, in its order (`look.md`), with no module: each page links them as
  the bundle's page does, so `temper.css`'s files resolve as they do
  in the bundle. The bundle's own page and its empty module are unused.
- **An index,** `OUT/index.html`: each state's name, in the gallery's
  order, linking its page, with its mockup's address beside it where a
  mockup draws it.
- **Static:** no script runs. The opening focus is an `autofocus`
  attribute, and a named sample may hold one disclosure open; caret,
  scroll-following and live updates are not replayed.
- **A path printed for both widths:** a static page measures no box, so
  each path's drawing (`../pages/chat/conversation.md`, 3) is printed twice in its figure:
  across, drawn in a box 720 wide, and down the page, drawn in a box 388
  wide, the box at 420 pixels with a phone's 16 pixels each side
  (`../pages/README.md`). The page's own style, in its head, shows the second
  at 760 pixels and under, the shell's phone width, and the first above
  it, so that the print at 420 runs the path down the page as the shell
  does.

## 2. Its states

- **A state** is a world at a page's address, then what is done there:
  the client's root stepped from start by the documents a world
  delivers, then by the person's acts or the engine's answers that the
  state names (a close pressed, a reject's reason box, a refusal), as a
  page would be. No page state is set by hand.
- **The worlds are the mockups':** reference, solo and the first run at
  its steps, their words copied from `mockups/world-*.js` where temper's
  model holds them, so that a picture differs from its mockup by look,
  not content. A value the model has no place for is left out, never
  invented. The gallery's own documents carry what the domain's
  fixtures don't: the providers, accounts and their holders by name.
  A spend is the contract's whole number (jig's owner numbers), so a
  mockup's fraction is rounded to the nearest (0.8 is 1) until the
  contract holds one. A project lists the templates its box would offer:
  the first run's worlds the stock's chat (1), goal (2) and Ask temper
  (5, charter 11), as a fresh box does, so that their pages draw Ask
  temper's rail row and tab (the look request `ask-temper-place`); the
  reference world Ask temper at 5 too, and its own ways at numbers the
  stock leaves free; the solo world Ask temper in its Ask temper states
  alone.
- **Its time** is each world's, fixed: the mockup's now, its
  `world.now`, at which the view builds, and each record's time the
  mockup's offset from it, so that a time reads as its mockup's
  ("12 min ago"). The program still reads no clock (1).
- **Its bounds** are the browser's shipped client limits as they are
  (`temper-client-shell-terminal`'s `native()`, which the browser's
  shell takes), never widened, and the view's bounds are derived from
  them as the shell derives its own: a page shows what a person's
  browser would, a window's rows and a chat's lines included (`../view.md`,
  4), so that its outline is the live page's. A world holds more than a window shows; the page shows the
  window, and the state's test row says what it shows.
- **Names,** binding: a state a mockup draws is named by
  its address, `<page>-<world>[-<step>][-<task>]` (`chat-solo-T10` for
  `chat.html?world=solo&task=T10`, `chats-first-run-1` for
  `chats.html?world=first-run&step=1`); one no mockup draws takes a
  plain name (`signed-out`, `chats-empty`, `adoption-refused`). A name
  is never reused for another state.

| Name | The state |
|---|---|
| `changes` | a hand-made `ChangesPanel` delivery: three queued and four landed |
| `changes-empty` | a hand-made `ChangesPanel` delivery: nothing queued and nothing landed |
| `changes-one` | a hand-made `ChangesPanel` delivery: one queued and four landed |
| `changes-landed-only` | a hand-made `ChangesPanel` delivery, the mock's `hello`: nothing queued, something landed |
| `all-home` | the at-scale world's four projects, vsr, vsr-rs, cowsql and fabrig: three new briefs, two needs across two projects, four decisions, fabrig's main red; the whole story's `home.html`, and `phone-home.html` at a phone's width |
| `all-home-nothing-new` | the four projects with nothing landed since the last look, no brief new and no need: the kicker, "The last change landed SPAN ago.", "Nothing needs you." and Projects |
| `all-home-one-project` | one project read: no Projects, the headline "N changes landed in PROJECT." |
| `all-home-away` | `all-home` with Away on for vsr-rs and cowsql: the switch and the chip "Away on 2 of 4", not pressed (`../pages/README.md`, *Away*, *On All projects*) |
| `all-home-away-on` | `all-home-away` after a press, each answer Away: the switch "Away until 08:00", pressed, and the toast "Away turned on for 2 projects: agents decide within their limits." |
| `night-all-home` | `all-home` by night |
| `home-step-pair` | a project's Home under You approve: a brief, what landed, an approval's card leading with "Review on GitHub ↗" and a question (`../../draft/surfaces.md`, 12, Pair) |
| `home-step-team` | a project's Home with Decided for you and a question (12, Team) |
| `home-step-stock` | a project's Home with a brief and Decided for you, no need (12, Stock) |
| `home-step-scale` | a project's Home with a question and a funds card (12, At scale) |
| `needs-list` | Needs you across two projects, three open: the first open, the others closed rows; Decided for you under them; the whole story's `needs.html`, and `phone-need.html` at a phone's width |
| `needs-list-second` | `needs-list` with its second row opened |
| `needs-tell` | `needs-list` with words written in "Or tell it something else" and Send shown |
| `needs-approval-first` | Needs you with an approval's card open, "Review on GitHub ↗" first |
| `night-needs` | `needs-list` by night |
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
| `ask-first-run` | the same page in the first run's world at its step 7, template 5 listed, To begin unfinished, its way of working left: the rail's Home, Ask temper, the hairline and How work runs alone, the phone's three tabs; the lede and the ways, Agents review current and shown with "Keep it", Send disabled while the words are empty |
| `ask-ways-approve` | `ask-first-run` with "You approve" pressed: its line, its path with "you approve" and no review, and "Use this" |
| `ask-ways-draft` | hand-made `ways` delivery over a repository with CI and tiers on goal branches and on main, "Draft first" shown: its two landings, the goal branch's bar dimmer, "verify" before main |
| `ask-ways-card` | `ask-first-run` with the review's stop pressed: its card under the path, its caret at the stop, its model, what it can do and what it is asked to |
| `ask-ways-card-person` | `ask-ways-approve` with "you approve" pressed: a person's card, the role and what they decide |
| `ask-ways-yours` | hand-made `ways` delivery whose project runs no stock way: a fourth tab, "Yours", current and shown, its path, no act |
| `ask-ways-refused` | `ask-ways-approve` with "Use this" refused for too few approvers: the forge, the role, how many it asks and how many can, under the foot |
| `ask-ways-member` | a member's page: the tabs and the paths, no act |
| `ask-ways-kept` | after "Keep it": its toast naming the revision, Agents review current with no act, To begin's first step done |
| `ask-typing` | words entered; Send enabled; the rail and tabs as `ask-first-run`'s |
| `ask-sending` | the Ask temper start out; Send disabled, words kept; the rail and tabs as `ask-first-run`'s |
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
| `how-draft` | `how` with Draft first current, Goals shown and its template giving 80: two landings, the goal branch's bar, Roles from a hand-made `roles` answer as `whole-story/how.html` draws them: the chat, Ask temper, the drafter, the coordinator, the coding agent, the foreman and the status agent, each `who` saying when it runs and what it does, Budgets' row "Goals", "80" |
| `how-yours` | hand-made `ways` delivery whose project runs no stock way: "A way of its own", the way "its own", a fourth tab, "Yours", current and shown, no act |
| `how-card` | `how` with the review's stop pressed: its ring and words, its card under the drawing with its caret, its model, what it can do and what it is asked to, "Open the reviewer ›" |
| `night-how` | `how` with the night switch pressed: the path's marks by night |
| `how-reaches` | hand-made deliveries, Draft first current, its classes the whole story's `how.html`'s: wording, retries, implementation choices and test-only changes decided, then a path set, "vsr’s protocol core", asking even when away, and a new dependency and a departure Away may decide: What reaches you's three columns, "While you’re away, all but changes to vsr’s protocol core are decided with the recommended choice and listed.", and its rows, the question route and "Away until" |
| `night-how-reaches` | `how-reaches` with the night switch pressed: the columns' chips and marks by night |
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
| `chat-working` | a live chat at work: its three dots |
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
| `chat-draft-empty` | "+ Draft" pressed and nothing written: the empty pane |
| `chat-new-draft` | `NewChat { draft: true }`: the empty pane in Split |
| `chat-start-goal` | a chat's sheet to start a goal, open |
| `chat-terminal` | the `chat-draft` chat with ">_" pressed: the terminal, dark, DejaVu Sans Mono (the request's page 02) |
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
| `rail-new-menu` | the rail's "+ New draft or change" with its menu open: Draft, Change and Ask temper |
| `rail-person-menu` | the rail's person with its menu open |
| `rail-new-change` | a project offering no drafts, as at Pair: the rail's "+ New change", a link, no menu |
| `rail-all-projects` | a person in two projects, the switcher open: "All projects", "2 projects", first |
| `rail-across` | *Needs you* across projects, a need in each of two projects: the switcher names "All projects"; the rail's "Needs you 2", current, counting both, its Home to `AllHome`, and the phone's Home and "Needs 2" the same; its Work and How work runs the project last open (`../view.md`, 3, *All projects*) |
| `work` | hand-made deliveries after the whole story's `work.html`: two live goals, one with a change checking and one landed on its card; a change of its own, queued; a chat with its change; two drafts; four changes landed this week |
| `work-pair` | hand-made deliveries after `pair/work.html`: chats and their changes alone, drafts not offered; no Drafts or Goals filter, the rail's "+ New change" |
| `work-drafts` | `work` under Drafts: the drafts' cards alone |
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
| `chat-solo-T10` | the solo world's chat T10, its proposal in place |
| `inbox-solo` | the solo world's *Needs you* (`../pages/README.md`): the first of a proposal and a question |
| `needs-one` | *Needs you*: a question in the line form, the first of three |
| `needs-saw-open` | `needs-one` with "What the agent saw" open |
| `needs-reply-open` | `needs-one` with "Reply…" pressed: the reply form |
| `needs-second-answered` | the first answered, its line and toast showing: the second of three |
| `needs-later` | the first marked later: the next shown, the toast's words |
| `needs-last` | the last card: "The last one" |
| `needs-done` | every card answered: "Nothing needs you." and the answered lines |
| `needs-proposal` | a proposal's card |
| `needs-held` | a held task's card |
| `needs-funds` | a held task out of budget: "S of L used." |
| `needs-plain-words` | a question in no line form: "Reply…" alone |
| `needs-decided` | the unfiltered *Needs you*: a question, then the region "Decided for you · 2", "for 24 hours, then in each task's history" at its head's right, with rows of two projects, each "PROJECT · TITLE · 01:05" |
| `needs-decided-project` | *Needs you* filtered by one project: a question, then the region "Decided for you · N" with "Limits" at its head's right, each row "TITLE · 01:05" (`../pages/home.md`, 1, *Decided for you*) |
| `needs-why` | a question whose asked row says why: "Reached you because it is an implementation choice, which asks you unless you’re away." then "Keep asking" and "Stop asking: let agents decide implementation choices" on the line |
| `needs-stop-asking` | `needs-why` with its rule open for an owner: "Let agents decide implementation choices, then tell you." with "Apply"; "Keep asking" on the line, none under the offers |
| `needs-stop-asking-set` | a question that changes a path set, its rule "Stop asking: let agents decide some changes to protocol core" open for a member: the words offer's sentence over its field, the split, each with "Propose" |
| `needs-only-you` | the unfiltered *Needs you* (`../pages/README.md`, **Only you**): a question, then "Only you · 2", a signed-out ChatGPT sign-in and an expiring Claude Code login, each with "Sign in again" |
| `needs-only-you-done` | no card open: "Nothing needs you.", "Home", then "Only you · 1", a signed-out ChatGPT sign-in |
| `night-needs-only-you` | `needs-only-you` with the night look on |
| `accounts-expiring` | Accounts (`../pages/accounts.md`, 2, *Each state*): a ChatGPT sign-in read as expiring, "Expiring: sign in again before it ends." and "Continue with ChatGPT" |
| `goal-running` | first-run-projects, running: six of nine tasks in, its report, Left and Risk, the bar, three waiting lines, "6 landed" folded |
| `goal-needs-funds` | first-run-projects held for funds: "needs funds", its band with Release and Leave held |
| `goal-needs-question` | welcome-stall with an open question: its band and its parsed choices, the recommended one primary |
| `goal-planning` | forge-cards, no report and no tasks: the planning words, the dots and "Proposing tasks from the draft…" |
| `goal-paused` | first-run-projects paused by the person: "paused by you", Resume |
| `goal-landed` | pane-chat-line landed: its result, "Landed at 01:12.", no acts and no composer |
| `goal-landed-open` | `goal-landed` with "N landed" pressed: its rows under it |
| `goal-closed` | welcome-stall closed: "Closed by you…", "Closed before it landed." |
| `goal-failed` | forge-cards failed: "Failed: " and its reason, "Failed before it landed." |
| `goal-no-report` | first-run-projects running with tasks and no report: "N of M tasks done.", no byline |
| `goal-amending` | `goal-running` with Amend pressed: the editor, between the header and the report |
| `goal-closing` | `goal-running` with Close pressed: its confirmation |
| `goal-toast` | `goal-running` paused, its toast showing |
| `goal-sending` | `goal-running` with the person's words sent and one being sent, the composer holding its next words |
| `start-goal` | Goals with "Start one from a draft" pressed: the sheet with no draft, its Draft field and two ways |
| `start-goal-draft` | the sheet opened with ../../draft/first-run-projects.md, its name, its default way and "When it needs you", as the project says |
| `start-goal-refused` | `start-goal-draft` after a refused start: "Always ask" chosen, `.sheet-error` in jig's words |
| `task-solo-T3` | the solo world's goal T3, on `goal.html`'s page |
| `accounts-solo` | the solo world's accounts |
| `chats-first-run-1` | the first run's Chats at step 1: an empty list |
| `welcome-first-run-0` | the first run's welcome: its two cards |
| `welcome-go-on` | Welcome asked for with a real forge under way: Go on setting up, no playground |
| `connect-forge-first-run-6` | the first run's `ConnectForge` at step 6: Forgejo here setting up, the box preparing |
| `forges-first-run-7` | the first run's Forges at step 7: Forgejo here connected |
| `welcome-forge-choose` | the welcome's Forge choices, before any forge is connected |
| `welcome-forge-details` | the welcome's Forgejo Details, with organization and repository paired |
| `welcome-sheet-copies` | Forgejo's sheet with public copy addresses, in the order written |
| `welcome-sheet-copies-refused` | Forgejo's sheet refuses an invalid copy address above its foot |
| `welcome-sheet-nothing` | Forgejo's sheet with Nothing for now selected |
| `welcome-forge-under-way` | the Forge step's status in the column, Each step, and Continue while Forgejo sets up |
| `welcome-forge-connected` | the Forge step's connected Forgejo and first repository, with Continue |
| `welcome-forge-github` | the Forge step's accepted GitHub installation, with Continue |
| `github-app-details` | GitHub's own-App form, with masked key input |
| `github-install` | GitHub's connected App, installed nowhere yet |
| `github-picker` | the installation's reached repositories, ready to adopt |
| `github-adopted` | the adopted GitHub repository and its primary Open act |
| `welcome-model-choose` | the welcome's Model choices, ChatGPT and Claude |
| `welcome-model-forge-under-way` | Model at first run step 6, Forgejo setting up, its Forge pane and Each step |
| `welcome-model-forge-fetching` | Model at first run step 6, Forgejo fetching at 42%, its bar filled and the figure at its end |
| `welcome-model-forge-fetching-bytes` | Model at first run step 6, Forgejo fetching with no size, its moving bar and 41 MB at its end |
| `welcome-model-forge-done` | Model at first run step 7, Forgejo connected, its marked Forge pane without Each step |
| `welcome-model-forge-failed` | Model at first run step 9, Forgejo failed, its Forge pane and Each step |
| `welcome-model-claude` | Claude’s welcome command and I’ve signed in, with no Name field |
| `welcome-model-key-details` | the welcome's API key form, without an unsupported display-name field |
| `welcome-model-connected` | a usable ChatGPT plan, its ticked status and Continue to Ready |
| `welcome-model-away` | the page waiting for ChatGPT’s sign-in, its note, "Open it again" and folded paste |
| `welcome-model-signing-in` | the same note while signing in to ChatGPT inside the welcome |
| `welcome-model-signin-ready` | the connected state after signing in to ChatGPT |
| `welcome-model-declined`, `welcome-model-renewed`, `welcome-model-refused` | the other sign-in endings inside the welcome |
| `welcome-paste-waiting`, `welcome-paste-written` | the waiting page’s folded paste, with an empty or supplied secret address |
| `welcome-paste-too-long`, `welcome-paste-address`, `welcome-paste-state` | the length and address refusals, ready for a fresh paste |
| `welcome-paste-sending`, `welcome-paste-finishing` | the finish request in flight, then accepted |
| `welcome-paste-too-many`, `welcome-paste-ended`, `welcome-paste-expired` | the restart endings, with no paste field |
| `welcome-paste-lost`, `welcome-paste-unsent` | the address cleared after a lost or unsent finish request |
| `welcome-ready-starting` | a usable subscription's trial at Starts: its first step ticked, Claude Code starts on this machine under way |
| `welcome-ready-answering` | the trial past Starting: its first two steps ticked, Claude answers under way |
| `welcome-ready-answered` | an answered trial: ticked, answered in N s, no Each step |
| `welcome-ready-stopped-starting` | a trial whose run ended before starting: stopped at its second step, the run's why, Try again; Ready and Create stay usable |
| `welcome-ready-stopped-answering` | a trial whose run ended before answering: stopped at its third step, the run's why and Try again |
| `welcome-ready-waiting` | a running trial with a wait that won't settle in a moment: stopped at its step, the wait's words, no Try again until it ends |
| `welcome-ready-waited` | a trial ended after a minute: the last wait's words, or no answer within a minute; Try again |
| `welcome-ready-plan` | a plan's trial: Signed in to your PLAN plan ticked, then PLAN answers under way |
| `welcome-ready-key` | a key's trial: Key added ticked, then PLAN answers under way |
| `welcome-ready-checking` | Ready from its first frame, the declaration or host check under way, Model's line at its first step, its bar and Each step |
| `welcome-ready-copies` | Ready offers Widget and names the two repositories its act will copy into the project |
| `welcome-ready-none` | Ready with neither forge nor model, the pane's two lines, their inline connect links and the offered playground |
| `welcome-ready-not-yet` | Almost ready, the settled check without a usable login, Model's line stopped at its first step, Try again, Sign in again and Each step |
| `welcome-ready-signing` | Ready while ChatGPT signs in, Model's line in the pane as it is |
| `welcome-ready-under-way` | Ready while Forgejo sets up, Forge before Model, Each step and the note below the form |
| `welcome-ready-adopted` | Ready with Forge and Model connected, the project form and Back to Model |
| `forge-setup-failed` | the failed setup's reason and neutral retry act |
| `forges-github-install` | GitHub's Forges row before installation, with its primary Install act |
| `forge-admin-details` | a managed Forgejo's admin Details, with its first-repository fieldset and legend |
| `connect-forge-choose` | the shared Forgejo and GitHub cards, with Your Forgejo below |
| `connect-forge-connected` | shared cards with the connected Forgejo marked and GitHub primary |
| `connect-forge-yours-admin` | Your Forgejo’s admin form, with the other form absent |
| `connect-forge-yours-tokens` | Your Forgejo’s bot tokens, with the admin form absent |
| `adoption` | the adoption form, empty |
| `adoption-filled` | the adoption form, filled in |
| `adoption-refused` | the adoption form after a refusal, its alert |
| `sheet-adopt` | Add repositories with a long list, search, count and disabled project rows |
| `sheet-searching` | the sheet searching with a pick retained |
| `sheet-none` | the completed search with no match and Not listed opened |
| `sheet-few` | a small completed list, no search or count, More open |
| `sheet-empty` | the sheet with no reachable repository |
| `sheet-copy` | Copy public, one address per line |
| `sheet-copy-lines` | invalid copy lines and their repairs |
| `sheet-empty-source` | Make empty, with organization, name and the destination forge |
| `sheet-from-home` | `home-add-repository`'s deliveries, To begin's "Adopt another…" pressed: Add repositories over the project's Repositories, as that entry opens it for a project with no repository (`../forge/family.md`, 3) |
| `sheet-from-chats` | `chats-no-repository`'s deliveries, its band's "Adopt a repository" pressed: Add repositories over Repositories, as a project with no repository opens it (`../forge/family.md`, 3) |
| `repositories-making` | copies under way and failed, the forge’s reason, Copy again and Dismiss |
| `chats-reference` | the reference world's chats |
| `chat-reference-T1` | the reference world's chat T1 |
| `chat-solo-T1` | the solo world's chat T1, closed: no composer and no chooser |
| `chat-solo-T6` | the solo world's chat T6 |
| `chat-held-task` | the solo world's chat T10, its proposal's reason saying a task is held |
| `chat-first-run-2-T1` | the first run's chat T1 at step 2, its change running |
| `chat-first-run-3-T1` | the first run's chat T1 at step 3, its change landed |
| `chat-first-run-4-T4` | the first run's chat T4 at step 4, waiting at rest for the person’s words |
| `playground-resetting` | the first run's Chats while the playground resets: "Leaving the playground…", and no playground's start |
| `chats-first-run-5` | Chats after the first plan connects |
| `inbox-reference` | the reference world's *Needs you* |
| `accounts-reference` | the reference world's accounts |
| `accounts-first-run-1` | the first run's accounts at step 1: the playground's account |
| `signed-out` | signed out: the top bar's mark alone, and the column "Signed out" with the printed link's words (`look.md`, *Outside a project*) |
| `out-of-date` | a bundle older than the engine: the column "Out of date" |
| `no-project` | signed in, `me` naming no project, on a deployment with no welcome: the top bar with the person and "Sign out", and the column "No project yet" with the form "New project" (`../pages/README.md`, *With no project yet*) |
| `approve-typing` | the approval page at `/terminal`, no code: "Sign in a terminal?", the field holding a code typed in lower case with a space, `wdjb mjht`, and Check (`../pages/README.md`, *The approval page*) |
| `approve-unread` | the same field after Check with a code that doesn't read, `WDJB-MJH`: its alert |
| `approve-asking` | `/terminal?code=WDJB-MJHT` read waiting, asked two minutes before: the code, "Asked 2 min ago", the warning, Approve and Refuse |
| `approve-approved` | the same approved, waiting for the terminal's take: "Approved" and the code |
| `approve-signed-in` | the same taken: "Signed in. You can close this tab." |
| `approve-refused` | the same refused |
| `approve-expired` | the same read expired |
| `approve-used` | the same decided twice: Approve answered used |
| `approve-unknown` | a code the engine doesn't know: "Unknown code" and "Type a code" |
| `approve-signed-out` | the approval page with the browser signed out: "Sign in first", the printed link first, then the page's address again |
| `approve-signed-out-google` | the same with Google's way given: "Sign in, and this page comes back with your terminal's code." and "Continue with Google", its `then` `/terminal?code=WDJB-MJHT` |
| `approve-busy` | a read answered busy: "temper couldn't answer just now" and Try again |
| `chats-empty` | the chats with none, outside the playground |
| `inbox-empty` | nothing needs the person |
| `team` | hand-made deliveries after the whole story's `team/team.html`: two people, both owners of the project, the person reading with two questions and a review waiting, "Three things wait on you.", and Sam with a question waiting on one of Sam's chats, "One question waits on Sam."; "Two of you."; "owners of shop" at People's head's right, and each box "Owner"; a service member left out |
| `team-alone` | one person, the project's owner, nothing waiting: "Just you.", People's head its title alone, and one box, "Owner" and "Nothing waits on you." |
| `night-team` | `team` with the night look on |
| `chat-closing-below` | a chat at rest with one task below it: a singular count and the hairline |
| `chat-closing-proposal` | a chat at rest with one proposal: a singular count and the hairline |
| `task-releasing-uncertain` | a release with an uncertain effect and its two choices |
| `chat-closing-work` | a run, two delegates with opposite fates, and two proposals: what closing changes |
| `chat-closing-failed` | the same close refused: the sheet keeps its choices and says why |
| `chat-closed` | the requester closed it: one end line at 21:05, with a quiet bar |
| `chat-paused` | a chat its person paused: "Paused", Resume and Close, and the composer's hint "Sending resumes this chat." |
| `chat-closing-start-over` | an Ask temper chat's Start over confirmation |
| `task-closing` | an agent's task Close confirmation, with a task below it |
| `task-paused` | an agent's task its person paused: "Paused", Resume and Close, and no own held card |
| `proposal-accepting` | a proposal's Accept confirmation |
| `proposal-rejecting` | a proposal's Reject confirmation with its reason |
| `chat-closing` | `chat-solo-T6` with Close pressed: its confirmation |
| `chat-rejecting` | `chat-solo-T10` with Reject pressed: the reason's box |
| `task-held` | an agent's task's plan with a held task, Release offered |
| `task-releasing` | `task-held` with Release pressed: its confirmation |
| `connect-model-choose` | shared plan cards and key rows, with the owner’s quiet holder switch |
| `connect-model-connected` | a connected plan marked on its card, with the first unconnected plan primary |
| `connect-model-key-details` | a key's secret field and Add key |
| `connect-model-program-details` | a program’s panel, command, Copy, login name and I’ve signed in |
| `connect-model-signing-in` | the waiting note, "Open it again" and folded paste |
| `connect-model-declined` | plan usage declined, with another sign-in and a key way |
| `connect-model-refused` | another holder's account refused, with another sign-in |
| `connect-model-key-lost` | a key request's answer lost, its alert asking for the value again |
| `task-reference-T28` | the reference world's change T28, queued 2nd after T26 |
| `task-reference-T24` | the reference world's change T24, landed, its ended task T25 |
| `task-reference-T12` | the reference world's goal T12, its tasks with its ended delegates T13, T14 and T24 folded |
| `task-solo-T9` | the solo world's agent T9, its run's committed calls and words within the shipped window |

## 3. Its tests

In `crates/temper-client-gallery-tool`'s own tests, in the view's world: each state's tree
read by skein's fake person (`../../testing/fakes.md`, 4), as the
gallery builds it.

- **The names:** the states' names, in order, are `crates/temper-client-gallery-tool/names.txt`,
  kept by hand, so a name gone, changed or added fails until the file
  says so.
- **Each state holds what its name says:** a table of rows, each a
  name, its page's heading and the one part the state is about, found
  by role and name (the proposal in `chat-solo-T10`, the three
  suggested requests in `chat-new-first-run-1`, the alert in
  `adoption-refused`).
- **The print:** the program run into a scratch directory writes every
  name's page, the index linking each, and the bundle's stylesheets in
  `scripts/bundle.sh`'s order, read from that script.
- **Their cost:** together at most 1 s (0.5 s before the user's doubling of
  10-22, provisional) of the default suite; no fuzzy test.
