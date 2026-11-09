+++
id = "writer-rows"
part = "writer-rows"
kind = "feature"
needs = ["link-refusal"]
files = ["jig/tests/core/tests/suite/named_people.rs", "jig/tests/core/tests/suite/main.rs", "jig/crates/jig-domain-core/src/admission/**", "jig/crates/jig-domain-core/src/acts/**", "jig/crates/jig-domain-core/src/tests/admission/**", "jig/crates/jig-domain-core/src/runs/delegation.rs", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 180
+++

## Goal

The second candidate family for the deployment's dangling link: a
writer that saves a task row naming a person the people child does not
hold, which needs no kill and fails every later start. The request has
"person 102 a person task": 102 may be a task's number saved where a
person's goes. Each route by which a person's number reaches a task row
from outside the core gets two rows: a person the core holds is written
and linked at the next start; a number it does not hold (a live person
task's number, the deployment's shape) is refused where it is admitted
and never written (`jig/docs/design/domain/engine.md`, 5.1, *Nothing
links ahead of what it names*). A route that writes it is the cause,
fixed in its admission (request item 2).

Acceptance:

- The table of Steps, 1 exists with a row pair per route Steps, 1
  names, and passes. A row that failed on the trunk was committed
  first, alone, its failure quoted in the report (after `link-refusal`
  the start refuses `Failure::Link` naming the task and the person),
  and its fix followed. Each that passed rules its route out and stays
  as a guard. The report says which, per route.
- After every row, the next start reaches `Phase::Open` with
  `start_failure()` `None`, and no saved row names the absent person.
- A refused input is refused with the route's own refusal for an
  unknown party, saying which, and changes nothing.
- The **Not yet** entry `5.1: *Nothing links ahead of what it names* ...`
  in `jig/docs/design/domain/engine.status.md` is removed, its lines
  alone. That departs from the implementer's text, which closes
  `docs/design/` to you: the entry records this task's gap.

## Inputs

- `jig/docs/design/domain/engine.md`, 5.1 (*Nothing links ahead of what
  it names*) and 6 (*It refuses a link it cannot make*).
- `jig/docs/design/domain/tasks.md`, 5.4 (person tasks).
- `jig/docs/plan/store-crash-safe/notes/contracts.md`, 1 and 2;
  `jig/docs/plan/store-crash-safe/notes/common.md`.
- `jig/docs/plan/store-crash-safe/inputs/request.md`, item 2.

## Owns and may change

- Owns the files in `files`; `main.rs` gains the `mod named_people;`
  line alone.
- Pre-approved:
  - **The refusal's words.** A route that admits an unknown person gets
    the refusal it already has for an unknown party
    (`people::Refusal::Unknown`, `tasks::Refusal::Unknown`, a batch's
    own unknown refusal; `admission/amendments.rs` and
    `admission/policy.rs` show the forms). If a route has none and one
    would add a variant to a refusal that crosses to the client's wire
    or the store, that is a `Question:`, with the failing row pushed.
  - **A fix in the tasks child** (`jig-domain-tasks`, where a batch's
    tasks are admitted): its admission files and its unit tests are
    yours for that fix; name them.
  - **A route the core cannot reach in its world** (an application's
    request with no fixture): its row in `jig/crates/jig-domain-core/src/tests/admission/`,
    as a unit test of the admission, and say so.
  - **The rows' scenarios** reuse the world's stories (`Setup`,
    `Boot::Caller`, `procedure_seeds.rs`'s batches, `person_task.rs`'s
    world); a fixture helper they lack goes in `named_people.rs`, not in
    `jig-core-kit` (`crash-rows` owns the kit's world).
- Not yours: `jig/crates/jig-domain-core/src/start/**` (`link-refusal`'s,
  merged before you), `jig/tests/core/tests/suite/faults.rs` and the
  kit (`crash-rows`), every figure.

## Steps

1. **The rows,** a table test in `jig/tests/core/tests/suite/named_people.rs`,
   `a_person_named_from_outside_is_held_or_refused_and_the_start_links`,
   rows `(route, named)` with `named` either a person the world holds or
   the number of a live person task the world made (not a person). Find
   each route by where a task row's person fields get their value from
   an input: `rg 'PersonAddress::Person|Party::Person|Funder::Pool|taken_by|EscalationHolder::Person|ProposalHolder::Person' jig/crates/jig-domain-core/src/{admission,acts,routing}`.
   The cut's list, each a row pair:
   - a procedure's or connector's batch whose task's executor is a
     person (`acts/bounds.rs`, `acts/objects/task.rs`,
     `admission/compound.rs`);
   - a batch whose task's funder is a person's pool, and one whose
     requester is a person;
   - an agent's ask that makes a person task for a named person;
   - a proposal held by a named person (`admission/proposals.rs`);
   - an escalation to a named person (`admission/escalation.rs`);
   - a role's holding naming a person (`admission/roles.rs`);
   - an amendment adding a named person (`admission/amendments.rs`).
   A route you find beyond these is a row pair too, named in the
   report. Each row: drive the input, settle, restart
   (`world.restart(..)`), and assert the acceptance. On the trunk with
   `link-refusal` merged, a route that writes the absent number makes
   its row's restart refuse `Failure::Link`.
2. **A failing row:** commit it first, alone, with its failure; then the
   fix in that route's admission: the input naming a person the people
   child does not hold (`core.people.identity(n)` is `None`; a live
   task's number is not a person) is refused before any child changes.
   Break the fix once by hand, see the row fail for that reason,
   restore, and name it in the report.
3. **Runs.** A fix changes what admission accepts, which every route's
   readers see: `heavy cargo nextest run -p jig-domain-core --lib`,
   `heavy cargo nextest run -p jig-core-world`, and, since temper's
   forge procedures send batches through these routes,
   `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/forge_routes::/) | test(/root_routes::/)'`
   and `heavy cargo check -p temper-engine-domain --tests`. With no fix,
   only the first two. Check each run's count.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-core-world -p jig-domain-core --all-targets -- -D warnings`
   (adding `jig-domain-tasks` if you fixed it); remove the status
   entry; commit saying why (no route writes a link to a person the core
   does not hold, so a start finds every link); the citations check;
   push.

## Report

`Done:` / `Failed:` / `Question:` first. Then: per route, ruled in or
out and by which row, with the failure quoted for one ruled in; the
fix and the refusal it answers; the hand-broken rule; the runs and their
counts; your implementation and check minutes apart.

## Answers

2026-11-09. *May writer-rows edit the admission methods in
`jig/crates/jig-domain-core/src/runs/delegation.rs`, or should acceptance
permit the existing `Executor` refusal for an unknown person?* Edit it:
the file is now in `files`, and no other task of this plan owns it. The
acceptance stands: a number the people child does not hold
(`self.people.identity(n)` is `None`) is refused `tasks::Refusal::Unknown`,
checked in `Core::delegate_preflight` per member before
`delegation_executor`, so nothing changes. `Executor` stays the refusal for
a held person with no role in the task's project, and for the other
executor kinds it already answers. `procedure_batch_admit` (same file)
takes the same check only if its route's row, committed first, shows it
writing or refusing other than Unknown. The other routes of Steps, 1 still
each need their row pair before the status entry goes.
