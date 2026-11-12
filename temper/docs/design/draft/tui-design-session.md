# tui-design-session: a design session in the terminal, T2 of whole-story

The spec is `temper/docs/design/draft/surfaces.md`, section 11.2, with 11.1's and 11.4's `/draft`. It's carried out as `temper/docs/design/draft/whole-story.execution.md`, T2 says, with the mockup `temper/docs/design/client/tui/whole-story/design-session.html` as its look and `surfaces.md`, section 13, for the mockups' precedence. Read `surfaces.md`, section 11, whole, then T2's entry. `surfaces.md`, 11 revises `temper/docs/design/draft/tui.md`, section 3, *Drafting* (`tui.status.md`), and makes this v0.1.0 work. It designs and builds together, writes its design into `temper/docs/design/client/tui/pages/README.md`, with status entries for what it leaves, and lands whole.

It builds on P6, landed: the drafter is charter 12 and a design session template 6 (`temper/docs/design/domain/agent/agents.md`, 4); the client knows one by `pages::chats::DRAFTING_TEMPLATE` and `pages::chats::drafting`, and starts one with `Chats::start` and `draft` (`temper/docs/design/client/pages/README.md`, *A chat's document*, *A design session*). The drafter's instructions, `temper/crates/temper-service/src/stock/drafter.txt`, say how it asks and writes: read them first.

## What it builds, ranked

1. **`/draft` starts a design session,** from the launchpad's Start and as a command, through `Chats::start` with `draft`, as the web's new chat with a draft does: template 6 where `me` lists it, a chat otherwise. Inside a session or a chat with a document, `/draft` shows or hides the draft (11.4). Start and the command list gain `/draft` (11.1); the status entry T1 left for it goes.
2. **The draft beside the conversation, in a side pane:** the chat's document, read as the web reads it (`temper/docs/design/client/pages/README.md`, *A chat's document*: the same reads, no new watch), folded, half the screen or all of it, Ctrl-P cycling them (`tui.md`, 3). A design session opens with the pane at half; any chat with a document can show it. The sections the last turn wrote are marked "new" at their right. No diffs. A flow in the draft prints as its fenced text. Where the terminal is too narrow for two readable halves, half shows as all: the plan sets the width and names it in the design.
3. **One question at a time:** the drafter asks as numbered choices at its reply's end, its recommendation first with its reason, "Tell me something else" last. The TUI reads such a reply as a prompt with numbered choices, as a chat's confirmation is drawn today: a number answers with that choice's words as the person's message, and the last choice leaves the box to the person. A reply with no such list reads as today.
4. **`/goal` starts a goal from the draft,** through the domain's start sheet as the web's "Start goal…" uses it (`pages::start_goal`, `Act::OpenStart`, `Act::StartGoal`), its choices the project's ways of working, kind 7's tracked templates (`temper/docs/design/client/pages/goal.md`, 3), the preferred way first, as a prompt with numbered choices. It's offered once the document has a section, on a project with a home repository and no goal linked; once a goal is linked, the quiet line names it and `/open` opens it.
5. **Sittings:** a session is kept. Opened again, from the launchpad's Where you left off or `/chats`, it resumes from its draft and conversation, the pane at half. Where you left off says "a design session" for one, as 11.1 lists what a row is.
6. **The quiet line in a session,** as the mockup has it: the project, "drafter", the model and effort; then the pane's state, "Ctrl-P folds" and "/goal starts a goal from the draft".

## Out

- Talking with a role, `/talk` and `/tell`: T3.
- The drafter's charter, template and instructions: P6's, unchanged here.
- The web's design session: P6 built it.
- A diff of the draft, and a picture of a flow: none for v0.1.0.

## Tests and journeys

- **Golden frames** (`surfaces.md`, 14): a design session with the pane folded, half and full, at 80 by 24 and wider, in both themes; the drafter's question as a prompt; the `/goal` prompt of ways.
- **Rows of existing tests:** `/draft` choosing template 6 or a chat; Ctrl-P's cycle; a reply's choices read as a prompt, and one with none read as today; the "new" marks after a turn's write.
- **A journey, a design session to `/goal`,** recorded as `temper/docs/design/client/tui/journeys.md` says, where a fake box reaches its states. If the fake box can't have the drafter write a document, the plan says what's missing in its report and leaves the journey with a status entry.
- The mockup is hand-made: say in the plan's report where the build departs from it and why.

## Overlaps

- **`temper-client-domain`** is shared with the web's owner: the smallest change the TUI needs, named in your report. The web's design session and start sheet stay as they are.

## Walk

`walk = false`. The walk, each screen against its mockup in a real terminal, follows the landing.

## For every task

- **A plan is complete when it lands.** No walk, render check or throwaway driver in it. Recording a journey again is generation, which a task runs.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled (`cargo check --tests`), not run. A slow-tier test whose expected output the change makes stale is updated in the same task, compiled, and named in the report.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Budgets and timeouts are never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration; note it in your report. A small edit a task needs just outside its file list is made and named in the report. Ask only what truly blocks the work.
- **Recording flakes:** a re-record that stops on a moving screen is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait and say so.
- **Generated files** that a change makes stale are that task's to regenerate.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
