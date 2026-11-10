# The goal as designer

Draft, 2026-11-06, from a session with the user that day. It revises
`reference-code-production.md`, sections 1 to 5 and 12 to 14, and wins where they
differ until its plans fold it in. The comparisons with the pipeline we develop
with today matched that pipeline's prompts by keeping its plan in files: a
producer wrote them and the goal's agent copied them into changes
(`dogfood-hacks.md`, 2.10). That proved the engine, and it isn't the shape the
user meant. The user: "Temper's engine (jig) is supposed to handle all
mechanics ... I'd expect the top-level designer (presumeably the one running
the top-level goal task) to use jig-provided tools to create the task tree
himself, and when the tree is created it should mostly run itself". And what
matching means: "the same assignment/request to temper (a goal) results in a
similar execution, i.e. mostly same roles in shape/number/sequence/timing with
matching prompts and model/effort". Reasons: `notes.md`, section 7.

## 1. In one page

- **One session designs.** The goal's first run surveys, writes the design,
  and makes the tree with `change`. No plan file, and no agent copying one
  (section 3).
- **The tree then runs itself.** jig orders the changes, the forge merges them,
  and the goal wakes only for what needs judgement (section 7).
- **Design docs go through git.** The goal delivers them to its own run branch,
  and a change with no producer lands that branch into g. g stays written by
  change alone (section 4).
- **A spec goes with its task,** whole, in the change's words. Plan-wide rules
  are repeated in each spec. No spec files, and no notes as a go-between
  (section 5).
- **A call and a report get room** to carry that: about 128 KB a call, and a
  report that follows it (section 6).
- **The same roles as the pipeline,** at its models and efforts, with its
  charters and temper's tweaks. The effort of the goal's later runs waits on a
  survey (section 8).

## 2. The roles

| Role | Does | Executor | LLM |
|---|---|---|---|
| the designer | surveys, writes the design, makes the tree | the goal's first run | Opus 5.5, xhigh |
| a survey | one question, answered to the run that asked | a sub-agent of that run, inspect only | gpt-6.1-sol, high |
| the goal, woken | answers, escalations, what section 7 lists | the goal's later runs, each fresh | Opus 5.5, section 8 |
| a coding agent | one spec | a change's producer | gpt-6.1-sol, high, whatever the spec's size |
| a foreman | settles a question from the spec and design, or passes it up | a delegate of the change, read-only | gpt-6.1-sol, medium |
| a repair, a resolution | as `reference-code-production.md`, section 2 | the coding agent's charter | gpt-6.1-sol, high |
| the merge job | as `reference-code-production.md`, section 2 | a job | none |
| the close-out | status files, Later sections, the draft's fate | a change into main, after the landing | gpt-6.1-sol, high |

- **Extra designers stay changes** (N13), for a plan too wide for one writer to
  keep the work fed: a change under the designer's charter, its design landed
  into g. Most plans need none. In the pipeline's last 63 cuts no writing
  sub-agent ran.
- **The close-out moves after the landing,** where the pipeline has it. This
  revises N16.

## 3. The first run

1. **Reads and surveys:** the draft, the designs it touches, their status files
   and the code, with a shell and read-only surveys.
2. **Writes what outlasts the plan** into its checkout: design sections, status
   entries, Later sections. Delivers and lands them (section 4).
3. **Makes changes as they become certain,** each a spec (section 5), with
   `after` for the order, and a change citing a design that hasn't landed
   `after` that design's change, so it starts early on its pushed head
   (`reference-code-production.md`, section 7).
4. **Doesn't wait to finish.** A change starts as soon as `change` is decided,
   while the run goes on. This is the pipeline's release of early tasks, with
   nothing to build. The run ends when every part of the request is in the tree.

Then the goal parks (jig's `domain/engine/executors.md`, 4).

## 4. A goal's delivery

- **A goal's run may modify its checkout and deliver,** where it holds a goal
  branch. A chat's, an advisor's and a reviewer's runs still inspect alone
  (`../domain/agent.md`, 4.3).
- **It delivers to its own run branch,** `temper/<root>/r<task>`
  (`../domain/forge.md`, section 11), never to g, so it takes no writer slot of
  g, and merges into g go on while it runs. The project's checks run at
  delivery, as for a producer.
- **Delivering isn't finishing.** The goal's result stays a report. jig's
  charter translation grants `deliver` and `modify` with a change contract today
  (jig's `domain/hosts/charters.md`); a goal takes them with no change among
  its outcomes.
- **`change` lands it:** `{"branch": "run", "landing": "goal", "words": "..."}`.
  It is a change on its requester's held branch as `"mine"` is
  (`../domain/forge.md`, 8.1): no producing step, its head the branch's tip as
  read at opening, held by the goal, one at a time, and closing deletes nothing.
  It lands into g by g's queue like any change. A conflict goes to the merge job
  and a resolution.
- **g stays written by change alone** (`../domain/forge.md`, 11.1).
- **Later runs may deliver too:** a revision of the design is the goal's to
  write, or a designer's. A run starts from g's tip, with whatever the run branch
  holds that g lacks merged in, so its delivery fast-forwards the run branch.

## 5. Specs

| | A design doc | A spec |
|---|---|---|
| Says | how the system works | one task's work order |
| Lives | `docs/design/`, on main for good | the task's words, in the store |
| Read by | anyone, later too | that change's agents |
| Goes through | git | `change` and `amend` |

- **A spec holds** the goal and its acceptance; the design sections to read;
  sketches of the outer layer and of the tests; the files it owns, the reader and
  neighbour files it may change and how; the likely options, each pre-approved
  with a default; its checks; its report.
- **Plan-wide rules are repeated in each spec.** A shape that outlasts the plan
  is design. This costs a few KB a change, and saves a producer a read.
- **Notes keep jig's use:** what runs learn. They carry no spec: the user sees no
  gain in the indirection, since they use the same network, memory and disk. A
  survey answers the run that asked.
- **A change to a spec is an `amend`,** which shows as a revision. It replaces the
  words whole, so a large spec costs its output again. Answers stay in the store
  (`reference-code-production.md`, section 8).
- **Costs,** as the user reads them: "I don't think there's any downside in
  carrying the spec with the task, it can easily be 64k". jig's `tasks.spec_bytes`
  is already 131,072 and a brief's task section already holds a whole spec
  (`../domain/agent.md`, 4.3). A goal's plan section shows each delegate's state
  and spec revision, not its words.

## 6. Room

- **A call:** `tool_payload` follows a model's declared `output`, about 10,662
  bytes at 8,192 tokens (`../protocol/configuration.md`, 13.8). `output` isn't sent
  to the providers we use, so it changes no model's behaviour. The reference
  takes about 98,500 tokens, 128 KB, as the user sized a message on 2026-11-03,
  and 48,000 tokens, 64 KB, if the start's checks refuse it: the agent's largest
  turn, 16.8 MB at init's figures, has a ceiling of 33.5 MB and hasn't been
  measured there. The start's checks name each limit that moves with it.
- **A batch:** `authority.batch` goes from 4 to about 32. Today a `change` of five
  changes is refused as too large, whatever its bytes.
- **A brief:** `hub.charter_bytes` goes to 262,144, to hold a whole spec beside the
  charter and its tools. Past it, a brief is cut.
- **A report:** a change's `body` is held to 3,584 bytes by jig's
  `CHANGE_BODY_MAX`, smith's outcome and `hub.outcome_bytes`
  (`../domain/agent.md`, section 10). It follows the call's payload instead, so it
  is a setting from then on. In the pipeline, 71% of 4,468 coding reports passed
  3,584 bytes, with a median of 5.3 KB and a p99 of 24.7 KB.
- **First, a test:** an argument of 64 to 128 KB through Claude Code into smith's
  MCP server has never run. The largest seen is about 10 KB.
- **Budgets aren't a plan's bound yet:** the pipeline has none, so a deployment
  starts goals with `--as-allowed`, or too high to hit. The half a `change` carves
  (`../domain/agent.md`, 4.3.1) stays.

`model-catalogue.md` moves models into the store, and keeps window and output the
stock's (its section 2). This section changes the reference's figures. Whether the
stock's follow is settled with that plan's owner.

## 7. Wakes

The goal's charter takes `results: last_or_failure` (jig's
`domain/tasks/messages.md`, 3): the pipeline's designer is never woken by a result.

| Wake | The goal |
|---|---|
| a question a change relays | settles it, amends, makes a change, or asks its person |
| an escalation, a failure | releases, amends, cancels, or passes it up |
| a designer's result, where there is one | subscribed with `subscribe`: makes the family's changes |
| a landing overlapping open work, as news, at most once in 30 minutes | a catch-up (`reference-code-production.md`, section 11) |
| a red, as news | a repair (`reference-code-production.md`, section 10) |
| the last result | reads the reports' Left lists; makes the landing |
| the landing's result | makes the close-out, into main |
| the close-out's result | finishes |

- **Questions wake at once,** as the pipeline answers an integration plan's.
- **Catch-ups wake on overlap alone** (`../domain/forge.md`, 7.3), not on every
  move of main as the pipeline's do. The user: "it covers plain conflicts, and I'm
  okay with semantic conflicts being caught at merge time or after landing, they
  should be rare."

## 8. The goal's later effort

Every answer the pipeline has given ran at medium, against xhigh for its cut. One
task has one LLM choice (N11). A survey of those answers decides: fine means
medium; tight means try high, and look again. Medium then needs xhigh for the first
run and medium after it, which is the LLM choice per kind of wake of
`reference-code-production.md`, section 16; until then, an amendment after the
first run. No choice is made here. `per-wake.md`, section 5 gives a first and a
later choice, made at the goal's start, and ends the amendment.

## 9. Where charters live

- **`temper/stock/init/` and `temper/stock/reference/`,** each `charters/*.md`, in
  plain Markdown, beside its settings: models, efforts, tools, wakes, templates,
  landing rules and limits. A `README.md` lists every charter with its model,
  effort, tools, wakes and size. More stock sets join as siblings.
- **Rust reads them with `include_str!`,** as `temper-docs` reads help, with no
  generated code. `docs/examples/reference/*.json` becomes generated, and a test
  holds it to its sources.
- **The reference stays out of the binary** (`../protocol/configuration.md`, 13.9).
- **The move changes no text.** It follows `model-catalogue.md`, which changes these
  charters.

## 10. Open questions

- **The landing and the close-out made up front,** with `after`: then no wake makes
  them, and the tree runs to the end by itself. A landing kept by dependencies hasn't
  been walked, and each later batch must amend its `after`.
- **A spec past one call:** split it, or have `change` name a note, as
  `propose_settings` does (`../domain/agent.md`, 8.1). One of the pipeline's 1,406
  specs passed 42 KB, and none 64 KB. Not built until one needs it.
- **A foreman before a repair:** the pipeline reads a refused branch first. temper
  repairs at once. Nothing has measured the difference.
- **What a designer reads of goals beside it,** to keep off their files, once several
  overlap (`reference-code-production.md`, section 16).
- **A goal's own notes across its runs:** a walk found a goal couldn't find them, and
  search takes a literal, case-sensitive match of a description. It's a bug to fix,
  and blocks nothing here.
