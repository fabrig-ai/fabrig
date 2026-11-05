# ui-chat-draft: the document beside a chat, Start goal, and the rail's menu

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work exactly like the greenfield mock, modulo what look requests have changed. There is no supported deployment: formats, protocols and public interfaces may change freely. The user is asleep and trusts these decisions; they are taken, not questions.

It starts after `ui-chats` has landed: every chat in the session layout, `Address::NewChat`.

## What is being built

A chat keeps one Markdown document beside its conversation. Its agent writes it; the person reads it in Chat, Split or Draft; a goal starts from it. The configuring chat's "How Hello works" is the same mechanism, so `configure-chat-draft` lands here and there is one document pane, not two. The rail's "+ New chat" gains its menu.

## The mock and the requests

- **The mock:** branch `origin/ux-ideas/greenfield`, `temper/docs/design/client/greenfield/`: `chatPage`, `startSheet`, `startGoal`, `sendChat`, `chatsPage`, the `addDraft`, `show`, `mode` and `Ctrl+\` handlers in `app.js`; `mock.css`'s `.seg`, `.session`, `.doc`, `.doc-*`, `dialog.sheet`, `.option`, `.field`, `.text`. To see a mode, a throwaway copy of `chat.html` sets `localStorage['temper-greenfield-v2']` to `{"panel":{"away-mode.md":"split"}}` (or `"full"`) first; `chat.html?id=T4`.
- **Look requests,** on `origin/look-requests` at `b13703aa2b`: `configure-chat-draft` (`727f3523ee`) and `new-menu` (`4194189e15`): read each whole commit message and its pages: `temper/docs/design/client/journeys/configure-chat/02.html` to `04.html`, `new-menu/01.html`, and `overlay.css`'s rules from those commits, which are the look to copy. **The procedure is `temper/docs/design/draft/look-requests.md`: every step of sections 3 to 5, and its account.** Both were held; the user's request above releases them. Each request's commits carry its `Look-request: NAME` trailer directly above `Co-Authored-By:`; its last commit holds its account table. `configure-temper-own-way` is not this plan's.
- **Where they and the mock differ,** the request wins: the rail stays and folds to the 56-pixel strip in Split; Draft keeps the composer on white at the foot; a chat opened again starts in Chat (no stored mode); phone has Chat and Draft, and Split left on a wide screen shows as Draft. Where a request says something only of the configuring chat, it holds of every chat.
- **Left out of `configure-chat-draft`:** the drawing. skein's `diagrams.md` waits for the user; a fenced block reads as code; the agent isn't told to draw.

## Decisions already taken

1. **The document is a jig note** (`jig/docs/design/draft/notes-memory.md`, 5, this part alone) in the chat's own scope. No `kind`: every note in a chat's own scope is its document. **One per chat:** the lowest-numbered. Its description is its title, its body Markdown. The draft's kinds, party scope, working notes and section 6's own bound are not taken up; bring that draft and its status up to date on what was.
2. **Live reading is no new watch:** section 2 of the client's part.
3. **A goal takes the body in its words,** a snapshot: "The goal takes the draft as it is now."
4. **No file drafts.** The mock's rows for drafts no chat holds, `chat.html?draft=`, and "draft, not discussed" are out.
5. **A document's name:** the mock's `slug` of the title, 28 characters, plus `.md`: `away-mode.md`. `.doc-path` says `docs/design/draft/NAME.md`, and the goal's words tell it to commit the draft there first.
6. **How it runs:** see the sheet below; **When it needs you** is out until the away switch exists.
7. **No `parts`.** One function in the client's domain, `pages::drafts_offered(me, project)`: true while `me` lists a template with `tracked` and no conversation flag. It gates "+ Draft", the Drafts filter, the lede's words and "Start goal…".
8. **Agent-started goals** (`goal` tool) keep `ui-chats`' line in the conversation; a row says "started goal X" only for the person's own starts.

## The engine

### jig

- **A chat's own scope.** `jig-domain-core/src/routing/mod.rs`, `ToolKind::Note`: `notes::Scope::Goal { project, goal }` is admitted when `Some(goal) == goal_of(task)`, or when `goal == task.number` and the task is a root conversation (no tracked ancestor, flagged conversation). `note_readable` and `routing/run/preparation.rs`' brief scopes do the same, the chat's scope first, so the agent's brief lists its document. Stored format unchanged: the client's `NoteScope::Task { project, task }` already maps here. A party's read of that scope needs the party to see the project: confirm in `reads`.
- **The note's answer.** `Event` `NotesWritten` and `CallAnswer::NoteWritten` gain `lines`, `added`, `removed`, `first`: the body's lines; lines of the common prefix and suffix trimmed, what remains of the new and of the old; `first`, the 1-based new line where they first differ. A pure, bounded function in `jig-domain-notes`, tested; none, `added = lines`, for a first write. No LCS.
- **A document's room.** `notes.body_bytes`: 65,536 in `standard`, 32,768 in `small`, the box's as `standard`; receive bounds, page bounds and derived limits follow; `jig-protocol-config` tests hold. `tasks.spec_bytes` (131,072) already holds a goal's words.
- **The project's documents.** Contract and wire: a new `NoteScope::Tasks { project }`, reads only, onto the core's existing `Read::Notes { project }`, restricted to `notes::Scope::Goal`, each `Note` with an empty `body` marked cut. Any write naming it is refused. Pages as any; the client reads them all, to the bound of its rows.

### temper

- **Stock** (`temper-service/src/stock.rs`, `protocol/configuration.md`, 13.4): `notes: Scopes(1)` where a chat's authority comes from, today `Scopes(0)`, so that a chat may write its own scope and only it; charter 3's families gain note and recall (its number follows; each charter's host part still fits). A goal and an advisor keep "no notes".
- **`temper-engine-smith/src/answers.rs`:** `Note N revision R written · 64 lines`, first write; `Note N revision R written · +18 −6 · line 31`, a later one.
- **Instructions:** `stock/chat.txt`, under the chat's own paragraph: when the person asks for a draft, or says "write this up", keep one document: `note` with scope goal, the project and your own task number, description its title, body Markdown beginning with `# TITLE` and a muted line "Draft, DATE, with the user.", whole each time, naming the note and the revision you read, one `##` section per topic, open questions under "## Open questions". Say in a sentence what changed; don't repeat it. Mermaid isn't drawn. `stock/configuring.txt`: the same, its title "How PROJECT works", written once the person's way is understood, kept current as the conversation changes it (`domain/agent.md`, 8.1). Confirm a run knows its own task number from its brief, or add it there.
- **A person's facts.** The per-person family of `Dismissal` (`temper-engine-domain-forge/src/dismissed.rs`, `temper-client-domain/src/forge/dismissed.rs`, wire, store, restore, `erase_project`): rename the type `PersonFact`, keep `ToBegin { project }`, add `GoalFrom { project, chat, goal }`. Same request, same watch, same bound: `limits.dismissed()` grows by a factor of 8. Over the bound, the answer is `TooLarge`, and the link alone is lost. `domain/forge.md`, 5.5 and `client/README.md`, 4.6 say so.

## The client

### 1. The page's state (`temper-client-domain/src/pages/chat.rs`)

`Chat` gains: `mode: Mode { Chat, Split, Draft }`, `Chat` on open, per page; `document: Option<Document { note, revision, title, body, changed }>`; `wanted: bool`, a read wanted while one is out; `sheet: Option<Sheet>`. New acts: `Act::Mode { mode }`, `Act::ShowSection { line }`, `Act::AddDraft`, the sheet's, `Act::StartOver`.

### 2. Live reading

The answer to "how a chat's draft is read live": `Read::Notes { scope: NoteScope::Task { project, task }, description: None, after: None }`, on the same retries as an activity read:

- once the task's document has arrived;
- again when the thread shows a `note` call not seen before, and when the run's state moves off a call to `note`.

One read out at a time; `wanted` asks once more on its answer. The chat page already watches the task and its run, and only the task's own run writes the note, so nothing else can move it. The newest answer replaces `document`. A read refused or empty leaves it as it was.

### 3. The tool line

A `note` call whose answer parses, in `.turn-tool` (the mock's `turn` in `chatPage`): pen; "Wrote" for a first write, else "Edited"; `.file` NAME; `.turn-tool` detail "64 lines" or "+18 −6"; "Show", a button. It isn't counted in the "N tool calls" disclosure. A failed one stays in the disclosure. Parse the words in the client's domain, as `delegation::results` does, with a test against the words `answers.rs` writes.

"Show": moves to Split (a phone: Draft), unless already in Split or Draft, and moves the focus to the section holding that call's `first`, which brings it into view, and flashes it (`.doc-sec.flash`, `@keyframes flash`).

### 4. The bar

`ui-chats`' right part, after the byline, once a document exists:

- `.seg`, a group named "The draft": "Chat", "Split", "Draft", each its icon (`ICON.chat`, `.split`, `.doc`, added to `files/`), the chosen `aria-pressed`. `Ctrl+\` steps through those shown: the shell's shortcut takes letters, digits and Escape only, so extend `skein-ui`'s assertion to name this key, or bind it in `temper-client-shell-web`, whichever is smaller.
- With no document and `drafts_offered`, and the chat not ended: `.btn.small` "+ Draft".
- `.btn.primary.small` "Start goal…" once the document has a section, it's a chat of a project with a home repository, and no goal is linked; with a linked goal, a link `.btn.small` "Goal TITLE →" to `Task { task }`. Not on a configuring chat, which has "Start over" instead: `configure-chat-draft`'s words, its confirmation through `crate::confirm`, `Act::StartOver` closing the chat with the chat's own close and then opening `Configure { project }`.
- The composer's hint gains " · Ctrl+\ switches the draft" while a document shows.

### 5. The document pane

Once for both kinds of chat, from `configure-chat-draft`'s overlay rules, renamed to live in `temper.css`: `temper-mode-chat`, `-split`, `-draft` on the session, `.temper-doc`, `.temper-doc-inner`, `.temper-doc-path`, `.temper-doc-sec`, `.temper-doc-changed`, `.temper-doc-note`, the rail strip with `.temper-rail-unfold` and `files/menu.svg`. Draw only what these name: `.temper-seg`'s rules carry over too.

- **Its line,** `.doc-path`: left `docs/design/draft/NAME.md`; a configuring chat's "T7 · not proposed yet", or "Proposal 12 · waits for an owner", or "Accepted at 14:03", from its proposal cards; right "edited 1 min ago", `TimeForm::Ago`, lower case.
- **Sections:** the body is split at each `#` and `##` line, each section read by `skein_ui::markdown::read(.., 2, b)` into a slot `.doc-sec`, keyed by its heading, so that a change keeps the others' nodes. The heading level 2 is drawn as `.doc-inner h1`, 3 as `.doc-inner h2`; the first section's first paragraph as `.doc-muted`. Tables as `configure-chat-draft`'s.
- **The tint,** `.doc-sec.changed`: the section holding the newest `note` call's `first`, or for a first write the first `##` section; under it `.changed-note` "Added by T4 3 min ago" or "Changed by T4 just now".
- **Empty,** `+ Draft` pressed and nothing written: `.doc-empty` "The draft appears here as you write it together.", `.doc-path` left with the mock's `untitled.md`, right "writing".
- **Terminal:** in Chat, the conversation alone; in Split or Draft, the document after it as a region named its title. Goldens hold what they mean.

### 6. + Draft

- **On a chat:** `Act::AddDraft` goes to Split with the empty pane, and sends the chat `Ask::Message` "Write this up as a draft beside us." The meta line, when the thread is empty: "A chat that writes NAME. Say what it should hold."
- **On a new chat:** `Address::NewChat { draft: bool }`, `/chats/new?draft=1`: `+ Draft` in the bar (`.btn.small`) moves there. The empty pane in Split, meta line "A chat that writes a draft. Say what it should hold.", placeholder "What should this draft hold?". The first message starts the chat with, under the person's words after a blank line, "Write this up as a draft beside us."; those words are visible as sent. The chat opens in Split.

### 7. Start goal

`Sheet { name, way, budget, out, refusal }`. `Role::Dialog` named "Start a goal from NAME", `dialog.sheet`; Escape and "Cancel" close it; on a phone, the mock's width rule.

- `h2` "Start a goal from NAME", muted "The goal takes the draft as it is now. Later edits reach it through Amend."
- `.field` "Name", `.text`, the slug without `.md`, focused.
- Fieldset "How it runs", `.option` radios, from what exists, in this order, the first checked, its small line ending " PROJECT’s default.":
  - each offered template with `tracked`, no conversation flag and a symbolic resource (its own goal branch): "Plan, tasks, land whole", "A plan from the draft, its tasks on a goal branch, landed as one change once all are checked.";
  - stock template 2: "Task by task", "The same plan; each task lands on its own once checked.";
  - stock template 2 agreed: "Plan, then wait for me", "Proposes the plan and its tasks, and starts nothing until you approve.".

  Then a muted link "The project’s ways of working", to `Settings { project }`.
- `.field` "Budget", `.text` 7em, a number, in the deployment's unit, no `$`, its default what `temper ctl goal start` gives; beside it, muted, "asks you before going over".
- `.foot`: `.btn.quiet` "Cancel", `.btn.primary` "Start goal", disabled while the name is empty or one is out.

**Sent:** `shapes::set_goal_from(me, project, template, GoalDraft { words, executor, budget, deadline: None, priority: None, llm: None }, reason)`, executor charter 1 where the template lets the creator choose. `words`, within `tasks.spec_bytes`, refused in place if over:

```
NAME

Implement this draft. First commit it, as it stands, at docs/design/draft/SLUG.md.

BODY
```

followed by the line `goal`'s root gives a goal, so that charter 1's instructions read it as a goal (`domain/agent.md`, 4.3.2), and for "Plan, then wait for me" its agreed line too (8.2). Both lines move to one constant the root and the client reach.

**Answered:** `Judged::Create`: the `GoalFrom` fact is sent and the goal's `Task { task }` opens. `Judged::Propose`: the sheet closes with a line above the composer, "Asked RECIPIENT to approve it; it waits in Needs you." `Judged::Refused`, `NotOffered`, or a refused answer: jig's words under the form, kept. **After:** the fact's chat shows `.turn-tool`: pen, "Started goal", `.file` TITLE, its state, "Open", placed among the turns by the time it was made.

### 8. The Chats page

- `ChatsFilter::Drafts`, `?state=drafts`, between Live and Closed, while `drafts_offered`: the rows whose chat has a document, in the page, reading nothing more.
- The page reads `NoteScope::Tasks { project }` as it opens, and watches the facts; the row's top line left: "T4 · writes away-mode.md", then " · started goal TITLE" with a link's title from the goal's chip by `Subject::Chips`. A note written later is seen on the next open: no live read here.
- The lede grows: "A chat answers a question or changes the code, and can write a draft beside it. A draft becomes a goal for bigger work."

### 9. The rail's menu

`new-menu`, as its commit says, on `a.temper-rail-new`, which becomes `div.temper-rail-new` with the link, the 36-pixel chevron button "More ways to start work" and `Role::Menu`. The items: "Chat", "Ask a question or ask for a change.", `NewChat`; "Goal", "Agents plan a larger change and land it.", `NewChat { draft: true }`, the mock's own way to a goal (its Goals page says "Write a draft with a chat, then start a goal from it."), in place of the commit's Board, which has no address; then "All ways to start work", muted, to `Settings { project }`, or `HowWorkRuns` if it has an address by then. "Goal" shows while `drafts_offered`. The phone doesn't show the entry.

## Design and the cut

- **Design first:** `jig/docs/design/domain/engine.md`, 10; `jig/docs/design/domain/authority.md`, 3; `jig/docs/design/client/domain/contract.md` (the read, the scope); `jig/docs/design/draft/notes-memory.md` and its status; `temper/docs/design/client/README.md` (3.5, 4.6, 5.3, 5.5's chooser), `client/browser.md` (2.3, 2.5), `client/journeys.md`, `domain/agent.md` (8.1 and a chat's document), `domain/forge.md` (5.5), `protocol/configuration.md` (13.4), `draft/template-places.md` where the menu's items are chosen. Their status files. `tools/citations --all` stays at zero.
- **One order,** `needs` chained: design; jig (scope, answer, bound, list read; a world proving a chat writes, a party reads, and a goal and another project's task can't); temper's stock, instructions and facts, with a scripted turn for the fake model that writes a document (`domain/testing.md`, 2.4); domain state and live reading; `configure-chat-draft`'s pane on a configuring chat, with `Start over`; a chat's bar, tool line and pane; `+ Draft`; Start goal; the Chats page; `new-menu`; recording. Conflicts are in `temper.css`, `temper/tools/gallery/src/tests.rs`, `names.txt`, the terminal's goldens, and journey prints.
- **Gallery:** add `chat-draft-chat`, `chat-draft-split`, `chat-draft-draft`, `chat-draft-empty`, `chat-new-draft`, `chat-start-goal`, `chat-start-goal-refused`, `chat-goal-linked`, `chats-drafts`, `configure-chat-document`, `configure-chat-document-split`, `rail-new-menu`.
- **Recording:** a new journey, `chat-draft`: a fresh fake box, a chat, `+ Draft`, two turns of the fake model, Split, Draft, Start goal, the goal's page. Say what stops it, if anything does, and end that task with a `Question:` rather than working round it.

## The proof

For each of these, a render of the client beside the mock's page or the request's hand-made page, at 1440, 1000 and 420 wide: a chat in Chat, Split and Draft; the empty pane; a section tinted; Show; the sheet, and refused; "Goal X →"; Chats under Drafts; New chat with a draft; a configuring chat in the three modes; the strip, and the rail open over it; the menu open. Saved on the builder under `~/src/fabrig/ui-chat-draft/`, one folder per look request, outside any worktree, never committed; the report names the builder and each difference left, with why. Throwaway render scripts stay untracked and are gone before the push.

## Size

One cut is at most 90 minutes. Budgets and timeouts are the user's. If the cut needs more, split at "temper's stock" into a first plan of engine parts alone, ending with the fake model's document proved through `temper ctl`.

## Sources

- `temper/docs/design/client/greenfield/app.js` (from `origin/ux-ideas/greenfield`: not on main)
- `temper/docs/design/client/greenfield/mock.css` (the same)
- `temper/docs/design/draft/look-requests.md`
- `temper/docs/design/draft/template-places.md`
- `jig/docs/design/draft/notes-memory.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
