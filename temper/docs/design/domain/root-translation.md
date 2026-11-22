# The root's translation

This document holds the root's boundary translation and output assembly, split from [`root.md`](root.md).

## 1. Translation

Small total functions, one per pair of vocabularies, matching
exhaustively with no `_` arm (jig's `domain/root.md`, section 6).

- **Configuration,** `Config`, field by field as
  `../protocol/configuration/engine.md` says: `core` to the core
  whole, with jig's kinds' stock defaults and temper's kinds'
  declarations; `forge` to the forge, its namespace the deployment's id,
  with temper's kinds' stock defaults (`../protocol/configuration/stock.md`
  ); `identities`, each forge's git identity, to the forge. The
  landing rules and the forge's charter are settings of temper's kinds
  (`routing.md`, 6), not members of `Config`. The root holds its configuration to
  its limits as it is made, and a start that breaks one is refused,
  naming it (`../protocol/configuration/README.md`, section 2): one connector;
  the configured workers within the fleet's configured count and name
  bytes (jig's Membership); the deployment's provider other than 0, the
  forge's sign-in; each charter's run within an assignment's bytes; the
  forge's charter one of the core's; the resume limit within a row's
  bytes; each seeded owner's project naming its escalation role; the
  landing rules and the permission mappings within a row's bytes, and
  within the forge's judges' tables.
- **The landing policy:** the deployment's rules and each project's,
  each a `LandingRule` on a pattern of landing branches, whose gates
  every change into a covered branch asks for (forge/change/gates.md, 1). Nothing
  is compiled: the rules are values of temper's kinds, which the forge
  holds as it is told them (`routing.md`, 6) and judges (forge/change/gates.md, 2); the stock
  rules carry two standing requirements on the forge's merge, and no
  rule or policy of jig's changes with a landing rule
  (`../protocol/configuration/settings.md`, 3).
- **Names.** The root makes two names from the forge's own (the forge,
  the repository's id, what in it): a path for authority (`forge`, the
  forge's host, the owner, the name, then `branch` and its segments, or
  `pull` or `issue` and a number; forge/README.md, section 2); and, for a hold,
  three segments, the forge's number, the repository's id and the
  branch, which it reads back for the forge's branches only. A saved
  repository's tag becomes the task's saved resource, its path the
  forge's number and the repository's id (forge/adoption.md), and
  back.
- **The change procedure's decisions** become the core's procedure
  steps:

  | The change decides | The core hears |
  |---|---|
  | finished, merged | its result: the change and its pull request, the merge's id its words (below) |
  | a delegate: producing, a repair (CI, a semantic conflict, a gate's remarks, a person's request on the forge), a resolution, a gate | a batch of one delegate (below); the step waits |
  | ready; a wait | a wait; ready steps it again |
  | a queue repair | the repair started (`routing.md`, 4) and a wait, or, when it cannot start, held for its effects |
  | held: its branch moved or missing, its pull request retargeted or closed | held for drift |
  | held: stalled | held, stalled |
  | held: past its repairs, resolutions or updates | held by its procedure |
  | held: a delegate or an effect failed for good; an approval declined | held, an effect failed |

- **A landed change's result** is a change of the forge's connector and
  kind, its resource the pull request, and its words the merge commit's
  id as text: in lowercase hexadecimal, as a delegate's commits are
  (below), shortened to its first digits, as a short id is, when the
  task's contract bounds its words under the id's 40 or 64. Where the
  bound leaves room, verification words follow the id, whole or not at
  all: `Cleared by TIER run N.`, the repairing run and landing where
  applicable, or `Not cleared: ...` (`after-landing.md`, 6). These words
  precede the optional pull request line so they take room first within
  a task document's text window. Next comes the pull request in the
  forge's words (`../client/forge/family.md`, 1) and its address on the
  forge (`../client/forge/served.md`, 2), `pull request #7 of acme/app on
  github.com: https://github.com/acme/app/pull/7`, or the words alone
  where the forge gives no address or it is past its bound; the line
  cut whole, never in part, where the bound leaves no room for it.
  After the pull request line, where the bound leaves room for it whole,
  a line says `Landed: pull request #7 merged into main as 0123456789ab.`:
  the pull request's number, the branch it landed into and the merge
  commit's first twelve digits. The run that asked hears which pull
  request landed, since jig gives a parent the result's words and never
  its resource. The reported evidence of what the landing passed
  (`forge/change/README.md`, 2) follows, fitted to what the bound leaves. A client
  reads the id from the first line. A result's
  words are text that summarises it (jig's `domain/tasks/README.md`, section 3,
  and `jig/docs/design/client/domain/contract/documents.md`, 3), never the commit's bytes; the
  whole id stays the change's and its pull request's (`forge/change/README.md`, 2).
  The short id keeps any bound the task's creator chose: a bound under
  the id's digits is not refused, and the result never passes it.
- **A change's delegates,** in the core's terms: an agent task under the
  forge's charter for producing, repairs, resolutions, and agents' and
  checks' gates (the charter the change was made with, kept on it, which
  its procedure's delegation grants; `../protocol/configuration/settings.md`,
  2); a person task for the project's maintainers for a person's
  gate. Its parameters name the repository, its base and the change's
  branch, and a resolution's base commit or a gate's number and head,
  each commit as text: its id in lowercase hexadecimal, 40 digits for a
  SHA-1 id and 64 for a SHA-256 one, as the forge's ids are written
  (skein's `forge/forgejo.md`, 1.1), since a task's parameters are text (jig's
  `client/domain/contract/documents.md`, 3); its contract is a change, or a
  gate's verdict; its authority is the change's, its budget what the
  change has left, with no delegation of its own.
- **A collaborator's permission** on the forge (read, write, admin)
  becomes the role its project's configured mapping names, as a
  repository is adopted (forge/adoption.md).
- **A repository as a notes scope** is jig's resource scope on the
  forge's connector, its pattern the repository's path; the root
  translates nothing of it (jig's `domain/engine/briefs.md`, section 2).
- **The hub's link vocabulary** to the fleet's and back: a hello with its
  bounds, the worker's stop bound among them (worker/README.md, section 2), a
  run's phases, activity and counts. A run's live piece goes to the
  core as the run's text instead (jig's `domain/engine/views.md`):
  one of a kind other than text, or past the offsets the core keeps, is
  dropped and counted. A message relayed to a run
  keeps its kind in its words: a result, a proposal decided, a person
  task stalled and an effect not made each lead with what it is and its
  source's number.

## 2. Putting outputs together

### 2.1 An assignment

The core's part (the run, its attempt and conversation, its ask for the
run's live content, its charter, its brief's core sections, its start's
messages, its transcript's current window, the calls committed since its
last turn; jig's `domain/engine/executors.md`, 1) and the forge's (the
workspace, the brief's forge sections), put together while the task is
preparing, held until its claim commits:

- **The repositories:** for a change's delegate, the change's
  repository; for a run whose charter is an asking one, naming the asking family
  and no family but settings beside it, none, as
  for the status agent's (forge/attention.md, 3; `agent/ask-temper.md`); otherwise the
  project's home repository, those its spec names and those it saved
  work in; and, for every run but those with none, the project's context
  repositories, read only. Each item is the forge's: where it
  starts, and the branch it pushes to, the change's branch for a
  change's delegate and the task's own `r` branch for any other run
  (forge/branches.md; worker/hosted-runs.md, 1.1), pushed only when the task
  holds that branch, or the change it works for does, and its authority
  grants the push; otherwise read only.
- **The workstream:** the change's task for its delegates, so a change's
  producing, repairs and resolutions find its checkout cached; the task's
  own otherwise.
- **Saving:** the task's saved-work branch (forge/branches.md) for a
  run that may write, so the unfinished work it leaves is saved and the
  next run starts from it (jig's `domain/engine/executors.md`, 4; worker/hosted-runs.md,
  1.2).
- **Grants:** the LLM accounts its charter's models need, the core's;
  and each repository's forge's git identity, one of temper's
  application credentials, by number and generation (jig's
  `domain/engine/README.md`, 6.1). Their values come from the forge's records
  in jig's secrets' store, generation 1 at each start and the next at
  each change of value, each announced before the restart
  (`../protocol/credentials.md`, section 4). The forge's API token or App
  key, and a Forgejo forge's webhook secret, stay in the protocol layer,
  and are never granted.
- **The writes it claims:** the branches it pushes, the core's writer
  slots in the claim's commit and the forge's after it (`routing.md`, 3).

### 2.2 The forge's brief sections

- **For a change's delegate,** once the change's pull request is open,
  a pull section at the change's last head, required, naming its pull
  request before its contents (forge/change/gates.md, 1 and forge/change/repairs.md): against its base
  for a producer, gate or repair, with what landed in the base and the
  files in conflict for a resolution, and with what its clean update
  merged in for a semantic repair. A repair also has a required reason
  section: CI's failures, with the failed step and its output's end, for
  a CI or semantic repair; reviews' remarks for a gate's repair; the
  forge's reason for a merge-queue ejection. A read of one of a repair's
  two that fails becomes words in its place, the other standing for it
  (forge/change/repairs.md). Each kind has its budget (`ci`, `reviews`, `pull`).
- **For a task holding no run branch in its project's adopted home**
  (forge/branches.md), one section, required: why its runs read the
  home without pushing, bounded by its words. A goal's design writer
  (forge/branches.md, 3) needs no run branch to push, and its brief has none.
- **For a goal's design writer** (forge/branches.md, 3), one line beside that
  section's place: that `deliver` pushes the checkout's Markdown
  straight to the goal branch, merging its moved tip first, and that
  code goes through `change`. No other run's brief has it.
- **For each document its charter names** (jig's `domain/engine/settings.md`,
  9; `../protocol/configuration/settings.md`, 6), one section, optional,
  titled by the repository, the path and the heading: the file, or its
  part, at the branch's head when the run is prepared, within the
  `document` budget, or why it could not be read.
- **Planned with the core's.** The brief child places, cuts and orders
  them with the core's sections (jig's `domain/engine/briefs.md`, section 1);
  the forge gathers each and keeps it, cut to its allotment, and the root
  takes it once the brief is complete. A read refused at the forge
  client's admission (`Busy`) is retained and retried after the client
  reclaims, within the gather's deadline. A read sent but unanswered
  at that deadline leaves its required section late: the brief fails,
  counting as an infrastructure failure, and no attempt is spent. A section whose bytes
  the forge no longer has fails the brief too, counting as permanent
  (jig's `domain/engine/briefs.md`, 1.1).

Required forge sections take room from optional core sections alone, never
from a charter's document sections. The small root's eight section slots
hold three required core sections, two documents and three forge sections
(2026-11-04).

A charter document that names another connector or no forge branch is
answered missing at its gather without asking the forge. Its section is
titled `Document PATH` and says its read failed (2026-11-04).

### 2.3 A call's answer

The core's part (its status, an effect's entry, why it was refused) and
the forge's (what an effect made, a read's answer, the forge's refusal),
which the forge keeps under the call's key and the root writes beside
the call's record in the same commit. Asked again, the answer is put
together again from both. A run's read is answered through the door.

Read refusals also emit volatile root facts naming task, attempt, tool,
refusing layer and reason (`agent/agents.md`). Authority findings survive as
`ToolDenied`; local routing errors and forge errors retain their layer in
the fact. Call admission distinguishes call-record room from archive-read
room while keeping the host's busy response. The service drains the new
facts into operator notes as note room returns; a full diagnostic sink
changes no tool decision. The root's bounded fact queue counts overflow,
and no diagnostic variant enters the store or client wire.

Final forge-read rendering checks the settled tool-name and body bytes against
`core.call_answer_bytes`, then those bytes plus any retained typed answer
against `root.journal.transcript_bytes`. An exceeded bound returns an error
within both settled-answer limits, naming the bound, measured size and a
narrower read route (`agent/agents.md`). The root retains the original tool
identity for its `host-render` refusal fact even as it removes the flight.
A rendering size refusal is terminal; forge-client admission busy retains
its bounded retry path.


### 2.4 What a run left

- **Pushed heads,** from the answer's parts, kept until the core says the
  attempt has ended; then given to the forge against the branches the
  attempt was granted, which it reads afresh before their writer slot
  frees (forge/change/README.md, 2).
- **Saved work:** the repositories saved, as the task's saved resources,
  which its next run starts from.
- **A failed preparation** names its repository and why: what the forge
  lacks (the repository, the branch, the commit), that it refused the
  identity, or the head that did not merge into a base with heads merged
  in (`../protocol/channel.md`, section 5; worker/checkouts.md). The
  root reports it as a fact of the attempt: its task, its attempt, the
  repository's tag, the reason; a conflict it gives the forge too, for
  the change whose producer the run is (forge/change/questions.md, 1). Whether the
  task tries again follows the answer's class, as for any failure
  (worker/hosted-runs.md, 1.3).
- **Past their bounds,** an answer's parts fail its run as invalid,
  whatever the answer says.

## 3. Open questions

- **The forge's work done in the root:** choosing a run's repositories,
  planning the forge's brief sections from a change's state, making a
  change's delegates (`root-translation.md`, sections 1 and 2). jig's connectors gather their
  workspace items and plan their sections themselves (jig's
  `domain/connectors.md`, sections 9 and 10), but temper's forge names no
  type of jig's tasks. Default: translation in the root.

- **A failed preparation's reason on the task's page** (`root-translation.md`, 2.4): jig's core
  keeps an attempt's end as a class only; temper would ask jig to keep
  its detail. Default: the fact alone.
