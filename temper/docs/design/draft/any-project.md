# Any project: temper without fabrig's way

Draft, 2026-11-27, from a session with the user that day. temper is being made "more 'consumable' by any project": [ways.md](ways.md) and [pair.md](pair.md) move configuration and ownership into a repository's `.temper/`, and the user is starting on how a person meets it. What was still missing is "how ways like 'goal' and 'milestone' (or really any complex way the users may want, possibly fully custom, see the ways-gallery.md for reference) would work for a random project". Two questions: how much the stock is "'biased' towards fabrig's own way of developing, in particular the docs/design approach, with its drafts, .execution.md's, .status.md's"; and "what would a random project do to take advantage of temper's capabilities? how would a new project just 'plug' temper into it?". The user's direction: "encourage 'design as contract' but not quite require it (and expecially not require a certain definite form of it, like docs/design) and perhaps let it live more in a 'spectrum', where you can still take advanage of temper in your own way". This draft surveys where fabrig's way is built in (section 2), lays design out as a spectrum (3), splits who says what (4), gives a goal a plan of its own that never lands (5), carries goals and milestones to any project (6), and says how a project plugs temper in (7). On 2026-11-28 the user kept this first pass to the rungs' basics and mechanics: temper agnostic of any one form, and fabrig "its first concrete consumer, at rung 4, with fabrig's own tweaks living in its .temper/ (eventually, when the ways work completes) and in its AGENTS.md (and related documents)" (section 8). Proposing a higher rung to a project is left out. Its one review, on Codex the same day (`any-project.notes.md`, section 1), found the mechanisms of `.goal/`, the charter set and the goal start underspecified, and the execution's order unsafe; this pass settles them. It changes the stock charters ([stock-charters.md](stock-charters.md)), the chat's goal start, and [ways-temper-dir.md](ways-temper-dir.md), section 7's first change.

## 1. In one page

- **Nothing fails without fabrig's way, but the stock brings it along** (section 2). A project with no `docs/design/` can start a goal from words. The stock goal is then told to write "documents that outlast the goal", status entries and a contracts document into a repository that never asked for them, and the web's goal start commits a chat's draft at `docs/design/draft/`. A few words of fabrig's are fixed in code: `focused` and `fuzzy`, `main` and `temper`.
- **Design as contract, on a spectrum of four rungs** (section 3): the request alone; a plan of the goal's own; a record that stays; a design the code follows. Each rung keeps more and costs more. temper works on all four, and its stock goal stands on the second, so every goal gets the plan's benefits and no project gets files it didn't ask for.
- **The project's guide says its convention; temper's words say the method** (section 4). Where design lives and how it is kept is the project's, for every agent and person, so it belongs in `AGENTS.md`, where fabrig already says "the design is the contract". What a goal does with it is temper's, in a new stock charter file, `design`, that defers to the guide. The stock's words lose DESIGN.md's model. The rung is no setting.
- **fabrig is the first consumer, on the fourth rung** (section 8). temper knows nothing of its form. Its convention is its guide's, `AGENTS.md` and `DESIGN.md`, and its method its own charters' words: its overlay today, its `.temper/` once the ways work lands. What it needs is what any project on rung 4 would write. For fabrig the draft is a refactor: once it is complete, fabrig's development runs unchanged, its goals reading the same rules from their new places, and only its contracts document moving, into `.goal/`.
- **A goal's own plan lives in `.goal/` on its branch** (section 5): its changes read it in their checkouts, code takes it out before the branch lands, and the landing's pull request keeps it. It replaces the contracts document and the last change that deletes it.
- **Goals start from words, a draft or an issue** (section 6). The draft is optional on the web, and a chat's draft is committed only where the project keeps drafts. A milestone's design sits at the project's rung, and a person, or later a planner agent, cuts it into goals.
- **Plugging in is a ladder** (section 7). It starts with a forge, a repository and a model, and nothing written: Pair and Hand it over run as they are. Each rung after that pays on its own: a guide, a check at delivery, CI, ways of the project's own, a higher design rung. Ask temper reads the repository and proposes the setup as one change a person reads, "Set up temper here", in place of "Add .temper/ with your ways".

## 2. Today: where fabrig's way is built in

From a survey of main at `0d918ae9bc`, its five strongest claims checked against the code.

### 2.1 In the code

| Where | What it assumes | Here |
|---|---|---|
| `temper/crates/temper-client-domain-pages/src/pages/chat/sheet.rs`, `words` | a chat's draft becomes a goal's words: "Implement this draft. First commit it, as it stands, at" `docs/design/draft/` and the draft's slug, at the repository's root, where fabrig itself keeps no drafts: its drafts are per tree | names no path (6.1) |
| `temper/crates/temper-client-domain-pages/src/pages/terminal.rs`, the goal command; `temper/crates/temper-client-view-words/src/words.rs`, the draft's hint, path and settings line | the same root path, in the terminal's start and the pages' words | goes with it (6.1) |
| `temper/crates/temper-client-domain-pages/src/pages/start_goal.rs`, `send` | Start goal sends nothing without a draft, and has no field for a request's words | a request or a draft (6.1) |
| `temper/crates/temper-client-domain-pages/src/pages/goal.rs`, `draft_of` | the page reads a goal's last `Draft: ` line, where the closer and late-red filing read its first | one reading (6.1) |
| `temper/crates/temper-worker-domain-checkout/src/hold/design.rs` | a goal's own delivery takes Markdown alone, anywhere in the tree | kept, `.goal/` included (5) |
| `temper/crates/temper-engine-domain-forge-vocabulary/src/change_words.rs`, `report_lines` | a failed test is `fuzzy` where it matches the gate's prefix, and `focused` otherwise, a prefix set or not | the gate's own labels, or none (6.3) |
| `temper/crates/temper-engine-service-stock/src/stock/ways.rs` | Draft first's rules name the branches `main` and `temper` | goes as Draft first becomes Milestone ([ways-recipes.md](ways-recipes.md), section 4) |
| `temper/crates/temper-client-domain-pages/src/pages/goal.rs`, `holds_branch` | a goal's branch ends `temper/ROOT/gROOT`, which also misses a goal under a milestone | the engine's own fact (6.3) |
| `temper/crates/temper-engine-service-stock/src/stock.rs`, template 4 | its name, "goal for a draft" | words for a goal of any start (6.3) |
| `temper/crates/temper-worker-protocol-git/src/invocation.rs` | fetches `refs/temper/counts`, fabrig's published test counts | harmless, since its absence is a path; noted only |
| `temper/crates/temper-qa-tool/` | reads `TREE/docs/design/draft/` and fabrig's triage words | fabrig's own tooling, outside the split (6.3) |

### 2.2 In the stock's words

- **The goal's designer** (`temper/docs/charters/stock/designer.md`): "A lean design, eventually. Every part of the request designed, in documents that outlast the goal"; "a document of the goal's own named `contracts`, beside the area's design and with its own status where the project keeps status"; a last change deleting it; "the status the project keeps, as it will stand once the goal lands"; "Never edit the draft's text"; "test kits'" types among those its note counts. A lasting design is asked of every goal that writes code.
- **The goal's rules** (`temper/docs/charters/stock/goal.md`): "design and status documents" as what it writes; **Not yet** entries ending `[in flight: change N]` "where the project's design keeps status"; **Design departs** "where the project keeps one"; a spec's acceptance sketched as "world or file", skein's test worlds.
- **The coordinator** (`temper/docs/charters/stock/coordinator.md`): a shared shape "going in the contracts document, with a **Design departs** entry".
- **The drafter and the chat** (`temper/crates/temper-engine-service-stock/src/stock/drafter.txt`, `chat.txt`): the drafter's draft is a Markdown note ending `## Open questions`, the chat's a note of a section per topic. Both are generic enough.
- **Hedged, not neutral:** most of these say "where the project keeps", so a model that sees no status files may skip them. What it can't skip is the lasting design: told to write one, it invents a place.

### 2.3 Already the project's

- **fabrig's words** (`temper/docs/charters/fabrig/`, [stock-charters.md](stock-charters.md), section 5): `DESIGN.md`, the focused and fuzzy suites, `tools/tiers/`, budgets, `cargo`, citations, the strict subset. They are an overlay of the deployment's, and move into fabrig's own `.temper/charters/` when it picks its ways ([ways.md](ways.md), section 10).
- **fabrig's hooks** (`.temper/hooks/`, [hooks.md](hooks.md)): `deliver` runs `tools/pre-pr/pre_pr.py`, and `close` settles the status tags with `tools/close/close.py`, as `DESIGN.md`, section 2 asks of "its project's `close` command". The status mechanism is already the project's. Only the stock's words still ask for it.

### 2.4 What a project without a design tree meets

A goal started from words through `temper ctl` runs. On the web, Start goal wants a draft, and a chat's draft is committed at fabrig's path. The designer writes lasting Markdown wherever it guesses design belongs, and status files if it thinks the project keeps them. A project with no `AGENTS.md` and no hooks gets no checks before CI, and its agents find the build and test commands by reading. A goal of words alone needs no design tree; its template, authority and funds, and the project's rules and checks, apply as to any goal. The cost is files the project didn't want, and slower, costlier repairs.

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
| 2. The goal's plan | a design the goal writes for itself, in `.goal/` (section 5) | code alone; the plan stays on the goal's record | the stock goal, in any project |
| 3. A record that stays | the goal's design, landed with its code where and as the project keeps records: a decision record, a proposal, a page under `docs/` | code and the record | a project that keeps decision records or proposals |
| 4. A design the code follows | a maintained design that goals read and revise, its gaps tracked, as the project's design rules say | code and the design's revisions | fabrig, with `DESIGN.md`'s drafts, design documents and status files |

- **What each keeps:** rung 1, nothing but code; rung 2, the goal's plan, for the goal's life; rungs 3 and 4, a lasting design where the project keeps it, and in `.goal/` only the shapes the goal's changes share (5.3). Each step up writes more and keeps it longer.
- **No form is temper's:** rung 3 is whatever records the project keeps, rung 4 whatever its design rules say. temper names the rungs to explain them, in pages and in Ask temper's words. Nothing stores them.

### 3.3 Per project and per piece of work

- **A project's usual rung is what its guide says** (section 4): nothing means rung 2 for goals and rung 1 for changes.
- **A way chooses within the convention,** in the words of its charters (`ways-charters.md`, section 2): a hotfix way whose `design` file says "design nothing" runs at rung 1 where the guide lets a small fix go without design, as most do, and never past what the guide requires.
- **Words in a request** asking for less design are direction, as any request's are, within the same limit. Nothing checks them.

### 3.4 Encouraged, not required

- **Seen working:** a goal with a plan shows it on its page, while it runs and after, from its record (5.1). A person sees design as contract pay off before being asked to keep anything.
- **Never a gate:** no rung is checked, and no way refuses work for want of a design.

## 4. Who says what

### 4.1 The rule

- **The project's convention is the guide's:** where design lives, its form, how drafts and status are kept, said once in `AGENTS.md`, for every agent and every person, temper's and other tools' alike. fabrig's guide already does ("The design is the contract", `AGENTS.md`, section 1).
- **temper's method is the charter's:** the work's process within the convention, when it writes, where its own plan goes. It never lowers what the guide requires. It lives in the stock's words, or the project's own charters ([ways-charters.md](ways-charters.md), section 1).
- **No rule in two places** ([ways-charters.md](ways-charters.md), section 1): the stock's words name no project's form, and a guide doesn't tell temper's roles how to run.
- **Nothing an overlay must take back:** an overlay's text follows the stock's and can't remove it ([stock-charters.md](stock-charters.md), section 4), so a stock word that a rung-4 project's words would contradict is a fault. The stock stays silent where a convention decides.

### 4.2 The stock's `design` file

A sixth file in the stock's set ([stock-charters.md](stock-charters.md), section 2). Charter 5's instructions become `common`, `goal`, `design`, then its parts, so its designer and its coordinator read it on every wake. Charter 2 gets none of it. In an overlay, `design.md` is optional, its text after the stock's, so an overlay without one applies as it does today. The stock-set test's lint that refuses a repository path in the stock's text learns temper's own path, `.goal/`. Its words, as a first sketch:

```
# How this project keeps design

- **The project's convention first:** where the repository's guide says where design lives and how it is kept (decision records, a design tree, a page per feature), keep the goal's design that way, in its place and form, with its rules for drafts and status.
- **Where it says nothing,** the design is the goal's own: write it under `.goal/` on g, and deliver it before the specs that cite it; your changes' checkouts read it there. Code leaves `.goal/` out of what lands and keeps it on the goal's record. Where your brief says the goal holds no `.goal/`, keep the plan in your specs and your report note. Write nothing else the request doesn't ask for.
- **Shapes your changes share,** the types, fields and signatures no lasting document needs, go in `.goal/contracts.md` whatever the convention: binding while the goal runs, gone when it lands.
- **A request the convention lets go without design** (a fix, a small change, where it allows one) needs none: its words and your specs are the contract.
- **Where what is built differs** from a design the project keeps, record it as its convention says; otherwise in your report note.
```

- **The other stock files lose DESIGN.md's model:** "documents that outlast the goal" becomes "the design, kept as `design` says"; the contracts document becomes `.goal/contracts.md`, and its deleting change goes; **Not yet**, `[in flight: change N]`, **Design departs** and "the status the project keeps" leave the stock; "world or file" becomes "the project's test layout"; "test kits'" goes.
- **Composed in place by a way** (`ways-recipes.md`, section 2) like any file: a project with its own ways edits its `design` file, or a way names another, as Hotfix's might. It is a file of the goal's charter, not a charter of its own, so it needs no `settings` charter (`ways-charters.md`, section 2).

### 4.3 What fabrig keeps, and where

- **Its convention, in its guide:** `AGENTS.md`, section 1 says the design is the contract, and `DESIGN.md` says how drafts, design documents and status files are kept, with their **Not yet** entries, their `[in flight: change N]` tags and **Design departs**. Read through the stock's `design` file (4.2), they hold for fabrig's goals as they do today. Its overlay already says "Status follows `DESIGN.md`, section 2" (`temper/docs/charters/fabrig/goal.md`).
- **Its method, in its charters' words:** what only temper's roles need, which no other agent or person reads: the suites a spec names, budgets, reds' keys, the citations' maps. It is its overlay today ([stock-charters.md](stock-charters.md), section 5), and its `.temper/charters/` once the ways work lands ([ways.md](ways.md), section 10).
- **What the stock drops and fabrig still needs,** by role and wake. `DESIGN.md` says the layout, the labels and how they settle; it doesn't give a running agent its timing, its specs or its corrections. fabrig's overlay gains these in the same landing as the stock's change, so that its goals read the same rules as before:

| Overlay file | Gains |
|---|---|
| `goal` | once `change` returns its number, deliver the section's **Not yet** entry ending `[in flight: change N]`; each spec names its suite, focused or fuzzy, and sketches its test world or file, setup and checks; the departure rule as it stands, a departure recorded where `DESIGN.md` asks |
| `designer` | a lasting design for every request that writes code, by the lean design's rules, delivered before the specs that cite it; "never edit the draft's text"; the lasting status, the draft's status and its execution file written as they will stand once the goal lands; every public type added or removed counted, test kits' included |
| `coordinator`, on the reasons the stock's parts name today, `first` among them | a corrected shared shape delivered before the answer; on a result, missing detail sent to the lasting design, the contracts or an amended spec, as fits; the correction in the note, with its **Not yet** or departure entry where design and delivery part |

The stock's words drop the same in that landing, so that nothing is said twice.
- **What goes in `.goal/` for fabrig:** only the goal's contracts document, the shapes its changes share that no lasting document needs. Today it sits beside the area's design as a document named `contracts`, with a status file of its own, and the goal's last change deletes both ([design-economy.md](design-economy.md), 4.3). Until 2026-11-09 it lived under a tree's `docs/plan/GOAL/notes/` instead, now retired. In `.goal/contracts.md` it has no status file, and code leaves it out of what lands. The words that delete it, and that amend the deleting change's `after`, go, and [design-economy.md](design-economy.md), 4.3 follows (10, question 4).
- **What stays where it is:** the lasting design in each tree's `docs/design/`, the drafts with their status and execution files, as `DESIGN.md` says; the specs, which are the changes' words in temper's store and never files; and the goal's decisions, in its report note.

## 5. A goal's own plan

### 5.1 `.goal/`

- **A directory at the root of a goal's branch,** g, and nowhere else. A change into the base, a Pair offer or any other direct landing has no `.goal/`: one there is ordinary content.
- **The goal's only where the base has none:** at the goal's start, code records whether its base holds `.goal/`. Where it does, the path is the project's, the goal holds no `.goal/`, and its brief says so (4.2). A base that gains `.goal/` while the goal runs holds its landing, naming the path, and keeps the base's entry.
- **Written by the goal's own delivery,** which takes Markdown as it does today (`temper/crates/temper-worker-domain-checkout/src/hold/design.rs`), there as regular files within bounds, never links. The designer and the coordinator deliver it before they cut or amend a spec that cites it.
- **Read where design is read:** each change starts from g's tip or its dependencies' checked work, so its checkout holds `.goal/`, and a spec cites it by path and heading as it cites any design. A lasting document never cites it.
- **Never in what lands, left on g:** the closer's prepared landing head ([../domain/landing-prepare.md](../domain/landing-prepare.md)) leaves out what the goal added under `.goal/`, so the tree that lands holds `.goal/` exactly as the base does. g keeps it through catch-ups, a refused close and repairs. Each preparation leaves it out again, at its own head, and a check binds to that head: nothing under `.goal/` lands. A head or base that moves is prepared and checked again. A goal whose only diff was its plan lands nothing, and says so.
- **One check with `.temper/`'s:** the check of the tree that lands is the one [ways-temper-dir.md](ways-temper-dir.md), section 6 asks for `.temper/`. Whichever lands first builds it at today's queue and turn, and the other adds to it.
- **Kept on the goal's record:** at the landing, code keeps `.goal/`'s files on the goal's record, from the head that lands, within bounds per file and in all, as long as the goal's record is kept. The landing pull request names the goal and lists the files, never their text, so a project's transient shapes don't become the forge's.
- **Shown while it runs and after:** the goal's page reads `.goal/` at g's tip while it runs, and from its record after. How it draws it is the next pass's.

### 5.2 Why code, not words

- Today the designer is told to cut "a last change before you park, after every other, that deletes" the contracts document. That costs an implementer's run per goal, and it fails when the model forgets. Taking the plan out is a mechanism, so code does it ("LLM for judgement, code for mechanisms").
- The path is fixed, temper's own, the same in every project, so neither a setting nor the words need to say it.

### 5.3 At every rung

- **Rung 1:** the goal writes nothing to `.goal/`, and its page shows no plan.
- **Rung 2:** the whole design is in `.goal/`.
- **Rungs 3 and 4:** the design that stays goes where the project keeps it, delivered to g like any design and landing with the goal. `.goal/contracts.md` holds only the shapes the goal's changes share.

## 6. Goals and milestones in any project

### 6.1 Where a goal starts from

- **A request, a draft, or both:** Start goal gains a request, words of the person's, bounded, and starts with a name and a request or a draft. A draft keeps today's path: a landed file, its proposal and its funding as they are.
- **A chat's draft names no path:** the snapshot is the goal's words, "Implement this draft." and the draft, with the chat's provenance and no `Draft: ` line. Whether to commit it, and where, is the project's convention, read through `design` (4.2). fabrig's says a tree's `docs/design/draft/`, by `DESIGN.md`, where today's fixed path is the root's, which fabrig doesn't use. A project that says nothing keeps it as the request, with no file. The scope, the agreement where there is one, and Always ask go on as today.
- **One reading of provenance:** code writes a goal's `Draft: ` and `Chat: ` lines ahead of its words, and the goal's page, its closer and late-red filing all read the first of each. A line inside a request's own text can't pose as one.
- **An issue's text** as a goal's words, with the issue named for its updates, is [milestones.md](milestones.md)'s "a goal is an issue" carried one step further. It is left for later (section 11).

### 6.2 A milestone

- **Its design is at the project's rung:** rungs 1 and 2 keep it as the chat's draft, held by temper, whose parts each goal's words carry; rungs 3 and 4 land it first, where the project keeps design, as fabrig lands its drafts.
- **Cutting it into goals** is a person's step today, through `temper ctl` ([milestones.md](milestones.md), section 6), which fabrig's drafters do by hand. A gallery entry, Plan and schedule, would give it to an agent: a milestone task whose executor reads the design and starts its goals with their dependencies, as its `delegates` allow ([ways-recipes.md](ways-recipes.md), section 2). It waits for the gallery's core ([ways.execution.md](ways.execution.md), W9).
- **Each goal lands on its own,** by its way, as [milestones.md](milestones.md), section 1 already says, whatever the rung.

### 6.3 fabrig's other words, out of the code

- **A failed test's label** in a repair's words is the gate's own: its report gains two optional labels, for the tests its prefix matches and for the rest, kept with the receipt, so that a retry or a settings change can't relabel a run. An older receipt reads as `fuzzy` and `focused`, and fabrig's gate sets those two words at the rollout. A gate with no labels names a failed test with none.
- **A goal's branch** is told by the engine's fact that the goal holds it, the adoption's prefix, root and task all included, not by reading its path.
- **Template 4's name,** "goal for a draft", becomes words for a goal of any start.
- **Draft first's preset** goes when ways retire it (`ways.execution.md`, W4 and W14). Rules that an earlier "Use this" wrote into a project's settings stay until the project changes them.
- **temper's QA tool** (`temper/crates/temper-qa-tool/`) is fabrig's own tooling and stays outside the split: it reads fabrig's trees and drafts by design.

## 7. Plugging temper in

### 7.1 What a project needs

A forge temper serves (GitHub through its App, or Forgejo), a repository temper may write, its landing branch, a model account and a worker. Nothing in the repository: Pair and Hand it over run on the stock's words, and a goal runs at rung 2.

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
- **Ask temper reads the repository first:** its build files, CI workflows, guides, other tools' agent files among them, and how it keeps documents (`docs/`, decision records, proposals, a changelog).
- **A proposal, then code:** what Ask temper proposes is a selection of edits against a pinned base, which the person reviews. Once they accept it, code opens one ordinary change, by the starter's authority and under the live floor. A base that moved refreshes it against the new tree, keeping the guide's and the hooks' text already there. The ways and the charter files they need are one unit to select. Guide and hook edits land without `ways.toml`, and choosing the stock ways and nothing else writes nothing.
- **It proposes, part by part,** each explained in the change's words and each dropped by the person if they like: `AGENTS.md` lines the guide lacks, its commands taken from CI, a link where another tool's guide already says them; a `deliver` hook running the project's own fast checks; the ways picked, Hand it over preselected; and the convention it found, as a guide line: decision records at their path and template put the project on rung 3, design rules of its own on rung 4, and nothing found means no line, so rung 2.
- **It lands by the project's rules,** so that the first thing temper writes in a repository is a change a person reads. Every word of the project's own that an agent reads is then in the repository; the rest is the harness's, the deployment's lines and the task's ([ways-charters.md](ways-charters.md), section 1): "no magic, expecially around prompts" ([ways.md](ways.md), second pass).
- **In two steps:** W10's copy of picked ways and their charters, with its preview, comes first ([ways.execution.md](ways.execution.md), W10); Ask temper's guide and hook proposals join it once Ask temper can propose repository edits.
- **Judgement is Ask temper's, the change is code's:** reading a repository and choosing is a model's; opening the change and checking `.temper/` before it lands are temper's ([ways-temper-dir.md](ways-temper-dir.md), section 6).

## 8. fabrig and dev.fabrig.ai

- **The first consumer, on rung 4:** fabrig uses temper as any project on that rung would, with nothing in temper's code or stock words that knows its form. Each of its tweaks lives in its own repository: its convention in `AGENTS.md` and `DESIGN.md`, its method in its overlay's words until it picks its ways, then in its `.temper/charters/` (4.3).
- **The test of the split:** whatever fabrig still needs from temper that its guide and its charters can't say is a coupling left in temper, to be named and removed.
- **dev.fabrig.ai runs as before:** its lean charter is rebuilt from charter 5 by a script kept outside this repository ([ways.md](ways.md), section 10), so the stock's `design` file and fabrig's added words reach it together, in the release that carries both, by the by-hand step that release names under `## Dev` ([ways.execution.md](ways.execution.md), section 2). Whichever charter dev's lean template then runs, the step reads it back and measures its room on every kind of wake, on staging before dev.
- **Unchanged, as fabrig sees it:** at the draft's completion fabrig's goals run as they do today, on the same rules, read from its guide and its own words rather than the stock's. The one move is its contracts document, into `.goal/contracts.md`, with its status file and the change that deletes it gone (4.3; 10, question 4).

## 9. Upgrades

- **Words, not wires:** the stock's files change in a release, and work started before it keeps the charter revision it started with ([ways-charters.md](ways-charters.md), section 3).
- **`.goal/`** is new on a goal's branch. A goal already running keeps its contracts document and its deleting change, as its words said.
- **Provenance:** a goal started before the change, with its `Draft: ` line among its words, reads as before.
- **Labels:** a receipt kept before the gate's labels reads as `fuzzy` and `focused` (6.3).
- **The chat's snapshot words** change in the client. An older engine reads them as any words.

## 10. Open questions

**Settled by the user, 2026-11-28,** on the picks below: 1, 2, 3, 4 and 6 ("I think we're ready to go"), and 5 as the review changed it ("ok").

1. **Where the convention is said:** the guide, `AGENTS.md`, read through the stock's `design` file (the pick, 4.1); a `design` charter file alone, in `.temper/charters/`, which only a project with its own ways can write; or a value in `ways.toml` naming the rung, which the engine would read.
2. **The stock goal's rung where the guide says nothing:** rung 2, the goal's plan (the pick, 3.3); rung 1, no design at all; or rung 3, a record at a path temper picks.
3. **`.goal/` and its removal:** a fixed path taken out by code (the pick, 5.1); a path the project names; or words telling the designer to delete it, as today's contracts document.
4. **fabrig's contracts document:** into `.goal/contracts.md` like any project's (the pick, 4.3); or kept as fabrig's own, beside the area's design, by its overlay.
5. **Where a plan lives on:** the goal's record, bounded, the landing pull request naming it (the pick, 5.1, changed by the review); the pull request's description, which the forge caps at 65,536 bytes (`temper/crates/temper-engine-domain-forge/src/domain.rs`, `PULL_BODY_BYTES`) and whose source branch goes after landing; or nowhere, git keeping g's history.
6. **The first change's name and reach:** "Set up temper here", proposing guide lines and a hook beside the ways, W10's copy of ways first (the pick, 7.3); or W10's "Add .temper/ with your ways" alone, the guide and hooks left to Ask temper's chat.

## 11. Later

- **Issues as goals:** a goal started from an issue, its plan and progress written back to it.
- **Plan and schedule:** the gallery entry of 6.2.
- **Documentation other than Markdown:** a goal's own delivery taking the project's documentation formats where rung 3 or 4 keeps records in another.
- **Rungs in the gallery:** an entry per rung, its `design` file shown beside its way.
