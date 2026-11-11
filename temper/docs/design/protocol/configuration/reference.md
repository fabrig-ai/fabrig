# The reference's configuration

The reference process (`../../draft/reference-code-production.md`,
section 14) is settings over the stock, never a stock of its own: a
deployment runs the stock until an owner applies them, and a release
that changes the stock changes nothing of them.

- **A settings file,** `docs/examples/reference/settings.json`, in the
  form `temper ctl settings apply` reads (`../../client/ctl/tools.md`, 3),
  its charters' instructions in it as text. An owner applies it once,
  after the deployment's first start, as a walk on fabrig's box does
  (`../../draft/v0.1.0.md`, 2.1); a later edition is applied the same
  way, its values whole. Its charter 5 and template 4 are its own
  editions over the stock's (`stock.md`), and charters 7 and 8 are its own.
  It holds:
  - *charter 5, the coordinator's:* instructions for the cut (a plan of
    changes it specifies itself, each made by `change` into its own
    goal branch with its dependencies, one batch per family:
    `../../domain/agent/tools.md`, 1, each with questions `foreman`), for
    each question a change relays (`../../domain/forge/change/questions.md`, 1: answer it from the draft and the
    design; amend the change and answer what changed; make a new change,
    add it to the change's `after` by amendment and answer that the work
    waits for it; or ask its person through a person task, the options
    as its choices, and answer once the person has chosen), for each
    hold that reaches it (release it, amend then release it, cancel it,
    or pass it up to its person with its reasons, by `decide`), for
    the escalations of its closer's landing (`../../domain/forge/branches.md`,
    2: on a hold past its bounds, a release with an amendment, a
    change into the goal branch for the landing's needs and then a
    release, which the closer waits for, a cancel of the closer that
    drops the landing and keeps the branch, or a question to its
    person; never a write of the goal branch of its own, nor a landing
    of its own), and for its design's status, written up front with
    each entry a change will close tagged `[in flight: change N]`
    (fabrig's `DESIGN.md`, section 2), the tags of the entries written
    before their changes were made given in its next delivery; never a
    catch-up, a landing, a close-out, or a finish of its own once its
    plan's changes have ended: its closer lands the goal and reports
    (`../../domain/forge/branches.md`, 2); the document's models, the
    workers, its runs starting fresh, waiting 0, a run sized for a
    plan's turns; its default wake policy the coordinator's (jig's
    `domain/tasks/messages.md`, 3): questions, failures, the last delegate's
    result or a failure, which for a goal its closer ends wakes it for a
    failure alone (jig's `domain/tasks/lifecycle.md`, 11), its delegates' and
    its person's words at once, waking news at most once in 30 minutes,
    a designer's result and a person task's result reaching it by the
    watch it subscribes to each (8.4 there); its instructions in parts
    (*Charter 5 in parts*, below), which cover every kind its policy
    wakes it for;
  - *charter 2, the coding agent's:* the stock's with smith's shell
    family (`shell` and `job`: smith's `domain/tools.md`), its
    instructions keep the stock's rule that it runs no build or test:
    the delivery hook runs the lints and focused tests
    (`tools/pre-pr/design.md`). It fixes failures in its own files and
    records a failure outside them under Left; and adds: push early;
    finish with a Left list of what it found and does not own; mark a
    story that fails for code it does not own as ignored, the fault its
    reason, and never fix that code, the change that fixes the fault
    removing the mark; where its change's words, the design and the
    code leave it a choice it cannot make, push what it has and ask its
    change in the question form, its options and its recommendation in
    its words, then call `wait`, the answer reaching this run
    (`../../domain/forge/change/questions.md`, 1); its run
    sized for a change's work;
  - *charter 6, the foreman's:* the stock's, at gpt-6.1-sol, its
    instructions adding the reference's rules: the change's words and
    the design in its checkout settle scope, a file outside the change's
    own is `finish` with a Left item, never permission to edit it, and a
    choice the draft leaves open is `escalate`;
  - *template 4, the goal template for drafts:* charter 5; ended by
    temper's goal closer, `ended_by` `{"closer": {"connector": 1,
    "code": 6}}` (jig's `protocol/configuration.md`, 3.2;
    `../../domain/forge/branches.md`, 2);
    `../../draft/reference-code-production.md`, section 4's LLM choice,
    Opus 5.5 at xhigh with sub-agents at Opus 5.5 and high, and a later
    choice (`../../draft/per-wake.md`, section 5), Opus 5.5 at high with
    the same sub-agents, which every run after the goal's first
    activation takes, so that nobody amends the goal after its cut
    (`../../draft/goal-as-designer.md`, section 8);
    tracked at priority 1; a report; the creator's budget; *its own
    goal branch* among its resources (jig's `protocol/configuration.md`,
    3.3; `../../client/ctl/tools.md`, 3);
  - *template 4's work,* kind 11 (`settings.md`, 2), `template-goal` number 4:
    design and no close-out, since its charter 5's second edition
    delivers the shared design first, as a designer's change (*Designers
    and the cut*, below), and its closer's close step settles what a
    close-out wrote, so that the case's path draws "design" and no
    "close-out" (`../../domain/forge/change/branch-path.md`).
- **The rules and policy 1** are changed on the box from their current
  values, which hold the deployment's own budgets (`stock.md`): read with
  `temper ctl settings show --json`, charter 7 added to the rules' and
  policy 1's delegation (and charters 5 and 6, the merge kind, job 3,
  the goal's closer, procedure 6, the close kind, job 4, and role 0,
  which the stock's delegation holds (`stock.md`), where their stored values
  predate it), so the coordinator may delegate to its designers, its
  goals close, run its merges and ask its person through a person task
  (jig's `domain/people/README.md`, section 7), and jig's control family to
  their tools and role 0's, so the coordinator may name `change`'s plan
  members (`../../domain/agent/tools.md`, 1), and jig's notes family beside it, in
  the goal's and the project's scopes, so that it notes what it learns:
  in all, tools 1,983 with the decide, propose, subscribe and recall
  families its row names and no effect family, and notes 3, as the
  profile's `../README.md` gives them, then applied whole. The widening
  takes the drafter's charter 12 out of that delegation (`stock.md`), since the
  reference's work starts no design session, so that the stock's other
  sixteen executors, the goal's closer and the close kind among them,
  and charters 7 and 8 fill eighteen executor places: the profile's
  `authority.executors` and `tasks.executor_kinds` are 18, which the
  box takes at its next start; the profile's limits (below) reserve
  room for further work.
- **Designers and the cut** (`../../draft/reference-code-production.md`,
  sections 3, 5 and 12): the same file's later edition adds
  - *charter 7, the designer's:* instructions to write one family's
    design into the goal's branch where the project keeps it (here
    `docs/design`: the sections the family's work meets, the status file
    beside each document with a Not yet for each gap between them and the
    code, its Later for what the family leaves out, and the entries it
    closes in the draft's status), never the draft's own text; to cite by
    path and section; to survey with sub-agents where it judges it worth
    it; to ask its requester with `message` only what the draft does not
    settle; to push and finish with a report listing the family's
    changes, a few lines each: a name, the design sections and status
    entries it closes, what it owns, its dependencies and its effort; for
    a revision, the sections revised and the live changes they concern;
    each status entry a family's change will close tagged with it
    (fabrig's `DESIGN.md`, section 2); never a close-out, which the
    goal's closer makes (`../../domain/forge/branches.md`, 2). Claude Code's model at Opus 5.5, its sub-agents
    the same; inspect, modify and sub-agents; the workers; its runs
    starting fresh, waiting 0, a run sized for a family's design;
    documents: the design rules (`DESIGN.md`, whole) and `AGENTS.md`'s
    "1. In one page", on the home repository's `main` (`settings.md`, 6);
  - *charter 5's second edition:* the cut as the draft's section 5 has
    it (S, the shared design, a designer's change; one designer's change
    per family after S; changes needing no pending design), each family's
    coding batch only after every required design change is confirmed
    landed into g, a revision of the design as a
    designer's change followed by amendments of the live changes it
    concerns, and no close-out; each change's `producer` and `llm` (`../../domain/agent/tools.md`,
    1): a designer's change `"producer": 7` at the template's LLM
    choice, Opus 5.5 at xhigh with sub-agents at Opus 5.5 and high, with
    questions `up`, its designer's questions going straight to the
    coordinator with no foreman (`../../domain/forge/change/questions.md`, 1); a
    coding change the coding agent's charter, its default, at
    gpt-6.1-sol and the spec's effort, high or medium, on smith's direct
    backend with an OpenAI account's key (N20); its documents:
    `AGENTS.md`'s "1. In one page" and the code's rules, skein's
    `../foundation/programming-model.md`'s "1. In one page", on the
    home repository's `main`.
  Charter 7 joins the rules' and policy 1's delegation as charter 5 did.
  The box's `standard` profile (`stock.md`) holds the added charters and the
  delegation's executors within `tasks.charters`, `authority.executors`
  and `tasks.executor_kinds`, 12, 16 and 16.
  Each document names the forge's host, owner and repository the walk's
  document fills in, as the landing rules do.
- **The landing rules** are kind 1's rules 1 and 2, items of the
  settings file, with its forge's host, owner and repository, which the
  walk fills in: main's rule (`forge`, the host, the owner, the name,
  `branch`, exactly `main`: enforced, CI passed, up to date, no gate
  beyond CI, which runs the project's full gate at a goal's landing's
  head) and the rule for goal branches (the same path to `branch`, open
  on `temper`: enforced, asking no CI, no update and no gate), and no
  rule on every branch, since rules add (`../../domain/forge/change/gates.md`, 2) and
  such a rule's CI would hold every merge into a goal's branch. So the
  case's project sets its own rule 1 in `project.json`, main's rule
  naming a branch (`branch`, exactly `main`, as above: `settings.md`, 2), in place
  of the stock's rule 1 over every branch, whose CI and review would
  hold every merge into a goal's branch (`stock.md`).
  `docs/examples/reference/landing.json` is the document's
  `application.landing` (`engine.md`), which the walk's document carries:
  project 1's `merge_job`, `.temper/merge`, and its tiers (below).
- **The tiers,** in the same member, project 1's `gates` entry for its
  repository (`forge` and `repository` filled in as the rules' host and
  name are), with no gate and two tiers (`../../domain/after-landing.md`,
  section 2): main's, `slow`, the slow suites, workflow
  `tier-slow.yml`, following main; and g's tier, `goal`, the gate
  without its slowest suites, workflow `tier-goal.yml`, `branch`
  `goal`, `span_ms` 1,200,000 (20 minutes); each uploading its failing
  tests as `failing`, one id per line (`lines`), and described, as its
  card says it, "the slow suites" and "the gate without its slowest
  suites" (`engine.md`). The project's
  `tier_retries` and `verification_bound_ms` are left to their
  defaults.
- **Charter 5's triage** (`../../draft/reference-code-production.md`,
  sections 10 and 13; `../../draft/archive/reference-landing.md`, section 6), in
  its instructions beside the rest: on a red of g's tier, as news, it
  reads the failing tests, the log's end, the tree and the design, and
  finds the change that owns the failing files; a live change's failure
  is that change's, and it amends its words to say so; the rest makes a
  repair change into its goal branch, its words the failures, the log's
  end and the live changes with what they own, to leave alone, while
  other changes keep merging; a red marked still failing that a live
  repair owns needs nothing. On a red after landing, as news, it
  triages the same way, reading the goal's history now on main, then
  makes a repair change into main, small, with `repairs` naming the
  red landing's commit, its words the failing tests and what the triage
  found; a failure it finds a flake or inherited gets no repair, but a
  note on the tier's tests. It writes its report (`../../domain/agent/agents.md`, 3),
  and notes what it learns (a slow check, a
  dead end, a trap in the code) with jig's `note`, in the goal's or the
  repository's scope, and reads the notes before it triages. Its
  goal's end is its closer's (`../../domain/forge/branches.md`, 2): the closer's
  report gives each main pull's CI selection, read by code, so that no
  run of charter 5 reads a CI log for it, nor checks the reports against
  their acceptance before the landing.
- **Acceptance evidence, in each change's report.** Charter 2 maps
  each acceptance item to a named assertion at the lowest tier that
  shows it, adding or extending tests where existing assertions do not
  cover the requested behavior. Its report distinguishes assertions
  added or extended, existing coverage, and checks actually run, with
  what each assertion proves. Page bounds and link validity do not
  establish coverage of a new explanation or pointer. For help trimming
  or a supplied replacement patch, charter 5's shared specification
  instructions identify existing facts at risk and require a
  before/after fact mapping and named retention assertions; charter 2
  preserves unrelated facts during initial production as well as repairs
  and resolutions. Each requested deliverable carries its requested
  evidence in the coordinator's spec, and when each scoped-check table
  row's cost is requested, each row needs its own measured timing.
  Charter 2 reports what it leaves unresolved (missing coverage,
  preservation evidence or measurements) as unfinished, under its Left,
  which the goal's closer quotes word for word in the goal's report
  (`../../domain/forge/branches.md`, 2); the person reads it there, and a change
  that would settle it is theirs or the goal's to ask for. The domain
  world checks the assigned instructions; its scripted choices establish
  no model compliance.
- **No plan done of charter 5's** (`../../draft/goal-runs-itself.md`, sections 2 to 7
  ). Its plan's changes ending leaves its goal nothing to
  judge, and no run: its closer settles the goal's status, lands it and
  reports (`../../domain/forge/branches.md`, 2), the landing's words the goal's
  title in one line, then the draft it was built from, by path, which
  the forge makes the pull request's title and body
  (`../../domain/forge/change/README.md`, 1). A red of g's tier it has heard is its to
  repair while its changes live, by the triage below; a run of the tier
  in flight as the last change ends is not waited for: the landing's CI,
  the project's full gate, checks the whole.
- **A triage delegate** (`../../draft/archive/reference-landing.md`, section 6):
  *charter 8, the triage's*, in the settings file, read-only as charter
  6 is (inspect and the forge's reads; no modify, no shell, no
  sub-agents), at gpt-6.1-sol; its instructions: its words carry a
  red's news (the tier, the failing tests, the run, the log's end); it
  reads the tree, the design, the goal's history and the notes, and
  reports, for each failing test, the change or landing whose files it
  fails in, whether it is new, failing before (inherited) or a flake,
  and the repair it would make, with the files that repair would own;
  it never edits, pushes or runs anything. Charter 5's triage may
  `delegate` a red's triage to charter 8, its words the news, where the
  failures span files no one change owns, and acts on its report as on
  its own triage. Charter 8 joins the rules' and policy 1's delegation
  as charters 5 and 7 do. Charter 5 also gives each designer's change
  `"questions": "up"`, so that a designer's question reaches it with no
  foreman (`../../domain/forge/change/questions.md`, 1), the project's questions, kind 6,
  being none on the stock.
- **Charter 5 in parts** (`../../draft/per-wake.md`, sections 3 and 6;
  jig's `domain/tasks/messages.md`, 6). The bullets above say what charter 5's
  instructions hold; they are written as its shared instructions and
  four parts, in this order, each rule in the part that acts on it.
  The confirmed design landing rule is repeated in shared, cut and
  result instructions to guard each coding dispatch path:
  - *shared,* read by every run: who the goal is, its draft and its
    goal branch; how a change's words are written (a spec citing the
    design in its goal branch by path and section and the status
    entries it closes, the files it owns and what it leaves alone, its
    checks and its report), each change's `producer`, `llm` and
    `questions` (a designer's change and a coding change, as the second
    edition gives them) and a batch's `change` calls, since every part
    makes or amends changes; before issuing dependent coding changes,
    confirmation that every required design change landed into g, not
    a pushed head, delivery or designer report. Once landed, the spec cites its repository path
    and section in words and omits its ended task from `after`, which
    keeps other live prerequisites (`../../domain/agent/tools.md`, 1). This
    charter rule leaves early producer starts as `../../domain/forge/change/questions.md`,
    1 defines them; asking its person through a person task,
    the options as its choices and its recommendation in its words; the
    report note and the notes; reading its report note and the tree
    first at every wake, and parking with `wait` once its decisions are
    made; its design's status written up front, each entry a change
    will close tagged `[in flight: change N]` (fabrig's `DESIGN.md`,
    section 2); a watch subscribed to each designer's change's result
    and to each person task's result it waits on, since only a failure
    or a last result wakes it by its policy; never fixing code, writing
    its goal branch, landing it, making a close-out or a catch-up, or
    finishing once its plan's changes have ended, which its closer does
    (`../../domain/forge/branches.md`, 2); and how to take what every run is offered (jig's
    `domain/tasks/messages.md`, 6): an answer, a decision on its proposal or
    its settings, an effect not made, a `met`, an amendment, a request
    to finish and a person's words, each as it comes, and an entry
    waiting for its decision that no part of the run covers, left
    undecided for the run whose part it is;
  - *the cut,* `on` `first`: the first run
    (`../../draft/goal-as-designer.md`, section 3): reading the draft, the
    design and the code, with sub-agents for surveys; the plan; S, a
    designer's change per family as the first batch. Coding that needs
    pending design stays out of that batch until every required design
    change is confirmed landed into g; coding needing no pending design
    may proceed. Each status entry written before the change that will
    close it is made gets its tag in the delivery after it. Where a
    request settles its changes
    with no shape for the code to follow, as one made of documents
    does, the cut writes no design, says so in its report note, and
    makes its changes. The cut never waits for or reacts to its
    changes, and subscribes to none of their landings: the core
    subscribes the goal to the landings on each branch its changes
    target as the first change into it is made (`../../domain/forge/README.md`,
    5.2), so a `subscribe_forge` of its own to them is a duplicate,
    refused; the cut's part says so in one sentence;
  - *questions,* `on` `question`, `question_answered`,
    `question_ended`, `question_owed` and `words`: a designer's question,
    a question a change relays and a task's words, settled from the
    draft and the design by an answer, an amendment and its answer, a
    prerequisite change and the answer that the work waits for it, or a
    question to its person;
  - *results and holds,* `on` `result`, `notice`, `nothing_awaited`,
    `escalation`, `proposal` and `person_stalled`: a designer's result
    read for its design and family changes, then its family's coding
    batch only once every required design change is confirmed landed
    into g; a person task's result into the answer
    that waited for it; a change's failure, and a ruling that becomes
    design; an escalation released, amended, cancelled or passed up, its
    closer's and its landing's among them (`../../domain/forge/branches.md`, 2:
    a release, a change into g for the landing's needs then a release,
    or a cancel of the closer that keeps the branch);
  - *news,* `on` `news` and `timer`: a red of g's tier and its triage,
    the triage delegate among it, reading the notes first; a red after
    landing, while its closer's landing is verified.

  So the cut reads the shared instructions and the cut, and an answer
  the shared instructions and the questions part
  (`../../draft/per-wake.md`, section 4).
- **Each charter holds what its role acts on** (`../../draft/per-wake.md`,
  section 7), each rule once, and nothing its tools' descriptions or its
  brief already say: a call's fields, a call's room, the wake policy's
  mechanics. The project's rules for building, testing, machines and
  landing are charter 2's, which acts on them; charter 6 judges a
  change by its words and the design, and charter 8 reads. Charter 5
  builds and tests nothing, so it holds only what a spec must carry of
  those rules: the delivery hook checks the changed packages' lints and
  focused tests with their direct dependents, and a spec asks for no
  separate build or test command. Slow tiers and walks run after
  landing. Charter 7 writes
  design and names a change's effort, and holds no rule for building.
- **The profile, whole** (`../../draft/reference-code-production.md`,
  section 14; `../../draft/notes.md`, N21): a box is set up for the
  reference from `docs/examples/reference/` and the repository's own
  files:
  - `limits.json`, the members of the document the reference sets over
    `temper-box init`'s (`../setup/init.md`, 3): its engine's and its
    application's limits' overrides (`limits.md`), merged into the box's
    document before its first start, member by member, an object's
    members over the document's and any other value replacing it, as
    `jq -s '.[0] * .[1]'` merges two files;
  - `room.json`, a call's room (`settings.md`, 7), mostly what such a merge can't
    reach: its `agent` member, `tool_payload` and `output`, which `jq`
    sets on the box's `agent.json`, the output on every endpoint; its
    `worker` member, merged over each worker of the document's array;
    and its `application` member, `root.journal.run_bytes` 65,536,
    merged over the document's `application` as `limits.json` is: the
    journal's bound on a run's stored policy, which a charter's whole
    policy must fit, so that the box's 24,576 refuses a charter of
    31 KB. Its `../README.md` gives the commands. All of it before the
    first start;
  - `landing.json`, the document's `application.landing`, carried whole;
  - `settings.json`, applied once after the first start; then the rules
    and policy 1 changed as above, before the case's project is made,
    since a project takes policy 1 as it is made (`settings.md`, 5, step 1);
  - `project.json`, applied once the case's project is made, its number
    set by `jq` as its `../README.md` sets a `project-policy`'s: kind
    7 of that project, all four shown and `way` 4, so that the project
    shows its goals, and its kind 2 rule 1, main's (the landing rules,
    above). A project's setting cannot be in `settings.json`,
    which is applied before the project exists;
  - the repository's `.temper/pre-pr` and `.temper/merge`, and its
    tiers' workflows (below).

  Its limits size one goal of a hundred changes and bound each change's
  questions to four; no budget or time moves: the spends, the periods, a
  run's time and the agent's timeouts stay the box's (`stock.md`), and a run held at its spend is its
  person's to fund (`../../domain/forge/change/repairs.md`).

  | Limit | The box (`temper-box init`) | The reference | For |
  |---|---|---|---|
  | `tasks.tree_tasks` | 384 | 1,024 | a goal's tree over its life: about a hundred changes of about eight tasks each, G and its person tasks (N21) |
  | `forge.change_policy.questions` | 80 | 4 | questions per change (`../../draft/reference-code-production.md`, sections 8 and 14) |
  | the agent's `tool_payload`, its endpoints' and charters' `output`, and what moves with them | 10,662, 8,192 | 63,740, 48,000 | a call's room, a whole spec (`settings.md`, 7) |

  Every other limit the case needs the box's `standard` profiles hold
  (`stock.md`): G's live delegates (80) with the operations and the inbox
  that follow them, a family's
  changes in one `change` call (up to `authority.batch`, 32, subject to
  the requester's remaining lifetime task capacity), the live changes, holds and tasks
  the forge follows (128), a change's twelve tasks with four foremen
  (80), the executors and charters (16 and 12), and the agent's turns
  (800); so `limits.json` holds `tasks.tree_tasks`,
  `forge.change_policy.questions` and its engine's and application's
  share of `settings.md`, 7's room, and `room.json` the agent's, the workers' and
  the journal's run bound a charter's policy needs.

  A charter's turns, time and run spend must also fit a worker's
  advertised bounds (jig's `domain/hosts/hub.md`): journal room
  alone does not make a worker eligible to run it.

  The settings file sizes its charters and template to the box: charters
  2, 5, 6, 7 and 8 take the stock's run, 800 turns and 8 hours (`stock.md`),
  since a charter is a whole value and its run is no reason of the
  reference's; template 4's report takes the reference's `tasks.result_bytes`, 63,738; each
  charter names at most two documents, `tasks.charter_documents`,
  which jig's settings schema holds; template 4 names the forge's *its
  own goal branch* (`../../client/ctl/tools.md`, 3). Charter 5 makes a
  family's changes in `change` calls each within one call's payload,
  which its model's output bounds (`settings.md`, 7), a later call naming an
  earlier one's changes in `after` by the task numbers its answer gave,
  and fewer changes to a call where a call is refused for its size.
  A batch reserves one task per change and shares descendant capacity
  within each member's policy cap (`../../domain/forge/change/README.md`, 1); it does
  not reserve every member's full policy allowance before sizing the
  next. A task-capacity refusal gives the available and minimum required
  counts and whether fewer changes per call can fit. Ending tasks or
  restarting does not restore the lifetime count. The help's engine
  limits table uses `small` (32 lifetime tasks); the box selects
  `standard` (384), and named overrides replace the profile's values.
  The rules and policy 1 take the delegation the plan needs: 1,024 tasks,
  as `tasks.tree_tasks`, 32 deep as the stock's. A limit the start's
  checks ask to move with these moves with them, sized, and
  `limits.json` holds it.
- **What the repository carries:** the merge commit is its merge style
  on its forge, the repository's default on Forgejo
  (`../../domain/forge/adoption.md`), set where the repository is made;
  the project's checks are its `.temper/pre-pr` (smith's
  `domain/run.md`, 8.1; `tools/pre-pr/design.md`): the format check and
  the counted baselines on everything, then, against the base the worker
  pins (`../../domain/worker/checkouts.md`), the lints of the packages the
  delivery changed and the focused tests of those packages and their
  direct dependents; a lint or test failure the repository's
  `.temper/known-reds.md` lists, and a counted baseline or citation the
  base has too, is reported as inherited and refuses nothing; without the
  pin, the first two alone, saying so.
  Coding agents run no build or test: this check runs them, mechanically,
  at each delivery (the owner, 2026-11-09). A pull request into a goal's
  branch runs no CI (its rules ask none: the rules above;
  `../../domain/forge/change/gates.md`, 2, and `../../domain/forge/branches.md`, 1), so a delivery's checks are
  the only checks a change meets before it lands there; the landing's CI
  into main, the project's full gate, is the guard. An agent can change
  what a check runs: it is no security boundary (smith's
  `domain/run.md`, 8.1). Its tiers' workflows are
  `tier-slow.yml` and `tier-goal.yml`, dispatch-only, each running the
  ids in its `temper_tests` input alone when given them
  (`../../domain/after-landing.md`, section 2). Its merge command is its
  `.temper/merge` (`../tooling.md`, 4.1), which recounts a counted
  baseline left in conflict (each tree's `scripts/old-forms.txt`, by
  its `scripts/old_forms.py --write`) and leaves every other conflict.
  The delivery format check batches Cargo's target roots by target edition
  and nearest rustfmt configuration, retaining rustfmt's module traversal;
  all five trees' counted baselines run against the private snapshot index,
  including newly added sources. Only a tree whose checks all succeed, or
  fail only as inherited, may reuse that success for an identical tree on
  the same base. A first uncached delivery that changes no package must
  also fit the 60-second check timeout; one that builds takes the
  deployment's `check_timeout`, which the owner sets (`README.md`, section 9).
- **Checked as it is read:** a test decodes the settings file as
  `temper ctl settings apply` decodes one, and the landing member as the
  document's is parsed, so an edition that no longer reads is red before
  a walk meets it; and a test merges `limits.json` and `room.json` over
  `temper-box init`'s document, runs the start's checks on it with init's
  agent configuration as `room.json` sets it, holds its figures to `settings.md`, 7's
  relations from that agent, and holds the settings file to them: its charters
  within `tasks.charters`, their documents within
  `tasks.charter_documents`, their turns within the agent's
  `max_turns`, template 4's report within `tasks.result_bytes`. The same
  test holds each charter to what a host takes and a box stores
  (`../../draft/per-wake.md`, section 8): its turns and time, and the rules'
  run spend, within the bounds a stock worker advertises from init's
  agent configuration as `room.json` sets it (jig's `domain/engine/fleet.md`
  ), and its stored policy, its parts with it, within
  `room.json`'s `root.journal.run_bytes`, so that an edition no host
  would take, or that a box can't store, is red there and not on a box.

The reference's charter 6 also returns `finish` for a red the change did not
cause, with that red on the asker's Left list (2026-11-04).
