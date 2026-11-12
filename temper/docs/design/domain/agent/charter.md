# temper's charter

The core decides a run's charter in its own terms (jig's
`domain/engine/executors.md`, 1); `temper-engine-smith` writes it as smith's
(smith's `domain/run.md`, 3.1) from the root's assignment, and the
engine's routing encodes it (`../../protocol/llm.md`, section 5).

## 1. Instructions and brief

- **Instructions:** the charter's, from the deployment's configuration
  (jig's `protocol/configuration.md`, section 3): a chat's, a
  coordinator's, a reviewer's with a lens, a change's producer's; until
  an amendment gives a task its own (jig's `domain/tasks/changes.md`
  ), whole, where an empty text leaves it none, not its charter's. The
  task's next run carries them; a run already live hears them as the
  amendment's words. A charter is a setting in the store, read from the
  revision the task runs under (jig's `domain/engine/settings.md`, 3),
  temper's stock chat's and coding agent's held in the binary as their
  defaults (`../../protocol/configuration/stock.md`), the goal's agent's
  and the implementer's assembled from the stock's set of charter files
  (below).
- **A set of charter files** (`../../draft/stock-charters.md`, 2) is a
  directory of Markdown files, each named `<file>.md`, of five:
  `common`, `goal`, `designer`, `coordinator` and `implementer`; the
  stock's, `temper/docs/charters/stock/`, holds all five, and an
  overlay's (that draft's section 4) any of them. The assembly makes two
  charters' texts from it:

  | Charter | Its instructions | Its parts, in order |
  |---|---|---|
  | 5, the goal's agent | `common`, then `goal` | `designer`, on `first`; then each section of `coordinator` |
  | 2, the implementer | `common`, then `implementer` | none |

  - **A section** of `coordinator` starts at a line beginning
    `## part, on:`; its reasons are the rest of that line, split at
    commas, each trimmed of spaces, named as jig names a part's reasons
    (jig's `domain/tasks/messages.md`, 6), and its text runs from the
    next line to the next such line or the file's end, other headings
    included. The heading line is no part of the text.
  - **Texts join** with one blank line, every byte as written but the
    line feeds that open and close a file or a section. A file absent,
    or holding nothing but line feeds, gives nothing: no text and, for
    `designer`, no part. A section is a part whatever its text.
  - **Nothing is dropped:** the assembly refuses, naming the file, the
    line and the name: a Markdown file (a name ending `.md`) in the
    directory that is none of the five; text in `coordinator` before its
    first section, line feeds aside; and a section whose reasons are
    empty (none, or an empty name between commas), repeated within its
    heading, or unknown. Other files in the directory, a settings file
    among them, are no part of the set. The stock's set is assembled at
    each start and held by the stock's focused tests, which assemble it
    and list its directory, so a stock the assembly refuses never lands;
    an overlay's is refused at apply (that draft's section 4).
  - **The code:** the step crate `temper-charter-set`, which names
    neither the service nor jig's types, so that the stock module and
    ctl's `settings apply` assemble a set by the same rule. It reads
    texts, never a directory: its caller lists the directory and reads
    the files. Its public surface, which the stock and a settings file's
    overlay items (that draft's section 4) build on:
    - `File`, the five, with `File::name`, its name in a directory (the
      file's name, then `.md`), and `file`,
      which gives a directory entry's file: `Ok(Some(File))` for one of
      the five, `Ok(None)` for a name not ending `.md`, and the refusal
      for any other Markdown name.
    - `Set`, each file's text optional, `&str`.
    - `goal_agent(set, reason)`, where `reason` maps a reason's name to
      the caller's own value or `None` for an unknown name, gives
      `GoalAgent { instructions, parts }`, each `Part { reasons, text }`,
      its `reasons` the mapped values in the heading's order; the
      designer's part names `first` through the same map.
    - `implementer(set)` gives charter 2's instructions, and refuses
      nothing.
    - `Refusal`, one variant for each refusal above, carrying the file,
      the line (counted from 1) and the name where it has them, and
      saying them in its words.
- **The brief:** the sections jig's core gathers (jig's
  `domain/engine/briefs.md`, section 1) and the forge's (../forge/README.md, section 2),
  each rendered as one of smith's titled sections, titled by its kind:
  `Task`, `Lineage`, `Dependencies`, `Plan`, `Earlier attempts`,
  `Calls`, `Waiting`, `Notes`, `Transcript tail`, and the forge's
  `Pull request`, `CI`, `Reviews` and, for each document the charter
  names, `Document` followed by its repository, path and heading
  (`../../protocol/configuration/settings.md`, 6). A section its gather could not fill
  says why: its read failed, timed out, or exceeded its limit. smith adds
  the sections about its own mechanics, and each repository's guide.
  The small profile admits 14,592 body bytes across at most 64 brief
  sections; the forge reviews budget is 4,608 bytes within that total;
  `standard`, which `temper-box init`'s box runs, gathers what its host part
  leaves the largest stock charter, its task section a whole spec
  (`../../protocol/configuration/limits.md`). Assignment and receiving
  fitting (`tools.md`) may shorten those gathered sections to the charter and
  workspace ownership bound; the durable specification stays whole.

## 2. Workspace

The connectors name the repositories, where each starts, which may be
written and the branch a change goes to (jig's `domain/engine/executors.md`, 1;
../worker/README.md, 4.1); the worker prepares them and gives smith the workspace:
each repository's directory and name, whether it may be written, and for
a merge in progress the files in conflict (../worker/checkouts.md). Every
temper workspace is git repositories. A run with no repository, such as
a chat that names none, runs with no workspace (../worker/README.md, 4.1). A run
whose charter is an asking one (`tools.md`) is given no repository, whatever
its project's home and context repositories (`../root.md`, 6.1; `ask-temper.md`), so it
runs with no workspace: no checkout, no file tools, no shell, and smith
says nothing of a checkout (smith's `domain/run.md`, 3.3; through Claude
Code, smith's `protocol/programs.md`, 5.1). A run whose charter names none
of `inspect`, `modify` and `shell` is given no repository either, whatever
its project's repositories, since it could neither read, write nor run
anything in one: the drafter's runs so (`agents.md`, 4), and keep their sub-agents.

## 3. Result contracts

A task's result contract (jig's `domain/tasks/README.md`, section 3) becomes
smith's (smith's `domain/run.md`, 7.1):

| temper's | In smith |
|---|---|
| a report: words, bounded | a report, and a failure, within the same bound |
| a verdict from a closed list, each with the fields it requires and the follow-up tasks it may propose | a verdict for each choice, named by its number, with its fields and its words' bound; follow-ups as items, of the kinds of task the verdict may propose; a failure of at most 1,024 bytes |
| a connector's result: a change landed or a successful no-change conclusion | a change, requiring the fields `title` and `body`, each within the contract's bound, its checks required; a `no_change` verdict with bounded text and no fields or items; a failure within the same bound |
| an approval | a failure only, of at most 1,024 bytes: approvals are people's |

A change's `body` holds the producer's report, what its brief asks it to
report, whole, within the contract's words (jig's `domain/hosts/charters.md`), which
for a change the `change` tool or its procedure makes are `tasks.result_bytes`:
so a report follows the call's payload and is a setting
(`../../protocol/configuration/settings.md`, 7). Smith's outcome holds the `title`, the
`body` and their framing together, at the payload; its `title` and the
contract's other texts stay within 1,024. The run's contract prints
the bound. The coding charter's words say that the report goes there, and that
one past the bound is no question: the run cuts what can be read elsewhere and
says under Left what it cut (`../../protocol/configuration/stock.md`'s charter 2
row).

A rendering of a report may cut what it shows, and the store keeps it whole. A
landing's result to its requester holds the report as far as the contract's
words leave room for its evidence and for the label the requester's run
reads it under, with its cut marked (`../forge/change/README.md`, 1); `read_task`
with `field=result` gives the producer's report whole, in pieces. A document's
card shows what `documents.text_bytes` holds, marked as cut.

The `no_change` verdict becomes jig's successful `NoChange` result. It
retains the explanation, supplies no resource or landing, and ends the
change without a hold or escalation. The requester receives that result.

A change's result names the resource the worker's push and the forge
settled, never one from the LLM's fields. Its explicit `title` and `body`
are preserved independently through jig's typed result and durable codec, each
within the contract's words bound. The change procedure uses the title for
the pull request, without deriving it from the production brief.
The core judges a result again as it commits it, against the contract
and the state as it is now (jig's `domain/tasks/lifecycle.md`, 7).

## 4. Conventions

Checks at `.temper/pre-pr`, guides at `AGENTS.md` (smith's
`domain/run.md`, 3.1 and 8.1).

- **What a delivery checks** is its project's: the host runs the
  repository's `.temper/pre-pr` on the run's warm checkout before anything
  is pushed, at each `deliver` and at a `finish` that delivers, and a
  failing check refuses the delivery, its output's tail the run's feedback.
  A `save` runs none, and is marked unchecked (`tools.md`; `README.md`, section 5).
  The worker pins the landing branch's tip in each change producer's
  checkout as `refs/temper/landing`, and the head a repair's or a
  resolution's delivery replaces in theirs (../worker/checkouts.md), so a
  check can take what the delivery changed against it; fabrig's checks the changed
  packages' lints and the focused tests of those packages and their direct
  dependents, a failure the base has too reported as inherited
  (`../../protocol/configuration/reference.md`; `tools/pre-pr/design.md`).
- **How long a check may take** is its project's too. A run delegated
  for a change, its producer's, a repair's or a resolution's, a goal's
  landing's among them, has the charter's checks' bound (smith's
  `protocol/charter.md`, section 2) set to its project's `check_timeout_ms`
  where the project sets one (`../../protocol/configuration/engine.md`),
  and no bound otherwise, so the agent's `check_timeout` holds. Every
  other run names none. The bound is read when the run's charter is
  written, at each attempt, so a changed setting reaches the next
  attempt. A check that runs out of time runs once more at twice its
  bound (smith's `domain/run.md`, 8.1), so a check that only ran out of
  time never ends the run's work by itself.
- **No CI before a goal's branch.** A pull request into a goal's branch
  runs no CI: a change's delivery checks are all it meets before it lands
  there. The landing's CI into main, the project's full gate, is the
  guard (`../../protocol/configuration/reference.md`).
- **No security boundary:** an agent can change what a check runs
  (smith's `domain/run.md`, 8.1); the landing's CI and its authority's
  requirements guard what lands on main.

## 5. Budget, models, waiting

- **Budget:** the run's turns, spend and time, with its reserve of turns
  and time for finishing (smith's `domain/budget.md`, 1), from its
  charter in configuration, at the task's revision; its spend is what
  its task has left, capped per run, reserved at its claim, in the
  deployment's unit (jig's `domain/hosts/charters.md`).
- **Models:** the main session's and those its sub-agents may use, each
  with its endpoint, window, output and prices (`../../protocol/llm.md`,
  section 3): the task's LLM choice among those its charter allows,
  without one the charter's first whose LLM provider has an account
  usable for the payer (jig's `domain/tasks/README.md`, section 3),
  and the account the run's claim chose for each LLM provider (jig's
  `domain/accounts.md`, 4.3). A chat's new choice answers its person's
  next words, whether its run still listens or has parked (jig's
  `domain/tasks/changes.md`). Their credentials, or a program source's
  name, come to the agent as grants through the worker
  (`../../protocol/credentials.md`, section 3). A run on a program has
  smith's sub-agents, inspect only, each a program session of its own
  (smith's `protocol/programs.md`, 5.10).
  The stock demo is temper's fake model, declared with `gpt-6-luna`'s
  window, 1,048,576 tokens, and an output of 8,192, so it leaves every
  charter the room luna does. At smith's 4/5 threshold, after two
  outputs and less smith's own part, half of the remaining room gives
  the host part and a quarter the offer (smith's `protocol/limits.md`,
  3.9): ⌊(⌊1,048,576 × 4/5⌋ − 2 × 8,192 − own) / 2⌋, about 400,000
  bytes; `temper-service`'s step tests hold the figure at init's agent.
  Every stock charter retains that rendered room, which holds the stock
  tools and the gathered brief with a whole spec. Delivery also enforces
  the separate combined ownership bound, shortening the brief when needed
  (`tools.md`). A message offered a run is at
  most an offer and the agent's tool payload, which the model's output
  bounds (`../../protocol/configuration/settings.md`, 7).
- **Waiting and resuming:** the waiting time is the charter's; a
  coordinator's is zero. A chat resumes its transcript, and a
  coordinator starts fresh from its brief (jig's `domain/engine/executors.md`,
  4). A goal under stock charter 1 also resumes. Its granted `wait` may
  park in Finite wind-down, including the eighth completion after seven
  ordinary completions. The pending plan question and its answer route remain
  live while parked. The answer starts a fresh bounded activation with eight
  turns and one reserved turn; its waking prompt supersedes prior exhaustion.
  The task's monetary spending remains cumulative and each activation is
  funded only from what remains.
- **Its mode is its task's ending** (jig's `domain/tasks/lifecycle.md`, 9;
  `jig/docs/design/domain/hosts/charters.md`, 1), the assignment's `ended_by`, whatever the
  charter: a task its result ends runs smith's `Finite` mode, and one
  its requester ends runs `Open`, granted the wait and neither `finish`
  nor `deliver`. temper's chats are Open (`../../protocol/configuration/README.md`,
  section 2): ordinary text listens for the person's next words. Idle or
  wind-down parks the run, committing its reply with no result and no try
  spent. Close carries the grace's finish request live or in Start; its
  admission puts the Open run into wind-down, so its next text parks.
  Cancellation after the grace keeps its existing meaning; only the
  person's Close, or a cancel, ends the chat. Goals and every delegate
  run Finite and end by their results.

## 6. Open questions

- **A direct change result's rendering room** (`charter.md`, 3): the landing procedure
  reserves 59 bytes for its result's label and sender before fitting its
  report. A task made directly by `delegate` with a Change contract has
  no such reserve, so its result at the contract's bound can be refused
  by a live requester's receiving bound. Default: keep the durable result
  whole and read it through `read_task`; the rendering allowance remains
  jig's `domain/hosts/README.md`, section 10's question.
