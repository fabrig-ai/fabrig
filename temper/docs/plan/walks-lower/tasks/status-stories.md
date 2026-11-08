+++
id = "status-stories"
part = "status"
kind = "feature"
needs = []
files = ["temper/tests/client/tests/suite/status.rs", "temper/tests/client/src/front/status.rs"]
effort = "high"
minutes = 120
+++

# Home's brief on a person's plan, and the skip kept across a restart, in the joint world

## Goal

Two process walks of `temper/crates/temper/tests/fake_backends/walk/status.rs`
move to the joint world (`temper/tests/client`, package
`temper-client-world`): a new story holds what
`a_plans_project_has_its_brief_on_a_return_past_the_sitting` proves, and
the existing `a_restart_keeps_the_rows` grows what
`home_on_the_fake_box_keeps_a_landing_count_after_box_and_client_restarts`
adds to it. The walks themselves are `walks-go`'s; you never touch them.

Acceptance:

- **`a_plans_project_has_its_brief_on_a_return_past_the_sitting`**, new,
  in `temper/tests/client/tests/suite/status.rs`, in this order:
  1. The person's only account is a ChatGPT plan made by a sign-in
     (`SignedIn` through the joined engine, as the plan fixtures do), and
     the engine keeps no deployment account: the person's account facts
     read one party account, provider 1, one source of kind sign-in,
     usable, and no deployment account. The person is project 1's only
     member.
  2. Home opened before any landing shows no brief (`Facts::brief` none,
     and the view has no "Written by the status agent").
  3. A change lands for real (the forge's CI, review and landing path, as
     `Story::land` drives it), counted once.
  4. Home entered again within the sitting (less than `forge.sitting`
     after the last look; the existing stories use 29 minutes) shows no
     brief and starts no status run.
  5. The clock moves past the sitting (31 minutes after the last look:
     the story's `after`, never a sleep). Home entered again: the forge
     asks for a status run, the root makes its procedure task, the run
     is admitted, paid for by the person's plan, and its worker is
     assigned (observe the assignment; never seed it). The story answers
     that worker's run as a scripted worker does, with the brief's
     report; the production procedure saves and publishes it. Home shows
     the brief's sentence and lede (the walk's: "1 change landed on the
     local forge." and "The README now records the local status
     change.") and "Written by the status agent".
  6. The plan is still the only account and still usable.
- The story runs on the **stock's budgets** (`temper_service::stock`, the
  project's period 81 × 24K beyond the owner's pool of 80 × 24K): build
  its engine configuration with `temper_service::stock::with_models`
  keeping the stock's policies, as
  `temper/tests/engine/domain/tests/suite/forge_routes/status.rs` does
  for its funded case, never a policy of the story's own.
- **Red first:** with `let period_spend = chat_spend.checked_mul(81)?;` in
  `temper_service::stock::checked_with`
  (`temper/crates/temper-service/src/stock.rs`, about line 379) changed by
  hand to `80`, the new story fails at step 5 (no brief, the status run
  refused or never admitted) for that reason; restore it and it passes.
  The report quotes the failure seen.
- **`a_restart_keeps_the_rows`**, extended in place (keep its name): the
  person presses Skip on Home's "To begin" before the landing, as a
  person does (the act `Act::SkipToBegin` through the client, as
  `a_skip_sends_it` sends it); after the engine's restart and the fresh
  client's return, `Facts::to_begin()` is false and the view has no Skip
  button, and the view says "One change landed since you last looked.
  Nothing needs you." (the walk's words), with "Landing: " and without
  "nothing has landed yet". The rows and count checks it has stay.
  Bite: drop the dismissal row from the restarted engine's store by hand
  (or skip the Skip press) and see the new assertion fail; restore.
- Clippy clean, fmt clean, `temper-client-world`'s focused suite green.

## Inputs

- `temper/docs/plan/walks-lower/notes/common.md` (read first).
- `temper/docs/plan/walks-lower/inputs/request.md`, the two `status::`
  rulings.
- `temper/docs/design/domain/testing.md`, 2.6, the bullet **The fake box
  keeps two walks**, its first two items; and 6.3 (the joint world).
- `temper/docs/design/domain/forge.md`, 5.6 (*What changed since a person
  last looked*: the sitting, `ask_run`, **What funds it**).
- `temper/docs/design/protocol/configuration.md`, 13.4 (the stock's
  policy 1 row: 80 × 24K for role 0, 81 × 24K for the project).
- The walks being lowered, for their words and order:
  `temper/crates/temper/tests/fake_backends/walk/status.rs` (read only).

What the survey found, for your bearings (verify before relying):

- `Story::with_landing`, `Story::land`, `Joint::with_status_landing`,
  `begin_status_landing`, `approve_status_landing`
  (`temper/tests/client/src/front/status.rs`) make a real landing; but
  that fixture restricts delegation to charters 1 and 4 and procedure 2,
  so it can't run the status procedure (charter 9) as it stands.
- `account_acts::empty_plan_fixture`, `signing`, `event`, `settle`
  (`temper/tests/client/src/front/account_acts.rs`) and
  `suite::accounts::a_sign_in_started_answer_is_followed_by_its_actual_end_on_the_accounts_watch`
  show a sign-in's `Visited` and `SignedIn` issuer events and the
  account they make. Those fixtures replace the stock charters.
- `Joint::with_engine(seed, limits, store, configs, engine_limits, slots)`
  (`temper/tests/client/src/front.rs`) accepts a composed configuration.
- The brief's decision: `temper-engine-domain-forge`'s `status.rs`
  (`ask_run` at a Home look past the sitting; no timer starts one); the
  root's `start_status_run`
  (`temper/crates/temper-engine-domain/src/route/forge_route/status_route.rs`)
  makes the procedure; its report comes back through `finish` as
  `StatusHeard`. Worker assignments are visible through `Joint`
  (`hosted_attempt`, assignments) and answered as
  `fixtures::report_task` answers one.
- The plan pays only while its holder is the project's only member
  (`jig/crates/jig-domain-core/src/runs/coverage.rs`).
- The refusal story's seeded `landed: 1` and exhausted period are its own
  oracle: don't copy them here.

## Owns and may change

Owns:

- `temper/tests/client/tests/suite/status.rs`: the new story, its private
  helpers, and the extension of `a_restart_keeps_the_rows`.
- `temper/tests/client/src/front/status.rs`: a new constructor appended
  (for instance `Joint::with_plan_status(seed, limits)`: the stock's
  charters, policies and delegation with the status procedure, a plan
  account and no deployment key, the forge stand that `with_status_landing`
  lands on), if the story can't compose it in the suite file. Add, never
  reshape what is there: another draft also edits this tree.

Reader and neighbour files (edits allowed as stated):

- `temper/tests/client/src/front.rs`, `src/front/account_acts.rs`,
  `src/fixtures.rs`: only a `pub(crate)`→`pub` widening or one new small
  function appended at the end, if the constructor needs it; named in the
  report.

Pre-approved options:

- **Composing the fixture:** if `with_status_landing`'s stand and an
  account fixture can't be joined in the suite file, add the constructor
  above; if neither can make the status procedure run with the stock's
  budgets and the plan without a stub, write what holds (steps 1 to 4 and
  6), and report step 5 under `## Left` with what the world lacks: the
  walk then stays for it.
- **The brief's report words:** use the walk's (`LEDE` and the status
  charter's sentence, `temper/crates/temper-fake/src/backends.rs`), or
  any sentence and lede: the story checks they are shown as the brief.

## Steps

1. Read the inputs, then `suite/status.rs` whole and
   `temper/tests/engine/domain/tests/suite/forge_routes/status.rs`'s
   funded fixture.
2. Write the new story, failing first where it can: run it alone,
   `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(=status::a_plans_project_has_its_brief_on_a_return_past_the_sitting)'`.
3. The red: change 81 to 80 as above, run it alone, see it fail for the
   brief's funding, restore, run it green.
4. Extend `a_restart_keeps_the_rows`; run it alone; bite it as above.
5. Once, at the end: `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-world` (check the count). Commit
   (`walks-lower: Home's plan brief and its kept skip in the joint
   world`, the body saying why: the process walks' behaviours belong to
   the joint world, where the sitting is the world's clock) and push.

## Report

As the implementer's text says: the first line; the commit pushed; each
run with its count; the red seen with 80 (its failure line) and the bite
on the restart story; the composed fixture and any harness line you
added; each keep held, or under `## Left` with what the world lacks; the
two stories' CPU seconds as nextest reports them; the minutes.
