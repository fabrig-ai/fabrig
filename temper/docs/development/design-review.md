# temper's design: content review

Report, 2026-11-11. A content pass after the design-layout overhaul: findings,
not decisions about the design. Read DESIGN.md, the citation mover's README,
temper's root and domain directory guides, every directory guide's introduction
and one-page/map material, and every Open questions and Later section outside
drafts. The required question windows were 61 sections, 257,745 bytes; the
mechanical Later search also selected protocol/setup/github's already-designed
later-installation flow, which was read, not counted as a question section.
Overlaps were searched across the 141 non-draft design Markdown documents,
using repeated headings and bold terms, exact normalized paragraphs, and
references in the candidate sections. Every question item's label/key symbols
were searched for answers; candidate answers and overlapping sections were read,
not the remaining bodies whole. The report form was read from review-ideas.
Drafts, status inventories and the other four trees were not assessed.

## 1. Account and reading limits

| Kind | Settled by a move/reference | Left for its owner |
|---|---:|---:|
| Overlap | 0 | 6 |
| Answered question | 0 | 12 |
| Misplaced question | 147 | 0 |
| Duplicated question | 1 | 3 |
| Stale question | 0 | 5 |
| Total | 148 | 26 |

D1 groups the repeated comparison-checklist pattern, with its instances listed
below; each other left entry is counted once. A current default with a genuine
future alternative was not treated as an answered question merely for having a
default. Generated CLI references, forwarding summaries with citations, and
contracts viewed from different layers were not treated as independent designs.
In particular, the worker assignment's list of start forms is not a second
checkout algorithm, nor is a protocol's encoding of a domain record a second
record design.

**Read only in part.** Except for the root and domain guides, long directory
READMEs were read for their introduction, one-page/map material and question
sections, with additional candidate sections where needed. Other design bodies
were searched and only candidate sections read. Thus this is not a claim to have
read every contract end to end, nor to have proved the absence of all overlaps.
The question sections were read whole, including the long browser, setup,
configuration, agent, forge and testing lists. Question search did not inspect
implementation, execute a provider flow or decide whether an after-landing check
has run. Where one document reports a walk and another still requests it, both
accounts are given for the owner to settle.

**Moves.** All qualifying moves were unnumbered list items, the brief's by-hand
case. Their words, defaults, bounds and deferrals are preserved; relative paths
and local section antecedents are repaired. Overview entries now cite the owning
document's questions instead of designing them again. No numbered section was
moved, so there was no move.py section application or By hand output to repair.
The one ledger is [temper-content-pass.txt](../../../tools/citations/maps/temper-content-pass.txt):
its commented lines record the unnumbered moves, rather than pretending to be
executable section directives. Incoming citations of the original question
sections remain valid and lead through those pointers. No status entry, draft,
code or help page was changed. Every new question section belongs to an existing
document with an existing status; deferred questions add no implementation gap.

The root's meaning, wire/stored forms and the competing texts below stay for
their designers. No tests were added: the citation and size gates are the proof
for this docs-only change.

## 2. Findings left

The owner is the document whose designer should settle the concept. Suggestions
below are not edits to that document's contract.

### 2.1 O1: Citation conventions twice

Kind: overlap. Owner: `temper/docs/design/README.md`.

`temper/docs/design/README.md`, 7 and `temper/docs/design/domain/README.md`, 3 both give the bare-name, jig-name, smith-name and code-citation conventions, and the shape of a document, in full. The domain guide adds its own forge-fact evidence rule. Suggest keeping the common conventions in the root guide and citing them from the domain guide, retaining only that addition there; this needs a merge of the differing examples, not a section move.

### 2.2 O2: The adoption picker twice

Kind: overlap. Owner: `temper/docs/design/client/forge/repositories.md`.

`temper/docs/design/client/forge/family.md`, 4 and `temper/docs/design/client/forge/repositories.md`, 1 each specify the picker's paging, scroll/search trigger, eight-row threshold, counts, disabled marks, retained picks and Not listed behavior. Suggest keeping the picker contract in repositories and making the form cite it, with only its fields and request handling in family. The lists are differently arranged and interleaved with other form rules, so a whole-section move would not settle them.

### 2.3 O3: The force-push rule and its prepare exception

Kind: overlap. Owner: `temper/docs/design/domain/forge/change/gates.md`.

`temper/docs/design/domain/forge/change/gates.md`, 2 says force-pushing does not exist and no kind moves a branch backwards, while `temper/docs/design/domain/landing.md`, 5.2 explicitly permits a prepare rebase and the preamble of `temper/docs/design/domain/worker/jobs.md` permits the prepare job's leased rewrite. Suggest having gates distinguish a run's delivery from landing prepare and cite that exception, rather than leaving two meanings of the prohibition. The landing text already distinguishes the two; no exception is being added by this report.

### 2.4 O4: Who owns the settling reread

Kind: overlap. Owner: `temper/docs/design/domain/forge/keeping-up.md`.

`temper/docs/design/protocol/setup/README.md`, 6 retells the mergeability/CI settling loop (hinted, doubling to poll, within settle), while `temper/docs/design/domain/forge/keeping-up.md`, 2 specifies it in full, including that the change procedure, not the keep, does those fresh reads. The setup text also speaks of the keep reading the hints. Suggest keeping the algorithm in keeping-up and retaining only box provisioning, delivery coverage and its proof in setup; the different descriptions of its actor need the owner's reconciliation. `temper/docs/design/protocol/forge/README.md`, 4 is a forwarding summary and is not a third finding.

### 2.5 O5: Two contracts for a change task page

Kind: overlap. Owner: `temper/docs/design/client/pages/task.md`.

`temper/docs/design/client/forge/README.md`, 2.1 through `temper/docs/design/client/forge/README.md`, 2.3 describe the task page's phases, panel fields and gate-amendment form, while `temper/docs/design/client/pages/task.md`, 3 gives that same page its six-step rendering and expressly leaves out requirements, heads, contexts and times; `temper/docs/design/client/forge/served.md`, 2 excludes those fields from the delivered panel. Suggest making the task-page document the page contract and distinguishing any future panel design from it in the forge overview. Choosing what remains future, including gate amendment, is content work, not a move.

### 2.6 O6: Whether the worker hosts jobs

Kind: overlap. Owner: `temper/docs/design/domain/worker/README.md`.

`temper/docs/design/domain/worker/README.md`, 1 says "It hosts runs, not jobs", while `temper/docs/design/domain/worker/jobs.md` designs the worker root's job capability and its four kinds in its preamble. Suggest having the worker overview name both capabilities and cite jobs for its contract. These are two domain-layer accounts of the same worker, not a domain/protocol distinction.

### 2.7 A1: The old whole-content comparison

Kind: answered question. Owner: `temper/docs/design/domain/agent/README.md`.

The "A larger whole compare" item in `temper/docs/design/domain/agent/README.md`, 8 still treats paths-only comparison as a change to a settled whole-content answer. The preamble of `temper/docs/design/domain/agent/tools.md` now defines "A compare is by names, with no contents", with counts, containment and bounded path recovery. Suggest retiring that answered premise; retain any separate question about the row bound only if the designer still wants it.

### 2.8 A2: What binary to download

Kind: answered question. Owner: `temper/docs/design/client/web/README.md`.

"A binary to download" in `temper/docs/design/client/web/README.md`, 3 leaves targets, C library, build and publication undecided and says the binary is not shipped. `temper/docs/design/release.md`, 2 chooses the GNU Linux target and glibc floor, and `temper/docs/design/release.md`, 5 specifies the build root, script and publication, with Apple Silicon in its subsection. Suggest replacing the answered item with a release citation, leaving the genuinely deferred targets in `temper/docs/design/release.md`, 7.

### 2.9 A3: The adoption form's place

Kind: answered question. Owner: `temper/docs/design/client/forge/family.md`.

"The form's place" in `temper/docs/design/client/forge/family.md`, 8 defaults to a page of its own linked from Settings and Home. `temper/docs/design/client/forge/family.md`, 3 and `temper/docs/design/client/forge/family.md`, 4 already settle the Settings Repositories dialog, its phone sheet, return address and reload. Suggest removing the answered placement question without resolving the separately retained Home-entry follow-up.

### 2.10 A4: The project's repositories in the picker

Kind: answered question. Owner: `temper/docs/design/client/forge/family.md`.

The item in `temper/docs/design/client/forge/family.md`, 8 says the client has no project repository list and therefore cannot exclude or mark those rows. `temper/docs/design/client/forge/family.md`, 4 already disables "In PROJECT" rows, and `temper/docs/design/client/forge/repositories.md`, 1 gives their rule; `temper/docs/design/client/forge/README.md`, 8 explicitly says the client joins the project's repositories. Suggest retiring the answered absence claim, preserving any distinct future question about filtering at the wire.

### 2.11 A5: Tokens for the person's scripts

Kind: answered question. Owner: `temper/docs/design/protocol/setup/README.md`.

"A token for the person's scripts and agents" in `temper/docs/design/protocol/setup/README.md`, 10 starts "Settled" and specifies the two read/write headers and the prohibition on using admin_token. It cites the design that settles it: `temper/docs/design/protocol/setup/owned.md`, 3, `temper/docs/design/protocol/setup/owned.md`, 5 and `temper/docs/design/protocol/setup/box.md`, 2. Suggest retiring the closed question; no token policy is changed here.

### 2.12 A6: The first project and the later repository

Kind: answered question. Owner: `temper/docs/design/protocol/setup/README.md`.

"After the first repository" in `temper/docs/design/protocol/setup/README.md`, 9 states a complete flow and the runtime project-creation rule, rather than identifying what is deferred. The first repository's creation and separate adoption are already `temper/docs/design/protocol/setup/owned.md`, 7, and the New repository act is specified in the preamble's act table in `temper/docs/design/client/forge/deployment.md`. Suggest removing the settled project/adoption account from Later; if a further generic-form design is intended, name only that remaining work. The owner must separate those parts, not move this mixed item wholesale.

### 2.13 A7: The pinned Claude download walked once

Kind: answered question. Owner: `temper/docs/design/protocol/setup/README.md`.

"Claude Code's download walked" in `temper/docs/design/protocol/setup/README.md`, 9 asks the next walker to try the real pinned fetch once. "What a real box's walk has not shown" in `temper/docs/design/client/pages/README.md`, 3 records that fetch up to the sign-in prompt and its reuse on a second run. Suggest closing that particular once-only request while keeping the unproved paid call, failure, concurrent-fetch and other-platform cases distinct. A retained test is not claimed by either account.

### 2.14 A8: The encoder of delivery evidence

Kind: answered question. Owner: `temper/docs/design/protocol/channel.md`.

"Where a delivery's evidence is made" in `temper/docs/design/protocol/channel.md`, 10 leaves its encoding side conditional on jig's settled calls. `temper/docs/design/protocol/channel.md`, 2 already says it is encoded and decoded by the worker's routing with smith's codec, and `temper/docs/design/protocol/llm.md`, 5 places worker-side result encoding too. Suggest closing the encoding-location question or naming a narrower remaining distinction explicitly.

### 2.15 A9: HEAD after saving

Kind: answered question. Owner: `temper/docs/design/protocol/git.md`.

"HEAD after saving unfinished work" in `temper/docs/design/protocol/git.md`, 10 is now a complete answer: a nonsnapshot commit does not move HEAD, preparation checks it out, and save/delivery snapshots advance it. Those rules are designed in `temper/docs/design/protocol/git.md`, 4.7 and in the preamble of `temper/docs/design/domain/worker/checkouts.md`. Suggest removing the answered question rather than moving its restatement.

### 2.16 A10: Clients that are not a person's

Kind: answered question. Owner: `temper/docs/design/client/README.md`.

`temper/docs/design/client/README.md`, 5.5 answers its own item with temper ctl for a one-shot command and a service party, while `temper/docs/design/client/ctl/README.md`, 1 and `temper/docs/design/client/ctl/README.md`, 7 design those uses. Suggest closing the ctl portion; if "A bot: none" is meant to defer another kind of client, say that separately. No bot design is supplied here.

### 2.17 A11: A push message versus a pull title

Kind: answered question. Owner: `temper/docs/design/domain/worker/checkouts.md`.

"A push's message" in `temper/docs/design/domain/worker/checkouts.md`, 1 asks how the pull request takes the run's title/body and whether a squash keeps them. `temper/docs/design/domain/forge/change/README.md`, 1 already specifies the producer result's explicit title, its cut/default, and the pull's body source separately from the spec. Suggest removing the answered pull-metadata part and leaving only the squash-message question if still undecided. The whole mixed item cannot be settled by moving it.

### 2.18 A12: The reference project's main-only rule

Kind: answered question. Owner: `temper/docs/design/protocol/configuration/reference.md`.

"The reference's project rule in stage one" in `temper/docs/design/protocol/configuration/README.md`, 9 and "The reference's combined project settings" in `temper/docs/design/protocol/setup/README.md`, 9 both prescribe applying project.json after project creation and including the project's rule in the description. The reference document's preamble already requires that rule to replace the stock's every-branch rule (`temper/docs/design/protocol/configuration/reference.md`, "The landing rules" and its project.json account). Suggest keeping a remaining golden/description lag in its status, not as an undecided design question. Status entries themselves were not changed.

### 2.19 S1: The TUI's design location

Kind: stale question. Owner: `temper/docs/design/client/tui/README.md`.

"The TUI's design location" in `temper/docs/design/client/tui/README.md`, 5 still defers moving the design under client/tui, which is where that very document and its page/theme children now live, as `temper/docs/design/README.md`, 6 assigns them. Suggest retiring the completed-location question; its preserved paths are evidence of the completed move, not an unresolved alternative.

### 2.20 S2: Indexes once the panels are served

Kind: stale question. Owner: `temper/docs/design/protocol/store.md`.

"Indexes for the forge's client parts" in `temper/docs/design/protocol/store.md`, 7 defers them until panels are served. `temper/docs/design/client/forge/served.md`, 2 already defines the served, watched change panel, and `temper/docs/design/client/pages/task.md`, 3 consumes it. Suggest restating any real indexing/scaling need without the obsolete not-yet-served prerequisite; this report makes no index or retention choice.

### 2.21 S3: Questions on a change page once task pages are designed

Kind: stale question. Owner: `temper/docs/design/client/forge/README.md`.

"A change's questions on its page" in `temper/docs/design/client/forge/README.md`, 8 leaves drawing them until the web pages for a plan's work are designed. `temper/docs/design/client/pages/task.md`, 3 now designs that change page and its live/ended task plan; `temper/docs/design/client/forge/served.md`, 2 already carries the questions. Suggest naming the actual remaining questions-region design instead of waiting for the page as a whole. It is not a claim that the region is implemented.

### 2.22 S4: Tiers on workers once workers run jobs

Kind: stale question. Owner: `temper/docs/design/domain/after-landing.md`.

The runner/worker choice in `temper/docs/design/domain/after-landing.md`, 13 says workers come "once a worker runs jobs". The worker job capability and four kinds already exist in the design, in the preamble of `temper/docs/design/domain/worker/jobs.md`; no tier kind is in that table. Suggest naming the missing tier-job contract, while preserving the still-open choice of where tiers should run.

### 2.23 S5: Away tied to the last goal

Kind: stale question. Owner: `temper/docs/design/client/pages/home.md`.

"A project's only goal ending while Home is open" in `temper/docs/design/client/pages/home.md`, 2 says Away's Shown reads has_goals too. `temper/docs/design/client/pages/README.md`, 2 now specifies Shown by the project's classes or an on Away row, and `temper/docs/design/client/view.md`, 3 uses that same predicate. Suggest removing that stale Away premise without assuming that the separate bounded-goal-read concern has gone away.

### 2.24 D1: Repeated rendered-comparison requests

Kind: duplicated question. Owner: `temper/docs/design/client/web/README.md`.

The following are independent full after-landing comparison checklists for the same named requests, not just citations to one checklist. Their different state lists and proof limits need to be reconciled before copies can become references. Suggest keeping one browser-comparison obligation per request in the browser document, with page and journey documents stating only their unique proof limits and citing it. Instances:

- Home/two-step/night: `temper/docs/design/client/pages/README.md`, 3, `temper/docs/design/client/web/README.md`, 3, `temper/docs/design/client/web/journeys.md`, 10.
- Chat terminal: `temper/docs/design/client/pages/chat/terminal.md`, 2, `temper/docs/design/client/web/README.md`, 3, `temper/docs/design/client/web/journeys.md`, 10.
- Models: `temper/docs/design/client/pages/settings.md`, 3, `temper/docs/design/client/web/README.md`, 3, `temper/docs/design/client/web/journeys.md`, 10.
- Connect flows: `temper/docs/design/client/forge/README.md`, 8, `temper/docs/design/client/pages/README.md`, 3, `temper/docs/design/client/web/README.md`, 3, `temper/docs/design/client/web/journeys.md`, 10.
- Chat heading: `temper/docs/design/client/pages/chat/README.md`, 3 and `temper/docs/design/client/web/journeys.md`, 10.
- Flow drawing and draft-to-goal: `temper/docs/design/client/pages/chat/document.md`, 2, `temper/docs/design/client/web/README.md`, 3, `temper/docs/design/client/web/journeys.md`, 10.

### 2.25 D2: Terminal approval at a public name

Kind: duplicated question. Owner: `temper/docs/design/client/README.md`.

`temper/docs/design/client/README.md`, 5.8, `temper/docs/design/client/web/README.md`, 3 and `temper/docs/design/protocol/configuration/README.md`, 9 each defer the public-name/Google terminal approval flow; the first two repeat the typed code, request origin/age and remotely ended sign-ins. Suggest keeping that future client-flow question with `temper/docs/design/client/README.md`, 4.1 and citing it from the browser/configuration questions, preserving the distinct init loopback restriction. That restriction and the differing detail make this a merge rather than a blind move.

### 2.26 D3: The forge OAuth flow described as two open questions

Kind: duplicated question. Owner: `temper/docs/design/protocol/credentials.md`.

The forge-as-people's-provider items in `temper/docs/design/client/README.md`, 5.8 and `temper/docs/design/protocol/credentials.md`, 8 both describe temper-owned OAuth routes and the same server-side Mint flow. The client item additionally suggests a terminal device-code flow; credentials additionally owns the OAuth registration's secret storage. Suggest keeping the prospective server/Mint contract in credentials, with the client question limited to its client-specific continuation and a citation. This concerns repeated protocol design inside the client question, not the legitimate distinction between UI behavior and credential storage.

## 3. Moves done

Each subsection gives its exact item label and ledger item line. A line names
the original numbered question section and the destination question section;
only the unnumbered item moved, not either numbered section. The duplicate
failed-link item keeps the existing client-root wording instead.

### 3.1 Proposal-form diagnostics

Map item 1: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/tools.md`, 3.

### 3.2 A charter where the process has no local agent

Map item 2: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/tools.md`, 3.

### 3.3 A collaborator's login in `read_collaborators`

Map item 3: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/tools.md`, 3.

### 3.4 `read_permission`'s login held to the settings family

Map item 4: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/tools.md`, 3.

### 3.5 Which host tools a charter offers

Map item 5: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/tools.md`, 3.

### 3.6 Who may name a plan's members of `change`

Map item 6: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/tools.md`, 3.

### 3.7 A change's foremen's LLM choice

Map item 7: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/tools.md`, 3.

### 3.8 A cancel's grace

Map item 8: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/tools.md`, 3.

### 3.9 A change produced by a job of any kind

Map item 9: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/tools.md`, 3.

### 3.10 A permission for its own requester alone

Map item 10: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.11 Constraining models in settings reads and proposals

Map item 11: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.12 The person's forge identity in a chat's brief,

Map item 12: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.13 A person known by a login alone

Map item 13: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.14 What sizes temper's own way

Map item 14: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.15 QA in temper's own way

Map item 15: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.16 Ask temper woken by a landing

Map item 16: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.17 A goal's question kept with its chat

Map item 17: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.18 Who else writes a question's line form

Map item 18: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.19 A direct change result's rendering room

Map item 19: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/charter.md`, 6.

### 3.20 A `see` answer by its worker's reply bound

Map item 20: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/ask-temper.md`, 5.

### 3.21 A document moved within one call

Map item 21: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/ask-temper.md`, 5.

### 3.22 Parts of a settings proposal across a restart

Map item 22: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/ask-temper.md`, 5.

### 3.23 How the settings' addresses read

Map item 23: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/ask-temper.md`, 5.

### 3.24 A revision's values of temper's kinds through `see`

Map item 24: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/ask-temper.md`, 5.

### 3.25 A project's own policy through `see`

Map item 25: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/ask-temper.md`, 5.

### 3.26 A workflow that runs on no pull request

Map item 26: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/adoption.md`, 8.

### 3.27 What a refused copy leaves

Map item 27: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/adoption.md`, 8.

### 3.28 A merge style per repository,

Map item 28: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/adoption.md`, 8.

### 3.29 A change whose forge is not connected,

Map item 29: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/adoption.md`, 8.

### 3.30 `Forbidden` and `Missing` taken as access lacking

Map item 30: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/adoption.md`, 8.

### 3.31 Signed commits required

Map item 31: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/adoption.md`, 8.

### 3.32 Connecting a forge as another user

Map item 32: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/adoption.md`, 8.

### 3.33 Saved branches outside the task's saved set

Map item 33: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/branches.md`, 4.

### 3.34 The connector's own retries

Map item 34: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/keeping-up.md`, 6.

### 3.35 Which checks are CI on GitHub, without required checks

Map item 35: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/keeping-up.md`, 6.

### 3.36 "The brief on Home" in Settings

Map item 36: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/attention.md`, 7.

### 3.37 A status run's kept row

Map item 37: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/attention.md`, 7.

### 3.38 A stock project's room, shared

Map item 38: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/attention.md`, 7.

### 3.39 What lands on a goal's branch and then on main

Map item 39: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/attention.md`, 7.

### 3.40 What ends the Away switch

Map item 40: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/attention.md`, 7.

### 3.41 The classes per person

Map item 41: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/attention.md`, 7.

### 3.42 What code decides, listed

Map item 42: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/attention.md`, 7.

### 3.43 A queue's throughput

Map item 43: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/queues.md`, 2.

### 3.44 A repair of the branch's own CI at a tip since passed

Map item 44: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/queues.md`, 2.

### 3.45 Several changes in a forge's merge queue

Map item 45: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/queues.md`, 2.

### 3.46 The update before entering a forge's merge queue

Map item 46: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/queues.md`, 2.

### 3.47 Taking a change out of a forge's merge queue

Map item 47: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/queues.md`, 2.

### 3.48 A gate's live run past the stall

Map item 48: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/repairs.md`, 1.

### 3.49 A green head no wake reaches

Map item 49: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/repairs.md`, 1.

### 3.50 A retry before the read

Map item 50: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/repairs.md`, 1.

### 3.51 A hold at gating at a green head

Map item 51: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/repairs.md`, 1.

### 3.52 A repair held for its brief

Map item 52: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/repairs.md`, 1.

### 3.53 A forge out of reach or not connected

Map item 53: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/repairs.md`, 1.

### 3.54 A failing check's repair brief

Map item 54: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/repairs.md`, 1.

### 3.55 What failed on a queue's group

Map item 55: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/repairs.md`, 1.

### 3.56 A refused `message` keeps no record

Map item 56: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/questions.md`, 3.

### 3.57 A passed-up question whose asker ends

Map item 57: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/questions.md`, 3.

### 3.58 A report's head past a merge

Map item 58: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/questions.md`, 3.

### 3.59 A landing branch with another name than `main`

Map item 59: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/stock.md`, 1.

### 3.60 A chat checking the forge before declaring a repository absent

Map item 60: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/stock.md`, 1.

### 3.61 The reference's later runs

Map item 61: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/reference.md`, 1.

### 3.62 The checks' deadline

Map item 62: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/reference.md`, 1.

### 3.63 Who writes the case's workflows

Map item 63: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/reference.md`, 1.

### 3.64 Settings refusal positions

Map item 64: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/settings.md`, 9.

### 3.65 A reference call of 128 KB

Map item 65: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/settings.md`, 9.

### 3.66 Large arguments through a real program

Map item 66: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/settings.md`, 9.

### 3.67 A real provider's list

Map item 67: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/settings.md`, 9.

### 3.68 A Claude-only box on the small profile

Map item 68: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/settings.md`, 9.

### 3.69 A message, a report or a person's spec past what a box takes

Map item 69: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/settings.md`, 9.

### 3.70 A brief past the host part's room

Map item 70: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/settings.md`, 9.

### 3.71 Requirements an application fixes

Map item 71: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/settings.md`, 9.

### 3.72 The real-browser check of watch sizing

Map item 72: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/limits.md`, 1.

### 3.73 A party's fair share of the watch rooms

Map item 73: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/limits.md`, 1.

### 3.74 `views.`'s other names

Map item 74: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/limits.md`, 1.

### 3.75 Profiles' numbers

Map item 75: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/limits.md`, 1.

### 3.76 The rest of an assignment's transcript and of a person's read

Map item 76: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/limits.md`, 1.

### 3.77 A configuration directory moved or copied

Map item 77: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/directories.md`, 2.

### 3.78 Repeated admission during fake-box preparation

Map item 78: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/box.md`, 7.

### 3.79 A box killed outright on macOS

Map item 79: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/box.md`, 7.

### 3.80 A box started in the background of a terminal

Map item 80: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/box.md`, 7.

### 3.81 The terminal after a front killed outright

Map item 81: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/box.md`, 7.

### 3.82 The fake box's retained ports under concurrent stories

Map item 82: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/box.md`, 7.

### 3.83 A fake fetch for the fake box,

Map item 83: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/box.md`, 7.

### 3.84 The other fake peers' refused connections

Map item 84: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/box.md`, 7.

### 3.85 The time of day on the terminal's error lines

Map item 85: `temper/docs/design/protocol/setup/README.md`, 10 → `temper/docs/design/protocol/setup/box.md`, 7.

### 3.86 A start holding the engine's documents to temper's clients

Map item 86: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/init.md`, 4.

### 3.87 The deployment's own UX

Map item 87: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/init.md`, 4.

### 3.88 Flags of their own

Map item 88: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/init.md`, 4.

### 3.89 An owned forge made managed

Map item 89: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/owned.md`, 10.

### 3.90 A lock and an exec in skein's shell,

Map item 90: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/owned.md`, 10.

### 3.91 A copy of a bottle of temper's own

Map item 91: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/owned.md`, 10.

### 3.92 The person's `claude` installed after init

Map item 92: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/owned.md`, 10.

### 3.93 Reading a runner's row

Map item 93: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/owned.md`, 10.

### 3.94 Overrides of `app.ini`

Map item 94: `temper/docs/design/protocol/setup/README.md`, 10 → `temper/docs/design/protocol/setup/owned.md`, 10.

### 3.95 Backups beyond an upgrade's

Map item 95: `temper/docs/design/protocol/setup/README.md`, 10 → `temper/docs/design/protocol/setup/owned.md`, 10.

### 3.96 A story of Reconcile over TLS

Map item 96: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/reconcile.md`, 7.

### 3.97 A secret drawn at a reconcile with no repository,

Map item 97: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/reconcile.md`, 7.

### 3.98 An adopted row's kinds after the forge changes

Map item 98: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/reconcile.md`, 7.

### 3.99 A managed forge whose host list refuses the listener

Map item 99: `temper/docs/design/protocol/setup/README.md`, 10 → `temper/docs/design/protocol/setup/reconcile.md`, 7.

### 3.100 A repository link in a folded forge row

Map item 100: `temper/docs/design/client/forge/README.md`, 8 → `temper/docs/design/client/forge/deployment.md`, 1.

### 3.101 A secret text area for the pasted key

Map item 101: `temper/docs/design/client/forge/README.md`, 8 → `temper/docs/design/client/forge/deployment.md`, 1.

### 3.102 An App of one's own, offered again

Map item 102: `temper/docs/design/client/forge/README.md`, 8 → `temper/docs/design/client/forge/deployment.md`, 1.

### 3.103 A connect's stop told from the one before it

Map item 103: `temper/docs/design/client/forge/README.md`, 8 → `temper/docs/design/client/forge/deployment.md`, 1.

### 3.104 A browser on another machine

Map item 104: `temper/docs/design/client/forge/README.md`, 8 → `temper/docs/design/client/forge/deployment.md`, 1.

### 3.105 Mending a stopped managed or owned forge

Map item 105: `temper/docs/design/client/forge/README.md`, 8 → `temper/docs/design/client/forge/deployment.md`, 1.

### 3.106 The picker's pages kept

Map item 106: `temper/docs/design/client/forge/README.md`, 8 → `temper/docs/design/client/forge/repositories.md`, 5.

### 3.107 A repository's role, health and landing rules on the Repositories
  tab

Map item 107: `temper/docs/design/client/forge/family.md`, 8 → `temper/docs/design/client/forge/repositories.md`, 5.

### 3.108 A connect's end told

Map item 108: `temper/docs/design/client/forge/family.md`, 8 → `temper/docs/design/client/forge/deployment.md`, 1.

### 3.109 A filter row while its project's ways arrive

Map item 109: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.110 A phone's width in a test

Map item 110: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.111 A chat's way to the Closed filter

Map item 111: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.112 The box's pace in the tier

Map item 112: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.113 A wait's CPU in the tier

Map item 113: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.114 The remaining browser cost

Map item 114: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.115 Merged stories after landing

Map item 115: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.116 Reading's test past its run's forty seconds

Map item 116: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.117 A long chat at the person's whole bound

Map item 117: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.118 A public page's plain text in one query

Map item 118: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.119 Every wrapped text fragment

Map item 119: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.120 A public page changing during measurement

Map item 120: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.121 Expected browser errors scoped to an act

Map item 121: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.122 A compile cache across the tier's tests

Map item 122: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/web/testing.md`, 7.

### 3.123 Ask temper before M3b

Map item 123: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/ask-temper.md`, 5.

### 3.124 Ask temper changing the catalogue

Map item 124: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.125 A real model asking its person

Map item 125: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.126 A real model revising a proposal

Map item 126: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.127 A goal as the designer

Map item 127: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/agents.md`, 5.

### 3.128 A real chat choosing `change`

Map item 128: `temper/docs/design/domain/agent/README.md`, 8 → `temper/docs/design/domain/agent/tools.md`, 3.

### 3.129 A closer's stops on a goal's path

Map item 129: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/branch-path.md`, 1.

### 3.130 A close-out's own charter

Map item 130: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/branch-path.md`, 1.

### 3.131 Project-scoped rules on a real forge

Map item 131: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/gates.md`, 4.

### 3.132 A clean update's other words

Map item 132: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/repairs.md`, 1.

### 3.133 A resource refused as forbidden

Map item 133: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/keeping-up.md`, 6.

### 3.134 A forge's run on a head replaced

Map item 134: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/README.md`, 7.

### 3.135 An early update against the bound

Map item 135: `temper/docs/design/domain/forge/README.md`, 11 → `temper/docs/design/domain/forge/change/README.md`, 7.

### 3.136 Ask temper's real-model walk

Map item 136: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/configuration/stock.md`, 1.

### 3.137 A page refused busy

Map item 137: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/client/pages/README.md`, 3.

### 3.138 A kept GitHub forge whose base ends in `/`

Map item 138: `temper/docs/design/protocol/configuration/README.md`, 9 → `temper/docs/design/protocol/forge/engine.md`, 9.

### 3.139 Four agents on a laptop

Map item 139: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/init.md`, 4.

### 3.140 Discovering models per account

Map item 140: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/init.md`, 4.

### 3.141 The LLM providers out of the document,

Map item 141: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/init.md`, 4.

### 3.142 Good defaults for a person's real work

Map item 142: `temper/docs/design/protocol/setup/README.md`, 10 → `temper/docs/design/protocol/setup/init.md`, 4.

### 3.143 A task's text past what the clients receive

Map item 143: `temper/docs/design/protocol/setup/README.md`, 10 → `temper/docs/design/protocol/setup/init.md`, 4.

### 3.144 A handover made by a Forgejo's own restart

Map item 144: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/owned.md`, 10.

### 3.145 A launchd agent for the box

Map item 145: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/protocol/setup/box.md`, 7.

### 3.146 Two deployments on one managed forge

Map item 146: `temper/docs/design/protocol/setup/README.md`, 10 → `temper/docs/design/protocol/setup/reconcile.md`, 7.

### 3.147 A Forgejo's own links at its `web`

Map item 147: `temper/docs/design/protocol/setup/README.md`, 9 → `temper/docs/design/client/forge/served.md`, 3.

### 3.148 A failed link with a live cookie

Map item 148: `temper/docs/design/client/web/README.md`, 3 → `temper/docs/design/client/README.md`, 5.8. Existing destination item retained; source now cites it.

## 4. Owner index for what is left

- `temper/docs/design/README.md`: O1.
- `temper/docs/design/client/README.md`: A10, D2.
- `temper/docs/design/client/forge/README.md`: S3.
- `temper/docs/design/client/forge/family.md`: A3, A4.
- `temper/docs/design/client/forge/repositories.md`: O2.
- `temper/docs/design/client/pages/home.md`: S5.
- `temper/docs/design/client/pages/task.md`: O5.
- `temper/docs/design/client/tui/README.md`: S1.
- `temper/docs/design/client/web/README.md`: A2, D1.
- `temper/docs/design/domain/after-landing.md`: S4.
- `temper/docs/design/domain/agent/README.md`: A1.
- `temper/docs/design/domain/forge/change/gates.md`: O3.
- `temper/docs/design/domain/forge/keeping-up.md`: O4.
- `temper/docs/design/domain/worker/README.md`: O6.
- `temper/docs/design/domain/worker/checkouts.md`: A11.
- `temper/docs/design/protocol/channel.md`: A8.
- `temper/docs/design/protocol/configuration/reference.md`: A12.
- `temper/docs/design/protocol/credentials.md`: D3.
- `temper/docs/design/protocol/git.md`: A9.
- `temper/docs/design/protocol/setup/README.md`: A5, A6, A7.
- `temper/docs/design/protocol/store.md`: S2.
