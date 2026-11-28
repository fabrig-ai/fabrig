# Any project: temper without fabrig's way

Draft, 2026-11-27, from a session with the user that day. temper is being made "more 'consumable' by any project": [ways.md](ways.md) and [pair.md](pair.md) move configuration and ownership into a repository's `.temper/`, and the user is starting on how a person meets it. What was still missing is "how ways like 'goal' and 'milestone' (or really any complex way the users may want, possibly fully custom, see the ways-gallery.md for reference) would work for a random project". Two questions: how much the stock is "'biased' towards fabrig's own way of developing, in particular the docs/design approach, with its drafts, .execution.md's, .status.md's"; and "what would a random project do to take advantage of temper's capabilities? how would a new project just 'plug' temper into it?". The user's direction: "encourage 'design as contract' but not quite require it (and expecially not require a certain definite form of it, like docs/design) and perhaps let it live more in a 'spectrum', where you can still take advanage of temper in your own way". This draft surveys where fabrig's way is built in (section 2), lays design out as a spectrum (3), splits who says what (4), gives a goal a plan of its own that never lands (5), carries goals and milestones to any project (6), and says how a project plugs temper in (7). Second pass, 2026-11-28: on the user's ask for "a 'simulation' of how temper would work" on "a couple of real world third party projects", [any-project.notes.md](any-project.notes.md) walks hono and vector through this draft, and click as far as its first step; 7.5 takes up what the projects' own processes ask of temper. It changes the stock charters ([stock-charters.md](stock-charters.md)), the chat's goal start, and [ways-temper-dir.md](ways-temper-dir.md), section 7's first change.

## 1. In one page

- **Nothing fails without fabrig's way, but the stock brings it along** (section 2). A project with no `docs/design/` can start a goal from words. The stock goal is then told to write "documents that outlast the goal", status entries and a contracts document into a repository that never asked for them, and the web's goal start commits a chat's draft at `docs/design/draft/`. A few words of fabrig's are fixed in code: `focused` and `fuzzy`, `main` and `temper`.
- **Design as contract, on a spectrum of four rungs** (section 3): the request alone; a plan of the goal's own; a record that stays; a design the code follows. Each rung keeps more and costs more. fabrig sits on the fourth. temper works on all four, and its stock goal stands on the second, so every goal gets the plan's benefits and no project gets files it didn't ask for.
- **The project's guide says its convention; temper's words say the method** (section 4). Where design lives and how it is kept is the project's, for every agent and person, so it belongs in `AGENTS.md`, where fabrig already says "the design is the contract". What a goal does with it is temper's, in a new stock charter file, `design`, that defers to the guide. The stock's words lose DESIGN.md's model, which moves to fabrig's own words. The rung is no setting.
- **A goal's own plan lives in `.goal/` on its branch** (section 5): its changes read it in their checkouts, code takes it out before the branch lands, and the landing's pull request keeps it. It replaces the contracts document and the last change that deletes it.
- **Goals start from words, a draft or an issue** (section 6). The draft is optional on the web, and a chat's draft is committed only where the project keeps drafts. A milestone's design sits at the project's rung, and a person, or later a planner agent, cuts it into goals.
- **Plugging in is a ladder** (section 7). It starts with a forge, a repository and a model, and nothing written: Pair and Hand it over run as they are. Each rung after that pays on its own: a guide, a check at delivery, CI, ways of the project's own, a higher design rung. Ask temper reads the repository and proposes the setup as one change a person reads, "Set up temper here", in place of "Add .temper/ with your ways".
- **A project's own process comes first** (7.5): its policy on AI, read before anything is proposed; what each change carries, its pull request template and its changelog entry; whose name is on a commit, for a CLA; the models named by code; the forge's other bots and reviewers; and no rewritten history. Two real projects walked through this draft found these, not design, to be what stands in the way ([any-project.notes.md](any-project.notes.md)).

## 2. Today: where fabrig's way is built in

From a survey of main at `0d918ae9bc`, its five strongest claims checked against the code.

### 2.1 In the code

| Where | What it assumes | Here |
|---|---|---|
| `temper/crates/temper-client-domain-pages/src/pages/chat/sheet.rs`, `words` | a chat's draft becomes a goal's words: "Implement this draft. First commit it, as it stands, at" `docs/design/draft/` and the draft's slug | names no path (6.1) |
| `temper/crates/temper-client-domain-pages/src/pages/start_goal.rs`, `send` | Start goal sends nothing without a draft | the draft is optional (6.1) |
| `temper/crates/temper-worker-domain-checkout/src/hold/design.rs` | a goal's own delivery takes Markdown alone, anywhere in the tree | kept, `.goal/` included (5) |
| `temper/crates/temper-engine-domain-forge-vocabulary/src/change_words.rs`, `report_lines` | a failed test is `focused` or `fuzzy` in a repair's words | the project's suite name, or none (6.3) |
| `temper/crates/temper-engine-service-stock/src/stock/ways.rs` | Draft first's rules name the branches `main` and `temper` | goes as Draft first becomes Milestone ([ways-recipes.md](ways-recipes.md), section 4) |
| `temper/crates/temper-client-domain-pages/src/pages/goal.rs`, `holds_branch` | a goal's branch ends `temper/ROOT/gROOT` | the adoption's prefix (6.3) |
| `temper/crates/temper-worker-protocol-git/src/invocation.rs` | fetches `refs/temper/counts`, fabrig's published test counts | harmless, since its absence is a path; noted only |

### 2.2 In the stock's words

- **The goal's designer** (`temper/docs/charters/stock/designer.md`): "A lean design, eventually. Every part of the request designed, in documents that outlast the goal"; "a document of the goal's own named `contracts`, beside the area's design and with its own status where the project keeps status"; a last change deleting it; "the status the project keeps, as it will stand once the goal lands"; "Never edit the draft's text". A lasting design is asked of every goal that writes code.
- **The goal's rules** (`temper/docs/charters/stock/goal.md`): "design and status documents" as what it writes; **Not yet** entries ending `[in flight: change N]` "where the project's design keeps status"; **Design departs** "where the project keeps one"; a spec's acceptance sketched as "world or file", skein's test worlds; "test kits'" types.
- **The coordinator** (`temper/docs/charters/stock/coordinator.md`): a shared shape "going in the contracts document, with a **Design departs** entry".
- **The drafter and the chat** (`temper/crates/temper-engine-service-stock/src/stock/drafter.txt`, `chat.txt`): a draft is a Markdown note with `## Open questions`, generic enough.
- **Hedged, not neutral:** most of these say "where the project keeps", so a model that sees no status files may skip them. What it can't skip is the lasting design: told to write one, it invents a place.

### 2.3 Already the project's

- **fabrig's words** (`temper/docs/charters/fabrig/`, [stock-charters.md](stock-charters.md), section 5): `DESIGN.md`, the focused and fuzzy suites, `tools/tiers/`, budgets, `cargo`, citations, the strict subset. They are an overlay of the deployment's, and move into fabrig's own `.temper/charters/` when it picks its ways ([ways.md](ways.md), section 10).
- **fabrig's hooks** (`.temper/hooks/`, [hooks.md](hooks.md)): `deliver` runs `tools/pre-pr/pre_pr.py`, and `close` settles the status tags with `tools/close/close.py`, as `DESIGN.md`, section 2 asks of "its project's `close` command". The status mechanism is already the project's. Only the stock's words still ask for it.

### 2.4 What a project without a design tree meets

A goal started from words through `temper ctl` runs. On the web, Start goal wants a draft, and a chat's draft is committed at fabrig's path. The designer writes lasting Markdown wherever it guesses design belongs, and status files if it thinks the project keeps them. A project with no `AGENTS.md` and no hooks gets no checks before CI, and its agents find the build and test commands by reading. Nothing refuses. The cost is files the project didn't want, and slower, costlier repairs.

## 3. Design as contract, on a spectrum

### 3.1 What temper gets from a design

- **Changes side by side agree:** two implementers build to one written shape.
- **Questions are answered without the person:** the coordinator answers from what the design says.
- **Reviews judge against something:** a reviewer checks a change against its contract, not only its diff.
- **The next piece of work starts from it,** not from reading the code again.
- **Gaps show:** what is designed and not built, or built otherwise, is written down.

Each benefit costs writing and keeping. A project buys only as much as it values.

### 3.2 Four rungs

| Rung | The contract is | What lands | Who it suits |
|---|---|---|---|
| 1. The request | the words the work starts from: a person's request, an issue, a chat; the specs carry what changes share | code alone | a fix, a small change, Pair, Hand it over |
| 2. The goal's plan | a design the goal writes for itself, in `.goal/` (section 5) | code alone; the plan stays in the landing's pull request | the stock goal, in any project |
| 3. A record that stays | the goal's design, landed with its code where and as the project keeps records: a decision record, a proposal, a page under `docs/` | code and the record | a project that keeps decision records or proposals |
| 4. A design the code follows | a maintained design that goals read and revise, its gaps tracked, as the project's design rules say | code and the design's revisions | fabrig, with `DESIGN.md`'s drafts, design documents and status files |

- **Each rung keeps what the one below keeps,** and adds to it. A goal on rung 4 still keeps its shared shapes in `.goal/` (5.3).
- **No form is temper's:** rung 3 is whatever records the project keeps, rung 4 whatever its design rules say. temper names the rungs to explain them, in pages and in Ask temper's words. Nothing stores them.

### 3.3 Per project and per piece of work

- **A project's usual rung is what its guide says** (section 4): nothing means rung 2 for goals and rung 1 for changes.
- **A way can say its own,** in the words of its charters (`ways-charters.md`, section 2): a hotfix way whose `design` file says "design nothing" runs at rung 1 in a rung-4 project.
- **A piece of work at a lower rung** is started with a way at that rung. Words in a request asking for less design are direction, as any request's are. Nothing checks them.

### 3.4 Encouraged, not required

- **Seen working:** every goal's plan is on its page while it runs, and in its pull request after. A person sees design as contract pay off before being asked to keep anything.
- **Offered at the right moment:** Ask temper, reading a project's landed goals, proposes the next rung once its plans repeat what a record would hold, as a change to the guide a person decides (7.4).
- **Never a gate:** no rung is checked, and no way refuses work for want of a design.

## 4. Who says what

### 4.1 The rule

- **The project's convention is the guide's:** where design lives, its form, how drafts and status are kept, said once in `AGENTS.md`, for every agent and every person, temper's and other tools' alike. fabrig's guide already does ("The design is the contract", `AGENTS.md`, section 1).
- **temper's method is the charter's:** what a goal does with the convention, when it writes, where its own plan goes. It lives in the stock's words, or the project's own charters ([ways-charters.md](ways-charters.md), section 1).
- **No rule in two places** ([ways-charters.md](ways-charters.md), section 1): the stock's words name no project's form, and a guide doesn't tell temper's roles how to run.

### 4.2 The stock's `design` file

A sixth file in the stock's set ([stock-charters.md](stock-charters.md), section 2), read after `goal` in the goal agent's instructions, by its designer and its coordinator alike. Its words, as a first sketch:

```
# How this project keeps design

- **The project's convention first:** where the repository's guide says where design lives and how it is kept (decision records, a design tree, a page per feature), keep the goal's design that way, in its place and form, with its rules for drafts and status.
- **Where it says nothing,** the design is the goal's own: write it under `.goal/` on g, where your changes' checkouts read it. Code takes `.goal/` out before g lands and keeps it with the landing's pull request. Write nothing else the request doesn't ask for.
- **Shapes your changes share,** the types, fields and signatures no lasting document needs, go in `.goal/contracts.md` whatever the convention: binding while the goal runs, gone when it lands.
- **A request that is only words** (a fix, a small change) needs no design: its words and your specs are the contract.
- **Where what is built differs** from a design the project keeps, record it as its convention says; otherwise in your report note.
```

- **The other stock files lose DESIGN.md's model:** "documents that outlast the goal" becomes "the design, kept as `design` says"; the contracts document becomes `.goal/contracts.md`, and its deleting change goes; **Not yet**, `[in flight: change N]`, **Design departs** and "the status the project keeps" leave the stock; "world or file" becomes "the project's test layout"; "test kits'" goes.
- **Composed in place by a way** (`ways-recipes.md`, section 2) like any file: a project with its own ways edits its `design` file to fix its rung, or a way names another, as Hotfix's might.

### 4.3 What moves to fabrig's words

fabrig's overlay gains what the stock drops, so fabrig's goals read the same rules as today ([stock-charters.md](stock-charters.md), section 5): status entries and their tags as `DESIGN.md`, section 2 says, **Design departs**, test worlds. Its contracts document moves to `.goal/contracts.md` like every project's (10, question 4).

## 5. A goal's own plan

### 5.1 `.goal/`

- **A directory at the root of a goal's branch,** g, written by the goal's own delivery, which takes Markdown as it does today (`temper/crates/temper-worker-domain-checkout/src/hold/design.rs`).
- **Read where design is read:** each change starts from g's tip, so its checkout holds `.goal/`, and a spec cites it by path and heading as it cites any design.
- **Never in what lands:** before the closer lands g, code commits the removal of `.goal/` on g, "Take out the goal's plan". temper's own check before any landing into the base refuses a tree holding `.goal/`, naming its files, as it refuses a broken `.temper/` ([ways-temper-dir.md](ways-temper-dir.md), section 6).
- **Kept with the landing:** the closer writes `.goal/`'s files into the landing pull request's description, after its own words. Past the forge's bound, it writes the files' names and the commit that last held them, which the pull request's head keeps.
- **Shown while it runs:** the goal's page reads `.goal/` at g's tip. How it draws it is the next pass's.

### 5.2 Why code, not words

- Today the designer is told to cut "a last change before you park, after every other, that deletes" the contracts document. That costs an implementer's run per goal, and it fails when the model forgets. Taking the plan out is a mechanism, so code does it ("LLM for judgement, code for mechanisms").
- The path is fixed, temper's own, the same in every project, so neither a setting nor the words need to say it.

### 5.3 At every rung

- **Rung 1:** the goal writes nothing to `.goal/`.
- **Rung 2:** the whole design is in `.goal/`.
- **Rungs 3 and 4:** the design that stays goes where the project keeps it, delivered to g like any design and landing with the goal. `.goal/contracts.md` holds only the shapes the goal's changes share.

## 6. Goals and milestones in any project

### 6.1 Where a goal starts from

- **Words, a draft or an issue:** Start goal's draft field becomes optional, and the goal's words are what the person wrote, with a `Draft: ` line only where one is named.
- **A chat's draft names no path:** the snapshot's words become "Implement this draft.", then the draft. Whether to commit it, and where, is the project's convention, read through `design` (4.2): fabrig's guide says `docs/design/draft/`, and a project that says nothing keeps it as the goal's request, in `.goal/draft.md`.
- **An issue's text** as a goal's words, with the issue named for its updates, is [milestones.md](milestones.md)'s "a goal is an issue" carried one step further. It is left for later (section 11).

### 6.2 A milestone

- **Its design is at the project's rung:** rungs 1 and 2 keep it as the chat's draft, held by temper, whose parts each goal's words carry; rungs 3 and 4 land it first, where the project keeps design, as fabrig lands its drafts.
- **Cutting it into goals** is a person's step today, through `temper ctl` ([milestones.md](milestones.md), section 6), which fabrig's drafters do by hand. A gallery entry, Plan and schedule, would give it to an agent: a milestone task whose executor reads the design and starts its goals with their dependencies, as its `delegates` allow ([ways-recipes.md](ways-recipes.md), section 2). It waits for the gallery's core ([ways.execution.md](ways.execution.md), W9).
- **Each goal lands on its own,** by its way, as [milestones.md](milestones.md), section 1 already says, whatever the rung.

### 6.3 fabrig's other words, out of the code

- **A failed test's suite** in a repair's words is the name the project's settings give its suite, beside the existing `report.fuzzy` prefix, or none where it names none.
- **A goal's branch** is recognized by the adoption's prefix, not by the literal `temper`.
- **Draft first's rules** go with Draft first, as Milestone's landing names `into = "base"`, the adopted landing branch whatever its name.

## 7. Plugging temper in

### 7.1 What a project needs

A forge temper serves (GitHub through its App, or Forgejo), a repository temper may write, its landing branch, a model account and a worker with the tools the project's checks need: hono's need Node, Bun and Deno, and vector's Docker for its integration tests ([any-project.notes.md](any-project.notes.md), 2.2). Nothing in the repository: Pair and Hand it over run on the stock's words, and a goal runs at rung 2.

### 7.2 What pays, in order

| Step | What it gives | Without it |
|---|---|---|
| A guide, `AGENTS.md` | how to build, test and format, the project's conventions, its design rung | agents read build files to guess, and design at rung 2 |
| A check at delivery, `.temper/hooks/deliver` ([hooks.md](hooks.md)) | the project's fast checks before a change is offered, repaired at once | CI is the first check, and a red comes back as a repair |
| CI the landing waits for | a landing judged by the project's own checks | the floor asks for a gate of temper's instead |
| Ways of its own, `.temper/ways.toml` | the project's method, from the gallery or written ([ways-temper-dir.md](ways-temper-dir.md), section 7) | the stock ways |
| A higher design rung | the benefits of 3.1 that outlast a goal | rung 2 |

Each step is a change to the repository, and none needs the next.

### 7.3 One change to set up

- **"Set up temper here"** replaces [ways-temper-dir.md](ways-temper-dir.md), section 7's "Add .temper/ with your ways" as a new project's first change, offered at creation or adoption and from "Tell temper how you work" ([to-begin.md](to-begin.md), section 3).
- **Ask temper reads the project's policy on AI first** (7.5), and says what it allows before it proposes anything.
- **Then the repository:** its build files, CI workflows, guides, other tools' agent files among them, its pull request template, and how it keeps documents (`docs/`, decision records, proposals, a changelog).
- **It proposes, part by part,** each explained in the change's words and each dropped by the person if they like: `AGENTS.md` lines the guide lacks, its commands taken from CI, a link where another tool's guide already says them; a `deliver` hook running the project's own fast checks; the ways picked, Hand it over preselected; and the convention it found, as a guide line: decision records at their path and template put the project on rung 3, design rules of its own on rung 4, and nothing found means no line, so rung 2.
- **It lands by the project's rules,** so that the first thing temper writes in a repository is a change a person reads. Every word an agent will read is then in the repository or in the stock's files: "no magic, expecially around prompts" ([ways.md](ways.md), second pass).
- **Judgement is Ask temper's, the change is code's:** reading a repository and choosing is a model's; opening the change and checking `.temper/` before it lands are temper's ([ways-temper-dir.md](ways-temper-dir.md), section 6).

### 7.4 Climbing later

Ask temper proposes the next rung as it sees a project use temper: plans that repeat what a record would hold, a red that a check at delivery would have caught, a way the project keeps adjusting. Each proposal is a change a person decides. It is left for later (section 11).

### 7.5 What a project's own process asks

From the walks in [any-project.notes.md](any-project.notes.md), section 5.

- **Its policy on AI, first:** an AI policy, the contributing guide, or an organisation's page it defers to. Ask temper reads it before anything else and says plainly what it allows: who must own a change, what must be disclosed, whether agents' pull requests are welcome at all. Where the policy bans them, as click's organisation does, temper says so and proposes nothing. Whether to change the policy is the owner's decision.
- **What each change carries:** the repository's pull request template, filled by the implementer's report; a changelog entry or fragment where the guide asks for one; and what else the guide names. The hook checks what is mechanical, as vector's `make check-changelog-fragments` does. The stock implementer's words gain one line: where the repository has a pull request template, the report's body fills it.
- **Whose name is on a commit:** temper's commits are its forge identity's today ([../protocol/git.md](../protocol/git.md), section 5), which can't sign a CLA. Authored as the requester, with temper as committer, a commit is covered by what its requester signed, and names the person a policy holds accountable (question 7).
- **The models named by code:** a landing's words name the models its runs used, in one line code writes, where the project asks for it (question 8). A tool's co-author trailer is left out where the guide forbids it, as vector's does.
- **People's approvals:** a project whose maintainers approve every change runs Hand it over with an owner's approval in its landing ([ways.md](ways.md), 5.4). temper's review is then a first pass, not the gate. A forge's own merge queue already fits: temper lands through it ([../domain/forge/queues.md](../domain/forge/queues.md), section 1).
- **The forge's other actors:** a project's bot that pushes fixes onto a change's branch, as hono's autofix does, and another reviewer's review, as vector's automated review. A bot's commit is taken as a push from outside, and the change's head is checked again (question 9). Another reviewer's comments reach the change's implementer as a repair, to fix or to answer with a reason.
- **No rewritten history:** temper's pushes to a branch a pull request has published add commits and never rewrite them, where the project asks so, as vector's guide does.

## 8. fabrig and dev.fabrig.ai

- **fabrig stays on rung 4,** said by its guide as now, its method in its overlay's words (4.3) until it picks its ways, then in its own `.temper/charters/`.
- **dev.fabrig.ai runs as before:** its lean charter is rebuilt from charter 5 by a script kept outside this repository ([ways.md](ways.md), section 10), so the stock's `design` file and fabrig's added words reach it together, in the release that carries both, by the by-hand step that release names under `## Dev` ([ways.execution.md](ways.execution.md), section 2).
- **The contracts document and its deleting change go** for fabrig too, once `.goal/` lands (10, question 4).

## 9. Upgrades

- **Words, not wires:** the stock's files change in a release, and work started before it keeps the charter revision it started with ([ways-charters.md](ways-charters.md), section 3).
- **A project's own charters** take the `design` file through Ask temper's review after an upgrade ([ways.md](ways.md), section 11). Nothing writes it by itself.
- **`.goal/`** is new on a goal's branch. A goal already running keeps its contracts document and its deleting change, as its words said.
- **The chat's snapshot words** change in the client. An older engine reads them as any words.

## 10. Open questions

1. **Where the convention is said:** the guide, `AGENTS.md`, read through the stock's `design` file (the pick, 4.1); a `design` charter file alone, in `.temper/charters/`, which only a project with its own ways can write; or a value in `ways.toml` naming the rung, which the engine would read.
2. **The stock goal's rung where the guide says nothing:** rung 2, the goal's plan (the pick, 3.3); rung 1, no design at all; or rung 3, a record at a path temper picks.
3. **`.goal/` and its removal:** a fixed path taken out by code (the pick, 5.1); a path the project names; or words telling the designer to delete it, as today's contracts document.
4. **fabrig's contracts document:** into `.goal/contracts.md` like any project's (the pick, 4.3); or kept as fabrig's own, beside the area's design, by its overlay.
5. **Where a plan lives on:** the landing pull request's description (the pick, 5.1); the goal's record in the store; or nowhere, git keeping g's history.
6. **The first change's name and reach:** "Set up temper here", proposing guide lines and a hook beside the ways (the pick, 7.3); or W10's "Add .temper/ with your ways" alone, the guide and hooks left to Ask temper's chat.
7. **Whose name is on temper's commits:** the requester as author and temper as committer, everywhere (the pick, 7.5); the requester only where the project asks, by a setting; or temper's identity, its CLA signed by the deployment's owners.
8. **Naming the models:** a line code writes into a landing's words, where a project's setting asks for it (the pick, 7.5); always, in every project; or left to the guide, for the implementer to write.
9. **A bot's commit on a change's branch:** taken as a push from outside, and the head checked again (the pick, 7.5); refused, the change starting again; or trusted without a check, for bots the project names.

## 11. Later

- **Issues as goals:** a goal started from an issue, its plan and progress written back to it. Both walks want it soon: vector breaks an RFC's work into issues, and hono keeps its design in them ([any-project.notes.md](any-project.notes.md), 3.4).
- **The worker's tools:** what a project's checks need installed, read from `.tool-versions`, a devcontainer or its CI, and said to the deployment's owners at setup (7.1).
- **A piece of work's base** among a repository's branches: a fix starting on click's `stable` rather than `main`.
- **Climbing:** Ask temper proposing the next rung from what it sees (7.4).
- **Plan and schedule:** the gallery entry of 6.2.
- **Documentation other than Markdown:** a goal's own delivery taking the project's documentation formats where rung 3 or 4 keeps records in another.
- **Rungs in the gallery:** an entry per rung, its `design` file shown beside its way.
