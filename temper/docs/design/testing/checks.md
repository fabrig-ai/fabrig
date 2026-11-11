# What the tiers check

The service's startup world holds call-input composition at every
entrance (`../protocol/configuration/limits.md`): omitted allowances below,
at and above the agent's declared payload, explicit profile-default and
short allowances kept, core/store/worker-wire parity, engine-only
configuration, worker-only runtime and memory agreement, and per-kind
call/reply Terms mismatches. The binary's offline store reader supplies
checked agent metadata; its init metadata regression holds the declared
payload through that reader.

Every world checks what the strategy lists (skein/docs/foundation/testing-strategy.md, section
6). In temper:

- **Contracts** include each operation's deadline and identity, and each
  component's own (`../domain/worker/README.md`, section 9).
- **Engine replay** runs two independent walking, escalation or role
  worlds in lockstep under the same stimuli. After each iteration it
  compares their complete traces exactly, and hashes the frozen root,
  children, store and pending scripts through their `Hash` implementations
  (skein's `runtime/lib.md`, 5.1). The world's test store hashes the four fields
  its `Debug` prints: `applied`, `rows`, `pending` and `held`.
  The comparison uses a test-only 64-bit word hasher: its
  digests are compared within one process and never kept, with no
  cross-run or cross-target promise. A digest mismatch fails with both
  states' text even when that text agrees. At each seed's end the worlds
  also compare their complete `Debug` text, retaining checks of what
  `Hash` omits, before both close through the cold restart oracle.
- **Secrets stay beside the domains:** in every world with the protocol
  layer, no git identity's value or API token appears in a domain's
  state, a fact, a store record or a log; the fake git program checks
  that a credential reaches git only through its environment (`fakes.md`, 3).
- **The process tier** adds that what the binary started ends with it:
  after a drain or a kill, no agent or git process of its remains; and
  that production's containment holds: a stopped run's process tree is
  empty, and a command cannot write a git directory.

## 1. Focused tests and fuzzy tests

Every tier follows skein/docs/foundation/testing-strategy.md, 8.1: CPU budget, wall hard limit,
starved warning and the three-question repair rule.

The two suites are the strategy's (skein/docs/foundation/testing-strategy.md, section 8). In temper:

- **Focused tests** are the strategy's, with facts changing nothing (a
  run is the same whether its facts are kept or dropped): every test
  in each world's `suite` binary outside its `fuzzy::` modules, the real-loop
  world's stories among them (`real-loop.md`). Step scenarios in the client domain and
  view also use `tests/suite`. Accounts, first-repository, dependency,
  change-panel, transcript, identity and rate scenarios run here: their
  behaviour defines their tier. Repeated formatting and codec cases are rows
  of the existing step tests; duplicated scenarios are removed.
  `tests/client/tests/suite/reloads.rs` checks watcher release across more
  loads than the party's watcher bound. The service's provider test holds
  document methods, query routing and authorization refusals without Chromium.
- **Fuzzy tests** are each world's `tests/suite/fuzzy/*.rs` modules. A sweep
  settles each random world under every invariant, and reaches every
  ending among them.
- **Budgets.** The focused suite's 30 s stands until the user sets its
  CPU budget from measured numbers; the fuzzy suite's is 180 s, the
  strategy's 120 raised by half by the user's word of 11-08, for now, to
  be revisited (`tools/tiers/budgets.toml`). Both are separate from the
  profiles' wall hard limits. Within them, guides:
  - the worlds below the domain other than the real loop (the protocol,
    machine, client, one-process and reconcile worlds) together at most
    8 s of the default suite;
  - the real-loop world's tests, its stories included, together at most
    15 s of the default suite: a standing ceiling, sized to the smoke
    that `real-loop.md` keeps the real loop to ("one end-to-end path per feature,
    and each behaviour and corner case tested in the lower world that
    owns it, never here"), which the user revisits when the sum reaches
    it;
  - the worlds below the domain, the real loop included, together at
    most 80 s of the fuzzy suite;
  - a focused protocol-world test at most 1 s, and a one-process or
    real-loop story at most 2 s;
  - a GitHub story of the real loop (`real-loop.md`) at most 1 s, and one that runs
    a whole chat (its agents, git and `ctl`) at most 3 s, so that its
    eight stay within the real loop's ceiling with the rest; a
    whole-chat GitHub story that would pass the ceiling goes to the user
    with its numbers.
  - a playground story walked natively through the web client's host
    (`../playground.md`, 9) at most 3 s: it runs whole chats, a delegate
    with its agents, git, CI and a landing, as a whole-chat story does.

  A test that misses its guide follows the three questions in 8.1; it
  moves only if what it checks belongs in another tier. It is never
  merged with another or named as an exception: a real-loop story with
  CI and webhooks (`real-loop.md`) is held to the 2 s as any other. If GitHub's
  stories would push the real loop's sum past 15 s, the ceiling goes to
  the user, as below, and no real-loop story moves to the fuzzy suite to
  make room for them: cost never decides a test's tier.
- **The worlds' sums are measured as the suites' rules are:** user plus
  system CPU time per process, waited descendants included, summed
  across the world's selected processes (skein/docs/foundation/testing-strategy.md, 8.1).
  No test's CPU is measured within a shared process. For comparing an
  individual test's guide, run it alone through nextest on the idle
  development machine with the tier's profile and `-j 1`, taking the
  median of three runs; these diagnostic measurements keep the same
  budget rule and do not replace a shared-process suite total. So no
  test asserts its own wall time, and no story's
  deadline is its guide: a deadline is the world's patience, there to
  end a hung run. A story that shows a landing needs no slow poll sets
  the polls beyond that patience, so that one needing a poll fails
  (`suite::protocol::a_landing_merges_on_its_own_wakes_before_any_resource_poll`).
- **The sums are the worlds' guides, never a landing's.** A test within
  its own guide lands. A change that adds to these worlds gives both
  default-suite sums before and after it: the other worlds' against 8 s,
  the real loop's against 15 s. When the other worlds together pass 8 s,
  or the real-loop world 15 s, their slowest tests are profiled, then
  made cheaper at the cause the profile shows, or moved only where their
  checks belong under 8.1, by named steps. A change after which the real
  loop's sum is at 15 s or past it puts the ceiling to the user, with
  its measurements (the repository's `AGENTS.md`, section 5); until the
  user moves it, the 15 s stands.
- **Outside both suites:** the process tier's binary `process` and the
  live tier's binary `live`, which both suites' filters leave out, run
  under the nextest profiles `process-temper` and `live-temper`, the
  live one after every landing and by `scripts/proof-live.sh` (`live-tier.md`). The
  gate never runs them.

A fuzzy test lives in its world's crate, beside the focused ones, and
uses that world: a world's settings, random ones included, are the
world's, not either suite's. The nextest filters select `fuzzy::` modules
in the `suite` binary. A binary's tests share its process, one at a time
(skein/docs/foundation/testing-strategy.md, 8, *A binary is a process*), so a memory scenario
meters only what grows within its own spans (skein's `heap`), and reads no
counter a sweep before it moved.

## 2. The worker's runtime world

`tests/worker/domain/src/runtime.rs` drives the real worker root and its
children against scripted engine and process records and deterministic
git terminals. Its independent boundary referee checks complete turn
replay, exact names and cumulative spend, read credit, identical answers,
separate push title and body, merge paths and expected heads, and release
at iteration reclaim points. Focused stories cover busy backoff,
reconnects, crossed acknowledgements, ordinary saved work and
snapshot-free merge parking. Replay compares digests of the full boundary
records and facts; dropping facts leaves every boundary record unchanged.
Referee tests inject a changed duplicate body and an unseen turn prefix
and require rejection. Counted memory in the root and its children
exercises saturated turn windows, transcripts and conflict paths; a small
deterministic sweep varies body lengths and one-to-three count and byte
windows.

## 3. The client and the engine, joined

A client's world on a scripted engine agrees with the client by
construction: only a world that joins the client to its engine shows
where either side has drifted from their contract. So `tests/client`
joins temper's client to temper's engine root, and there is no client
world of temper's on a scripted engine (../client/testing.md).

- **The translation** between the client's vocabulary and the engine's
  is exhaustive both ways, so a variant added on either side without
  the other is a build error.
- **The joint world is the client's acceptance.** Each client feature
  has its stories there, or extends one.
- **jig's scripted engine** is jig's; the forge's client family has no
  scripted engine of temper's, so its stories run against temper's
  engine root alone.
- **The agents' tools** run there too: their root, as a scripted agent
  beside the fake person (../client/ctl/testing.md, 2).
- **The TUI's host** runs there too (../client/tui/README.md, 3): its
  shipped host with its root linked, as a native shell holds it, whose
  exchanges and stream the world's front carries to the engine's
  clients' component as the transport would, signed in by the printed
  link the component makes; its stories read the host's frames as text.
- **A run's live text** has its story there, with a scripted host in
  place of an agent: the host sends a run's text in pieces, held after
  the first (jig's `protocol/channel/README.md`, 5.4), and the fake person
  reads the chat's words being written, that piece before its "▍", then the
  rest of the held text, and then the committed turn, which replaces it.
  The joint world's front plays the run's worker: at the chat run's
  assignment it sends the reply's first piece up the worker link, as
  the text kind at offset 0, and holds; at each of the story's releases
  it sends the next of its parts, the rest of the text as a second piece
  at its offset, then the committed turn with the whole text and the
  run's answer.
  Retries, gaps and cuts are jig's views' to test (jig's
  `domain/engine/views.md`); this story checks temper's wiring
  from the engine's root to the chat's page. The same story over real
  agents runs in the real loop (`real-loop.md`).
- **A change waiting on a review on the forge** extends the story of a change
  watched through CI, its gate and landing: with a landing rule asking for the
  person's approval, the change waits at its merge, the person's *Needs you*
  draws its card (`../client/forge/README.md`, 4.1) with "Review on Forgejo ↗" at its
  files page's address, a review approving it at its head is posted on the
  forge, and the card goes as the merge is admitted, before the change lands.
  This is where the card and its link are checked whole, rather than in the
  browser tier: no browser story there reaches a change waiting on a forge
  with an address, the box's demo forge refusing such a rule
  (`../domain/forge/change/gates.md`, 3), and the tier is at its cap (`../client/web/testing.md`,
  3).

## 4. Settings in the store

How settings in the store are tested (`../protocol/configuration/settings.md`
; jig's `domain/engine/settings.md`, and its
`domain/testing.md`, sections 4 and 9, for jig's own). By the pyramid,
each behaviour in the lowest world that owns it, and one smoke above:

| Where | What | Priced |
|---|---|---|
| step tests, in their crates | a landing rule's record: its golden bytes, the largest at its bounds within the kind's bound, one past refused; the forge's judge of each item, each refusal and an accepted rule, by a table; the two standing verdicts over a table of rules, deployment's and project's, enforcing or not, met, wait and refuse, a branch no rule covers; the stock defaults passing jig's one check for `temper-box init`'s document, for one with a single model and for one with the demo; each of the six members refused, naming its path; `temper-box init`'s document without them, and no `charters/` | milliseconds, outside the worlds' guides |
| the client's step and view tests (`../client/testing.md`) | How work runs' rows from the `settings` document and each setting, in each tag; the card for an owner and for a member; the review's split of a proposal's items into enforced and followed, and its effect; stale and accepted; each act as the operations it sends; the gallery's worlds and the terminal's goldens gaining the two pages | milliseconds each, in their suites |
| the forge's world (`tests/engine/forge`) | ../domain/forge/README.md 10's settings stories: a rule changed between two merges; a gate a rule adds asked for on a queued change, which leaves the queue; one a rule drops no longer waited on; a project's rule adding an approval; a rule refused; a change kept on its charter, and one held when it is no longer offered; a restart handing the values back before the records | about 0.3 s, seven stories at the world's usual cost |
| the engine's domain world (`tests/engine/domain`) | through the root and the core on jig's store: a change naming a landing rule and a charter committed in one decision, and refused whole when the forge refuses its rule; a project's rule changed by its owner, refused as not theirs for another; a new forge's charter reaching the next change's delegates while a live change keeps its own; a project made after the first start covered by the deployment's rule; the two requirements removed from the rules, landing left to the procedure's waits; the resume fixture restarting with changed values. A stored settings policy and a stored project policy, each admitted under a larger policy allowance and committed through the real store, refuse a cold start under the smaller allowance naming the setting or project; a policy at the bound is admitted in its own story. Root and store use the same test envelope, including sufficient journal and page bytes. | about 0.6 s |
| the engine's domain world, configuring by Ask temper (`../domain/agent/agents.md`, 1 and `../domain/agent/ask-temper.md`) | scripted runs of charter 11 from template 5: an owner's chat proposing and its acceptance heard; a member's, waiting for an owner; one withdrawn as stale, read again and proposed afresh; one rejected with a reason and revised; one accepted with running work moved; `see` reads of each kind, a held charter, a revision and the history, each part within its bound; `propose_settings` refused for a stale revision and for an item out of shape; a run with no `settings` family refused by its authority. The settings family's own tools, which no stock charter is declared (`../domain/agent/tools.md`), keep their stories on a charter of the world's own holding that family: the `settings` read's offsets and pages, a proposal from a note, the note's refusals and its replay, `more` refused to a charter that is not asking, and a rule refused at acceptance | about 0.5 s, its stories at the world's usual cost |
| the engine's domain world, Ask temper's configuring reads (`../domain/agent/agents.md`, 1, and `../domain/agent/ask-temper.md`, 1) | scripted runs of charter 11: a project's ways and questions read at their stock with none stored, and as stored once a change sets them; a read that finds nothing, answered not read; the inventory's people, its own project's alone, the person and temper's writer marked; a member's chat reads what an owner's does and a run without the settings family is refused; an approval's refusal naming no command | its stories at the world's usual cost, not measured apart |
| the engine's domain world, a box's approval identities (`../domain/agent/agents.md`, 1; `../domain/forge/adoption.md`, 6) | on a box whose owned Forgejo's `Reconcile` answered the person's id, and on one whose manifest-made App's account is the person's: the inventory names the person's forge account by the link; their owner-role rule with `people: 1` is accepted, and `people: 2` refused naming the forge; a link cleared when the deployment has a second owner, and the rule then refused. On a deployment of several members, and on a pasted App, the requester without a forge identity is refused as before, and accepted once a human collaborator is adopted as admin. The script checks context and outcomes, not a real model's judgement | within the domain world's focused budget |
| the engine's domain world, a box's approval waiting (`../domain/forge/change/questions.md`, 2) | on a box whose forge links its person and whose rule of theirs is accepted, as in the box's approval identities, on each forge it links: a change made and stepped to its merge waits on the person's approval; the person's `approvals` document holds its row, theirs to give, while a second party of another project reads none; the linked user's review approving it at the head is posted on the fake forge; the row goes as the merge is admitted, and the change lands. The read's rows by state, head and role are the root's and the forge's step tables | within the domain world's focused budget, steps of an existing story |
| the engine's domain world, a charter's documents (`../protocol/configuration/settings.md`, 6) | through the core's plan, the root's binding and the forge's gather: a run of a charter naming a document gets its section at the branch's head; after a change lands that edits the file, the next run's section has the new text; a heading's part alone; a missing file, heading or repository, a section saying so and the run going on. The gather's own cases (a heading's bounds, a cut at a line end, each section saying why) are the forge crate's step tests | about 0.1 s, four stories |
| fuzzy, in an existing sweep | one operation in the forge world's `suite::fuzzy::simulation`: random changes of landing rules among the changes stepping and merging; its referee holds that no merge lands without each requirement of the rules held at its decision | within the fuzzy suite's 180 s (1), measured with it; no new binary |
| the client and the engine, joined (3) | Ask temper's page, and `chat start --template 5` through ctl, starting Ask temper, whose run of charter 11 proposes; "How work runs" read at its defaults and after a change, with its revisions in use; a proposal's review page split into temper enforces and your agents follow, with its effect; one accepted, making a revision; one withdrawn as stale; a member's, waiting for an owner | about 0.4 s against the 8 s guide |
| the one-process world (`README.md`, 2.5) | its fixture's document without the six members; its restart story with one change of settings before it | no new story |
| the real loop (`real-loop.md`) | one smoke: a gate added to a project's landing rule from the client, the next change asking for its review and landing after it | at most 2 s, its guide; about 11.5 s of the 15 s ceiling, |
| the system world (`README.md`, 2.1) | running work keeping its revision, and `revision advance` moving it, one story (`../client/ctl/testing.md`, 4) | at most 2 s, a world story's aim (1) |
| the process tier | Owned Forgejo gates, real signals and process-tree supervision only (`process-tier.md`). Settings reads, proposals, owner decisions and held revisions use the CLI machine, core and system worlds. | process-temper; one focused wiring smoke |

- **The tests that move:** every test document and typed builder that
  writes the six members (25 files of documents, 20 of builders, a JSON
  fixture and the cluster documents) takes the stock defaults, or, where
  its stories need other settings, defaults of its own passed to the
  service in the stock's place (`../protocol/configuration/stock.md`),
  so that no story pays for a change of settings it is not about; one
  story per world makes its change through ctl after sign-in. A story on
  a cheap fake model gives its own run floor and cap: the stock sizes
  them from C, the default model's worst completion, so a fake's run
  would be capped at a few units (the fuzzy repair and case 4 keep 1
  unit, cases `fakes.md`, 1 and 3 a 10-unit cap). A world that copies init's
  seed onto a cheaper model prices the seed for its own model, and the
  cluster fixture keeps five charter slots for the stock's five. Their
  times do not change.
- **Its checks:** settings change the facts of the forge's world and the
  engine's domain world (how a merge is judged), so their focused and
  fuzzy scenarios run with them.
- **Measured** by 1's rule. Should the smoke miss its 2 s, or the real
  loop pass its 15 s, the numbers go to the user; no story is cut or
  moved to fit.

## 5. Stage 1: the golden, its check and its judge

Stage 1 of configuring temper by talking to it
(`../draft/configuring.md`, 8.1) is a walk, never in the gate: a pass
of QA, playing a person on a box with the stock settings, has Ask
temper (`../domain/agent/agents.md`, 1 and `../domain/agent/ask-temper.md`) propose the reference's
configuration from the person's documents alone, accepts it, and checks
what was accepted twice, by code and by a judge. What it is checked
against is the golden: `../draft/reference-code-production.md`,
section 14, cut into obligations, with the values of the reference's
profile, `docs/examples/reference/` (`../protocol/configuration/reference.md`
).

- **Held by the acceptance, never by the binary.** The golden, the check
  and the judge's brief are one tool crate, `tools/stage-one`
  (`temper-stage-one`, `README.md`, section 5), which may read temper's crates and
  which no crate of the binary reads. Neither the stock nor the
  documentation compiled into the binary carries any of it, so the agent
  works from the person's description alone.
- **The profile is the golden's values; the obligations say which
  count.** An obligation names a role or a setting, a member of its
  value and a test; its expected value is the profile's item for that
  role, at that member, unless the obligation states its own. A member
  no obligation names is free: a charter's spend, its call timeout, a
  run's reserves stay the box's, as the profile leaves them
  (`../protocol/configuration/reference.md`: no budget or time moves).
- **The profile's placeholders** for the forge and the repository
  (`127.0.0.1:3000`, `acme/widget`) stand for the box's: the check
  replaces them in every expected value, a landing rule's pattern
  compared as its segments' text.

### 5.1 The obligations

Each obligation is one field of one row of the reference's table, and
of one kind:

- *a setting*, which the accepted settings hold, by one of nine
  tests: `equal` (the value, whole), `at-least` (a number no smaller),
  `bits` (each expected bit of a family mask set, others allowed),
  `lacks` (no expected bit of a mask set),
  `contains` (each expected element of a list present, a member it does
  not name free and one absent counting as null, others allowed),
  `set` (present and not null, whatever its value), `judged` (text the
  judge reads, 5.4; the check only finds it set),
  `unchanged` (the stock's value, which the accepted revision leaves
  alone) and `only` (every current setting of a kind binds to one of
  the golden's roles);
- *the box's*, a prerequisite the box brings (5.2), which the agent
  proposes nothing for, and names where the box lacks it;
- *a gap*, a field of the table no design carries
  (`../draft/reference-code-production.status.md`), which the agent
  must say temper cannot model, offering the nearest thing
  (`../domain/agent/agents.md`, 1): the golden gives the words, any of whose
  alternatives in the proposal's or the chat's text names it.

**The roles.** Charters, templates and the deployment's landing rules
are numbered by the proposal, so the golden names them by role and the
check binds each role to a number:

| Role | The profile's | Bound by |
|---|---|---|
| `coordinator` | charter 5 | the charter `goal-template` executes |
| `designer` | charter 7 | assignment |
| `coding` | charter 2 | the charter forge charter 1 names |
| `foreman` | charter 6 | its number, 6, the charter the forge's foremen run under |
| `triage` | charter 8 | assignment |
| `goal-template` | template 4 | the template whose `resources` hold *its own goal branch* |
| `small-goal-template` | the stock's goal template | the stock's number |
| `main-rule` | landing rule 1 | the deployment's rule whose pattern ends exactly `main` |
| `goal-rule` | landing rule 2 | the deployment's or the project's rule whose pattern ends open on `temper`, or open on nothing beneath a segment `temper`: each covers a goal branch |

A role bound by assignment takes one of the charters the accepted
revision adds that no anchor took: of every assignment, the one that
meets the most obligations, ties going to the lower numbers. The check
prints the binding; a role left without a charter makes each of its
obligations missing.

**The rows, cut:**

| Section 14's row | Obligations |
|---|---|
| the coordinator's charter | setting: its instructions (`judged`); its model, `model.name`; its workspace and sub-agents, `inspect`, `shell`, `agents`, its `modify` free, since no word of the reference says what a goal's runs deliver, where it lets them (`../domain/agent/tools.md`); starting fresh, `resume` false; `waiting` 0; its wake policy, the members of `wake` `../draft/reference-code-production.md`, section 13 names (`words`, `notices`, `news`, `results`, `questions`), `answers` and `timers` free; its documents, `documents` (`contains`: `AGENTS.md` and the code's rules, by path, a heading free). Box: Claude Code, the program the document's model runs on. The tools it names are the authority's families, the rules' and policy 1's rows below |
| the designer's charter | setting: its instructions (`judged`); its model, `model.name`; the workspace, delivery and sub-agents, `inspect`, `modify` and `agents` true; `documents` (`contains`: `DESIGN.md`, the design rules, whole). Box: Claude Code, as the coordinator's |
| the goal template for drafts | setting: `executor.charter`, `{"role": "coordinator"}`; `llm`, whole (`fakes.md`'s choice); `tracked` (`at-least` 1); `contract.report` (`set`); `resources` (`contains`: *its own goal branch*) |
| a small goal's template | setting: the stock's goal template, `unchanged` |
| the coding agent's charter | setting: its instructions (`judged`); `shell` and `modify` true; the stock's run, which the table retains, `turns` (`at-least` 800) and `time` (`at-least` 28,800 s) |
| the foreman's charter | setting: its instructions (`judged`); its model, `model.name`, gpt-6.1-sol, set on the charter (the draft's status records the departure); `inspect` true, `modify`, `shell` and `agents` false |
| the forge's charters (kind 3) | setting: 1 names the coding agent's, its anchor; 2, the reviews', `unchanged` (unused, since neither landing rule asks a gate); the foremen's, the foreman's role's number |
| each change's LLM choices | setting: the coding agent's model, `model.name`, gpt-6.1-sol; the coordinator's instructions (`judged`: a coding change at gpt-6.1-sol at the spec's effort, a designer's at `fakes.md`'s choice); box: gpt-6.1-sol on smith's direct backend with an OpenAI account's key |
| the project's policy and rules | setting: the rules' ceiling's and the project's own policy's role 0's tool families, its own ceiling holding them by jig's check, (`bits`: 1,983, the stock's with the families the coordinator's row names, 13.9), their note scopes (`bits`: 3, the goal's and the project's), and for role 0 no effect family (`lacks`: 64); their delegation's kinds (`contains`: each charter role's charter, the stock's procedures 2 to 4, the merge job, job 3, and role 0) and its tasks and depth (`at-least`: 1,024 tasks, 4 deep, N21); policy 1 free, a kind no obligation asks for: it reaches no project already made (jig's `domain/people/acts.md`, 2), and the walk's project is |
| the project's ways | setting: its `way`, the goal template for drafts, by role; its switches free. The project's questions are free too, a kind no obligation asks for: the coordinator names each change's route |
| the landing rule for goal branches | setting: `goal-rule`'s `pattern`; `ci` false, `up_to_date` false; `gates` and `approvals` empty |
| the landing rule for main | setting: `main-rule`'s `pattern`; `enforce` true, `ci` true, `up_to_date` true; `gates` and `approvals` empty; the deployment's rules `only` the two, so the stock's rule on every branch, which asks CI and a review, is replaced or removed |
| the repository's merge style | box: a merge commit, the repository's |
| the project's checks | box: `.temper/pre-pr` in the repository |
| g's tier | box: the document's tier `goal`, following goal branches, `span_ms` 1,200,000 |
| main's tiers | box: the document's tier `slow`, following main |
| questions | setting: the coordinator's instructions (`judged`: `foreman` for each coding change, `up` for a designer's); box: four per change, the document's `forge.change_policy.questions`, the reference's explicit override of 4 |
| the merge job | box: the document's `merge_job`, `.temper/merge` in the repository |
| the profile's limits | box: the limits 5.2 names |

The golden's file holds each obligation with its id, its row, its kind,
and for a setting its subject, member, test and any value of its own;
its rows above are the design, the file their only encoding. The
triage's charter's row (`../draft/archive/reference-landing.md`, section 6;
`../protocol/configuration/reference.md`) cuts into its
instructions (`judged`), its model, and `inspect` true with `modify`,
`shell` and `agents` false. No field of the table is a gap: the design
carries each, as a setting or the box's. The kind
stays for a field a later edition of the table adds before a design
carries it.

**Beyond the obligations,** the check fails on:

- *a setting no obligation asks for:* an item of the accepted revision
  whose setting binds to no role and is no obligation's subject, of a
  kind the golden does not name as free, and no landing rule removed,
  since the stock's on every branch must go or be replaced;
- *a rule made structure without the person's choice*
  (`../draft/configuring.md`, section 2), as far as settings show it: a
  landing rule's gate or approval, or a charter's family the golden
  does not give, fails its `equal` obligation. The pass makes no such
  choice (5.3), so any is the agent's;
- *what needs a restart, proposed rather than said:* settings cannot
  carry it, so the judge reads the proposal and the chat for it
  (5.4).

### 5.2 The box

`temper box` with its owned forge (`../protocol/setup/box.md`),
its settings the stock's. What the box brings, and the pass confirms
before the chat starts (temper's through ctl, the repository's through
the forge's API as an observer, the limits against the box's document):

- **the models:** Opus 5.5 on Claude Code for the coordinator and
  designers; gpt-6.1-sol on smith's direct backend, on an OpenAI
  account's key, for producers, foremen and the triage; a real model
  for Ask temper, charter 11's;
- **the document's members the profile sets** over `temper-box init`'s:
  `docs/examples/reference/limits.json`'s overrides, and `landing.json`, the project's merge job and its two
  tiers (13.9); a change's question bound,
  `forge.change_policy.questions`, explicitly overridden to 4 by `limits.json`;
- **the project,** with its repository adopted, holding `AGENTS.md`,
  `DESIGN.md`, its code rules, `.temper/pre-pr`, `.temper/merge`, the
  tiers' workflows and a merge-commit merge style; and, at the path the
  walk's brief names, `../draft/reference-code-production.md` and
  `../draft/archive/reference-landing.md`, the person's process documents. It holds no
  recipe of the reference either: a copy of fabrig keeps neither the
  profile, this crate nor the plans that describe them, and an agent
  found reading a description of the reference's settings elsewhere
  is a finding against the walk's premise.

Each is a box obligation, recorded present or lacking with what the
pass saw. A lacking one ends the walk before the chat, a finding against
the box, never the agent.

The walk's bounds, the user's (2026-11-04): the box's limits are the
reference's profile whole, as above, its spends and times the box's; the
pass takes at most 3 hours of wall time from the box's start to the
stage's verdict, the Ask temper chat at most 40 of the person's
messages, and the judge one session at Opus 5.5, high, within 30
minutes.

### 5.3 The evidence

After the acceptance, the pass collects what the check reads into one
directory, with `temper-stage-one collect`, through `temper ctl`'s
reads alone (`../client/ctl/tools.md`, 3): the revision's record; its
items, each value whole, as a settings file; every setting's current
value, the project's policy and temper's kinds among them, as a
settings file; the proposal's review and the chat, as text. The pass
adds what only it holds: the box's confirmations, the agent's
description of the accepted settings back as a workflow (asked once
the proposal is accepted, `../draft/configuring.md`, section 6), each
document an accepted charter names, read at its branch's head through
the forge's API, and the person's two documents. The pass answers the
agent's questions from the documents and its brief, chooses no rule to
become structure, and accepts with running work staying as it is.

The revision's edits alone form `items.json`; `current.json` is collected
separately. The collector never unions earlier revisions or substitutes current
values for unrelated accepted edits. The recorded-ctl regression carries the
complete note proposal's accepted set and retains designer and triage bindings;
the eleven-proposal workaround's incomplete final revision leaves them unbound.
The engine domain world proves bounded note writes, including a line past one
call, exact whole-value reconstruction, no application before acceptance and
one complete accepted revision, with refusal and decided-call replay cases (4).

### 5.4 The check and the judge

- **The check,** `temper-stage-one check EVIDENCE`: the binding, then a
  line per obligation (met; wrong, with what it found and expected;
  missing; judged; unchanged; named, and where; unnamed; confirmed;
  lacking), then what fails beyond them. It passes when every setting is
  met, judged or unchanged, every gap named, every box obligation
  confirmed, and nothing fails beyond them. Exit 0 passed, 1 failed, 2
  for evidence it cannot read.
- **The judge,** an LLM, reads what code cannot: `temper-stage-one judge
  EVIDENCE` prints its brief, whole, for a session of its own (never the
  pass's), at Opus 5.5:
  - *the rules:* the golden's rules, the reference's text that stays
    text (`../draft/configuring.md`, section 2's table, and the
    instructions `../draft/reference-code-production.md`, section 14
    gives each charter), each with the role it
    concerns; for each, whether it reaches that role's runs, from where
    the person wrote it, through the bound charter's instructions or a
    document it names: `reaches`, `missing`, or `elsewhere` (another
    role's), with the quote;
  - *the description* the agent gave back, against the reference's
    section 1: `matches` or `differs`, with the differences. It differs
    where it says what the section contradicts, or leaves out what the
    flow turns on (the goal holding its branch, every change landing
    there, designers before their changes, how questions go, the whole
    landing into main at the end); a number or other detail it leaves
    out is no difference;
  - *what needs a restart:* whether the proposal or the chat claims to
    set what the box brings rather than saying it needs the box:
    `said` or `proposed`, with the quote.

  Its verdict is a JSON file in the brief's form, which the pass saves
  beside the evidence.
- **The stage's verdict,** `temper-stage-one verdict EVIDENCE`: passed
  when the check passes, every rule reaches its role, the description
  matches, and what needs a restart is said.
- **It moves with the reference.** A design that comes to carry a gap
  makes it a setting: the golden's file changes with the status entry
  that closes, in the same branch.
- **Tested in the gate as a tool is:** the crate's own step tests, in
  milliseconds: the golden decodes, and each setting obligation's member
  resolves in the settings form's `FIELDS` (`../client/ctl/tools.md`, 3)
  and in the profile; the check over evidence made from the profile
  itself, as if the agent had proposed it, passes with every setting met,
  and with the profile's charters renumbered still passes; each failure
  by one mutation of that evidence (a value changed, a charter added,
  a role's charter removed, a gap's words absent, a box obligation
  lacking); the judge's brief for that evidence, as golden text; the
  verdict's decoder refusing an unknown rule, a rule twice or missing;
  `collect`'s assembly from recorded ctl outputs.
- **Walked,** never in the gate: a pass of QA, run again whenever the
  agent's instructions, the documentation in the binary or a reference
  milestone's settings change (`../draft/configuring.md`, 8.1).
