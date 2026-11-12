# The reference's configuration

The reference process (`../../draft/reference-code-production.md`,
section 14) is settings over the stock, never a stock of its own: a
deployment runs the stock until an owner applies them, and a release
that changes the stock changes nothing of them. Its charters are
overlays over the stock's that carry the reference's fields alone and no
text (`../../draft/stock-charters.md`, 6), so that its runs read the
stock's words and a release's new stock reaches it with the overlays in
place (jig's `domain/engine/overlays.md`).

- **A settings file,** `docs/examples/reference/settings.json`, in the
  form `temper ctl settings apply` reads (`../../client/ctl/tools.md`, 3;
  `../../client/ctl/overlays.md`, 1). An owner applies it once, after the
  deployment's first start, as a walk on fabrig's box does
  (`../../draft/v0.1.0.md`, 2.1); a later edition is applied the same
  way. It holds:
  - *charter 5's overlay, the goal's agent's:* the reference's model,
    Opus 5.5 as the document configures it, with no alternatives; its
    documents, `AGENTS.md`'s "1. In one page" and the code's rules,
    skein's `../foundation/programming-model.md`'s "1. In one page", on
    the home repository's `main` (`settings.md`, 6), each naming the
    forge's host, owner and repository the walk's document fills in, as
    the landing rules do; the stock's run, 800 turns and 8 hours, a
    reserve of 1 turn and 60 s, and the reference's spend, named so that
    a stock that changes them leaves the reference's; no shell. No
    instructions and no parts: its runs read the stock's charter 5, the
    designer at its first run and the coordinator at each later wake, in
    parts, under the stock's wake policy (`stock.md`;
    `../../draft/stock-charters.md`, 2 and 9);
  - *charter 2's overlay, the implementer's:* gpt-6.1-sol as the document
    configures it, with no alternatives; the same run, reserve and
    spend; smith's shell family (`shell` and `job`: smith's
    `domain/tools.md`). No text: its runs read the stock's charter 2,
    which says what a run does with a shell (`stock.md`);
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
    3.3; `../../client/ctl/tools.md`, 3).

  It sets no template 4's work, kind 11: the stock's, no design and no
  close-out (`stock.md`), holds, since the stock's charter 5 delivers
  its design straight to its goal's branch with no designer's change
  for the path to draw. The reference's own value, design and no
  close-out, was its own charter 5's, which delivered the shared design
  as a designer's change, and went with it.
- **The rules and policy 1** are changed on the box from their current
  values, which hold the deployment's own budgets (`stock.md`): read with
  `temper ctl settings show --json`, charter 5, the merge kind, job 3,
  the goal's closer, procedure 6, the close kind, job 4, and role 0
  added to the rules' and policy 1's delegation where their stored
  values predate the stock's, which holds them (`stock.md`), so its
  goals close, run their merges and ask their person through a person
  task (jig's `domain/people/README.md`, section 7), and jig's control
  family to their tools and role 0's, so the goal's agent may name
  `change`'s plan members (`../../domain/agent/tools.md`, 1), and jig's
  notes family beside it, in the goal's and the project's scopes, so
  that it notes what it learns: in all, tools 1,983 with the decide,
  propose, subscribe and recall families its row names and no effect
  family, and notes 3, as the profile's `../README.md` gives them, then
  applied whole. The widening takes the drafter's charter 12 out of
  that delegation (`stock.md`), since the reference's work starts no
  design session, so that the stock's other fifteen executors, the
  goal's closer and the close kind among them, fill fifteen executor
  places, the stock's delegation holding no foreman's charter 6
  (`stock.md`): the profile's `authority.executors` and
  `tasks.executor_kinds` are 18, which the box takes at its next start,
  leaving room for further work.
- **The stock's words, not the reference's:** the reference keeps no
  text of its own. What its own editions of charters 5 and 2 said went
  with them, and so did its charters 7 and 8, the family designer and
  the triage delegate: the stock's charter 5 designs and cuts its goal
  itself, answers what its changes ask with questions `up`, triages a
  red of its goal's tier, and parks while its goal's closer lands the
  goal and reports (`../../domain/forge/branches.md`, 2); the stock's
  charter 2 checks its spec before it asks and reports what it leaves
  under Left (`stock.md`). Charters 6, 7 and 8 are no settings of the
  reference's, and its delegation names none of them.
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
    policy must fit: the stock's own bound (`stock.md`), kept here so
    that the reference's room holds whatever the stock's profile
    becomes. Its `../README.md` gives the commands. All of it before the
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

  The settings file sizes its overlays and template to the box: charters
  5 and 2's overlays name the stock's run, 800 turns and 8 hours
  (`stock.md`), so that a stock that changes it leaves the reference's;
  template 4's report takes the reference's `tasks.result_bytes`, 63,738;
  charter 5's overlay names two documents, `tasks.charter_documents`,
  which jig's settings schema holds; template 4 names the forge's *its
  own goal branch* (`../../client/ctl/tools.md`, 3). Charter 5 makes its
  changes in `change` calls each within one call's payload,
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
  relations from that agent, and holds the settings file to them,
  applied over the stock at the reference's room: the charters its
  overlays make within `tasks.charters`, their documents within
  `tasks.charter_documents`, their turns within the agent's
  `max_turns`, template 4's report within `tasks.result_bytes`. The same
  test holds each charter its overlays make to what a host takes and a
  box stores (`../../draft/per-wake.md`, section 8): its turns and time,
  and the rules' run spend, within the bounds a stock worker advertises
  from init's agent configuration as `room.json` sets it (jig's
  `domain/engine/fleet.md`), and its stored policy, its parts with it,
  within `room.json`'s `root.journal.run_bytes`, so that an edition no
  host would take, or that a box can't store, is red there and not on a
  box. The tests read the charter a run receives, the stock's words
  under the reference's fields, never an item's text
  (`../../draft/stock-charters.md`, 8).

## 1. Open questions

- **The reference's later runs** (`reference.md`; `../../draft/per-wake.md`,
  sections 10 and 11): compare high with medium after observing runs
  that read their selected parts, and decide whether the cut should
  settle entries. Documents per part, choices per wake kind, parts for
  resumed runs and offering a message with its part remain deferred in
  that draft. Default: four fresh-run parts, xhigh first and high later,
  documents shared by every run, and entries outside a run's parts
  left for the next run.

- **The checks' deadline** (`reference.md`): `.temper/pre-pr` names each check
  before it starts (`tools/pre-pr/design.md`, section 4), so a timeout's
  tail shows where it stopped; with lints and focused tests it builds, and
  the agent's 60-second `check_timeout` is the owner's to raise, separately
  (2026-11-09). Default: 60 seconds, with smith's check path and given time
  in the refusal.

- **Who writes the case's workflows** (`reference.md`). The repository a case runs
  on carries CI and `tier-slow.yml` and `tier-goal.yml` for its landing
  to meet them. Who writes them, and the runner's budget they take, are
  the user's, open since 2026-11-03, with no default: the walk's copy of
  fabrig carries them, written for the box that runs the case, CI
  running the gate's nextest profiles whole, `tier-goal.yml` the gate
  without its slowest suites and `tier-slow.yml` the slow tiers, within
  a runner budget the user sets; or fabrig carries them under
  `.forgejo/workflows/` once that budget is set; or the case runs on a
  smaller repository than fabrig, whose workflows are cheap.
