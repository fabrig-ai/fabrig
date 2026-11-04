+++
id = "init-standard"
part = "box"
kind = "feature"
needs = ["standard-scale", "forge-profile", "smith-calls", "client-capacities"]
files = ["temper/crates/temper-init/template/temper.json", "temper/crates/temper-init/template/agent.json", "temper/docs/examples/owned.json", "temper/docs/examples/owned/agent.json", "temper/crates/temper-init/src/tests/**", "temper/crates/temper/tests/suite/startup.rs", "temper/crates/temper/tests/suite/configuration.rs", "temper/crates/temper/tests/support/owned.rs", "temper/crates/temper-service/src/tests/startup.rs", "temper/crates/temper-service/src/tests/stock_brief.rs", "temper/tests/real-loop/tests/suite/template_bounds.rs", "temper/tests/engine/domain/src/forge_routes.rs", "temper/docs/design/protocol/configuration.status.md", "temper/docs/design/protocol/setup.status.md", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/tests/checks.rs", "temper/docs/design/protocol/configuration.md", "temper/crates/temper-engine-domain/src/limits.rs", "temper/crates/temper/tests/support/forge_land/rung.rs", "temper/crates/temper-service/src/tests/welcome_policy.rs", "temper/tests/engine/domain/src/store.rs", "temper/tests/engine/domain/tests/suite/forge_routes/b2_tree.rs", "temper/**/tests/**", "temper/**/src/tests/**", "temper/**/fixtures/**", "temper/**/golden/**", "temper/**/testdata/**"]
effort = "high"
minutes = 240
+++

# A box from `temper init` takes the `standard` profiles and an agent of fabrig's scale

## Goal

`temper init`'s document and `docs/examples/owned.json` name the `standard` profiles on all
three `"profile"` lines, their overrides saying only what differs from them, and their agents
take 800 turns, 48 calls per response, 26 conversations and the largest tool payload init's
models admit, with T and its carriers derived from that agent (contracts K1, K5, K8's window;
`temper/docs/design/protocol/configuration.md`, 13.4 and 13.8; `setup.md`, 8.1 and 8.3). After
this task a fresh box holds a project of fabrig's scale but for the stock's settings
(`brief-room`, `stock-scale`).

Acceptance:

- **The agent** (`temper-init/template/agent.json`): `declared` `tool_payload` the largest smith
  admits at init's models, 10,662 unless `smith-calls` reported another figure (its report, or
  its test's comment), `conversations` 26, `calls_per_response` 48; `policy` `max_turns` 800
  and `inbox` 82, jig's `standard` `tasks.run_inbox`, which the fleet holds a worker's agent's
  inbox to at its hello (`jig-domain-core/src/settings/fleet_needs.rs`, the start's messages)
  and temper's start checks the hub's against (`temper-service/src/startup/checks.rs`,
  `Unfit::Inbox`; `standard`'s `hub.inbox` is 82); the `demo` model's `window` 1,048,576 (K8).
  If smith's derivation or a check refuses an inbox of 82, stop and ask with its words. `docs/examples/owned/agent.json`: the same
  figures, its endpoints and programs its own (`calls_per_response` 48 too).
- **The document** (`temper-init/template/temper.json`): the three `"profile"` lines
  `"standard"`; the `demo` model's `window` in `engine.llm.models` 1,048,576; the overrides:
  - engine: `documents.rows` 2, `documents.text_bytes` 128; `tasks.message_bytes` and
    `tasks.result_bytes` the agent's advertised message bytes, `tool_payload` − 2 (K5); the
    machine's concurrency where jig's `standard` sets another than `small`'s (compare
    `profile/standard.rs` with `profile/small.rs` on `hub.slots`, `fleet.slots`,
    `fleet.workers`, `fleet.configured_workers`, `clients.streams`, `clients_http.streams`):
    small's figure, but the clients' streams 8 with `clients.exchanges` and `clients_http.calls`
    12 as init has them (configuration.md, 13.4); T's carriers (`core.resume_bytes`,
    `hub.turn_bytes`, `hub.transcript_bytes`, `hub.turn_queue_bytes`); nothing else. Every
    override today equal to or covered by `standard` goes (the grants, segments, notes' scopes,
    executors, charters, contract choices, parameters, `documents.lines`, watchers,
    `tasks.batch`, `tasks.authority_bytes`, `tasks.inbox_bytes`);
  - worker: `hub.accounts` 4, `hub.event_bytes` the agent's message bytes (`tool_payload` − 2),
    the hub's concurrency kept as above, T's carriers; nothing else;
  - application: `checkout.message_bytes` the agent's `tool_payload`; `forge.repositories` and
    `forge.client.repositories` 3; `forge.output` 82 where temper's `standard` has another; T's
    root carriers; nothing else (`routes.query_bytes` 2,048 is the profile's and goes).
  - **Pre-approved:** where a start check of `standard` refuses init's document (a relation, a
    carrier of the agent's call, a store bound), add or raise that override to the least that
    passes and list it in the report with the check's words; never lower a figure of the
    contracts' tables.
- `docs/examples/owned.json`: the same profiles and overrides, in lockstep (its known
  differences from init's stay: `the_default_is_the_owned_example_so_edited`'s list).
- **T and its carriers** rewritten by the writer, never by hand:
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper --test suite -E 'test(startup::turn_bounds)'`,
  for both documents; then the test passes without it.
- **The readers of init's documents**, each edited only in the figures that moved or the lines
  that named a dropped override:
  - `temper-init/src/tests/documents.rs` (`documents.lines` 16 near 34 and still ≥ the charters
    and procedures; streams near 43; `conversations` 26 near 363; the demo's window near
    349-357; the owned example's expected overrides near 131-150) and `sub_agents.rs`;
  - `temper/crates/temper/tests/suite/startup.rs` (`checked_in_deployment_profiles_pass_startup_admission`,
    the page against T near 1396, the refusal naming T near 1610);
    `temper/crates/temper/tests/suite/configuration.rs` (its owned overrides JSON near 80; its
    capacity assertions near 40-66 hold against the shells' new capacities);
    `temper/crates/temper/tests/support/owned.rs` (near 509-511, 716, 786-787);
  - `temper-service/src/tests/stock_brief.rs`: its pinned host part, room, offer and T (near
    75-122, 185-241, 339) at init's documents, as the code now derives them;
  - `temper-service/src/tests/startup.rs`, `capacity_overrides_name_the_failed_dependency`: each
    row's value moved so it still breaks its relation under `standard` (for one,
    `tasks.tasks` past half of `people.inbox_tasks` 256; `tasks.delegates` 81;
    `tasks.message_bytes` past the profile's `tasks.inbox_bytes`; `authority.grants` past
    `notes.scopes` − 3), its expected words with them; the two `tasks.batch` rows: one past
    tasks-v9's new batch ceiling, and the store-row row dropped if no batch within the ceiling
    passes the profile's store (say so);
  - `temper/tests/real-loop/tests/suite/template_bounds.rs` (T near 11);
  - `temper/tests/engine/domain/src/forge_routes.rs`, `box_limits()` (near 2455): temper's
    application profile is `standard` (`Profile::Standard`), as init's document names it.
- Status: `configuration.status.md`'s first **Not yet** entry keeps only what other tasks
  close (the stock's and the reference's clauses); its clauses on init's document, the owned
  example, their agents, T and the transcript ceilings go. `setup.status.md`'s 8.1 and 8.3
  entry keeps only its resource gate's clause.
- `heavy cargo nextest run -p temper-init`, `heavy cargo nextest run -p temper --test suite -E
  'test(/startup::|configuration::/)'`, `-p temper-service`, `-p temper-real-loop-world` (find
  the package of `temper/tests/real-loop`), `-p temper-engine-domain-world`, and `heavy cargo
  check -p temper --tests` pass.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K1, K2, K5 and K8
  (its first bullet).
- `temper/docs/design/protocol/configuration.md`, 13.4 ("The box's limits for that work"), 13.8
  (T, the store's ceiling, one call's payload).
- `temper/docs/design/protocol/setup.md`, 8.1 (the agent's configuration, `engine.limits`, the
  worker's `limits`, the turn's bounds), 8.3 (the demo's window, its limits).

## Owns and may change

- The files of the header, for what acceptance names. In `temper.json`, only the profile lines,
  the overrides and the demo model's window; nothing of its forges, demo scripts or workers but
  their `limits`.
- **A change of meaning:** every world that starts from init's documents now runs `standard`.
  Run the packages above; a test whose figure was small's changes its expected figure only,
  listed in the report; a test that fails otherwise is a question.
- `temper-service/src/limits.rs`, `stock.rs` and jig's profiles are not yours: a figure there
  that refuses init's document is a question, with the check's words.
- `temper/docs/examples/reference/` is not yours (`reference-trim`).

## Steps

1. The agent files; `smith-calls`' figure for the payload.
2. The document's profiles and overrides; the start's admission, overrides added where it asks.
3. The writer for T; the owned example in lockstep.
4. The readers, one package at a time.
5. The status lines; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-init -p temper -p temper-service -p temper-engine-domain-world --all-targets --
   -D warnings`; the citations check; commit and push.

## Report

As the implementer's text asks, with T, the payload, and every override added or kept beyond
the list above, each with why.

## Answers

2026-11-03. **`engine.limits.io.sockets: 106 bytes, past its bound of 64` under `standard`: who
raises `limits::SOCKETS`?** This task does; it is widened to that one figure. The count grows
with jig's `standard` (its OAuth driver's accounts, exchanges and listeners, sized for 16
accounts and 4 sign-ins), not with any override of init's document, and no task after this one
can unblock it (`forge-profile` has landed; `brief-room` needs this task). The sockets are
temper's own constant, which `configuration.md`, 7 says holds the owned example's document under
`small`; under `standard` it must hold it there.

- `temper/crates/temper-service/src/limits.rs`: `SOCKETS` 128 (the owned example's 106 under
  `standard`, with room for a webhook's hooks as the owned example has); its doc comment says it
  holds `standard`'s owned example with both listeners and a sign-in. Nothing else in the file.
- `temper/crates/temper-service/src/tests/checks.rs`: `one_ios_entities_hold_both_services_and_their_children`
  and `hook_connections_past_the_process_sockets_are_refused` keep their meaning: raise their
  overrides (`git_component.operations`, `forge_component.hooks`) so the count passes 128, and
  their expected words with them (`..., past its bound of 128`). If a raised figure trips another
  bound first, pick the override that reaches past 128 alone.
- `temper/docs/design/protocol/configuration.md`, 7 (the webhooks' listener's bullet): "and
  `small`'s hold a document ..." becomes "and hold, under `standard`, a document ...", the rest of
  the sentence as it is. Nothing else in the document.

**The other refusal, `engine.limits.store: the store's map sizing is past its codec or arithmetic
bounds` (`StoreMap(Ceiling)`)**, is this task's to diagnose once the sockets pass: find which step
returns it (`limits::engine` returning `None`, or `store::layout`'s `SizeRefusal::Ceiling`) and on
which figure. If an override of init's document causes it or can mend it (a carrier past a codec
ceiling, a store bound), the pre-approval above covers it: the least that passes, never lowering a
contracts' figure, listed in the report. If the cause is a figure in `limits.rs` beyond `SOCKETS`,
jig's profile or a codec ceiling, stop and ask with the function, the figure and the bound it
passes.

2026-11-03. **`StoreMap(Ceiling)`: widen core-v6's `EscalationDecisionReject.reason` past 8,192, or
change temper's projection?** temper's projection; no format changes. The core's
`escalation_reason_bytes` bounds "an ended person escalation rejection reason"
(`jig-domain-core/src/vocabulary.rs`): a person's words, which people-v14 already holds to
`people.words` (`codec/people.rs`, 8,192 in `standard`, contracts K2: "the most without a format
change"). A reason past it can never arrive, so a wider core-v6 ceiling would store room no person
fills. This task is widened to one line:

- `temper/crates/temper-engine-domain/src/limits.rs`, `core_limits`: `escalation_reason_bytes`
  also takes `.min(limits.people.words)`, beside the result and journal bounds it takes now.
  Nothing else in the file. jig's `codec/core.rs` then derives `max(8,192, 8,192)`, within
  core-v6's 8,192. If a test of `temper-engine-domain` pins the old figure, change its figure
  only and list it.

**The root's validity (`limits::valid` false) is init's document's,** under the pre-approval
above; list each with the check's words:

- `forge.client.op_bytes < tasks.result_bytes` (512 < 10,660): the application's override
  `forge.client.op_bytes` 10,660, the result bound. If it moves `forge_component.body_bytes` or
  another start check, the least that passes, listed.
- `root.journal.transcript_bytes` below `row_bound` (the row holds `standard`'s
  `tasks.inbox_bytes` 21,495,808, 82 messages of 262,144): keep the engine override
  `tasks.inbox_bytes` at `tasks.inbox_messages` × the box's message bound, 82 × 10,660 = 874,120
  (the inbox's bytes follow the box's message bound as its messages do; K2 couples them, the
  table does not name it, and jig requires only `message_bytes ≤ inbox_bytes`). The journal
  carrier stays T's, as the writer writes it. If the row then still passes it, stop and ask with
  `row_bound`'s figure and its largest terms; do not raise the carrier by hand.

Then rerun `startup::checked_in_deployment_profiles_pass_startup_admission`, both documents in
lockstep, and carry on with the acceptance. Another refusal from `limits.rs` beyond `SOCKETS`,
jig's profile or a codec ceiling: stop and ask as before.

2026-11-03. **B0's fixed carriers in `temper/crates/temper/tests/support/forge_land/rung.rs` no
longer hold its agent under `standard`: may this task repair them?** Yes; the task is widened to
that file's carriers. `startup::rung_b0s_live_configuration_is_admitted_without_an_account_or_backend`
is in this task's required suite and fails only because init's agent (800 turns, 26
conversations) now gives a larger T (2,000,788 at B0's model and one call), so the acceptance
cannot hold without it. `stock-scale` owns the file's funding step (near 560-576), not this.

- In `deployment` and `turn_limits`: after B0's edits to the agent (the model, its output and
  reasoning item, `calls_per_response` 1), derive the carriers from that agent with the shared
  helper, `support::configuration::turn_bounds::apply(&mut value, turn_bounds::turn_body(&worker))`
  (the writer's own arithmetic, `temper/tests/real-loop/src/turn_bounds.rs`), in place of the
  hand figures (`hub.turn_bytes`, `hub.transcript_bytes`, `hub.turn_queue_bytes`,
  `core.resume_bytes`, the `root.journal` and `root.loads` byte carriers). Keep what is not T's:
  `people.requests` 256, `root.loads.loads` 32, the memory declarations, the model recipe.
- The assertion on `hub.turn_bytes` (near 153) compares with the derived T, not 1,507,539.
- If B0's start then refuses on another figure (a memory declaration, a store bound), the least
  that passes in this fixture, listed in the report with the check's words; a refusal from
  `limits.rs`, jig's profile or a codec ceiling is a question as before. Nothing else in the file.

Then the two moved figures the run listed (`serve_refuses_an_explicit_call_bound_below_the_agents_call_input`
at 10,662, `serve_checks_the_owned_agents_admission_at_each_bound` at 10,660) and the rest of the
acceptance as written.

2026-11-03. **`template_bounds`: select `standard` in the test, or keep `small` with a separate
small agent?** Select `standard`, in `template_bounds.rs` alone. The test is the regression that
init's shipped agent, with T derived from it, fits the store page of the profile init ships: Small's
page was sized for exactly that agent (`7c0134012a`, "Size Small store pages for the shipped
two-call transcript"). init now ships `standard` (59,570,691-byte page), so the test follows it;
Small's own page ceiling stays pinned by `temper-service`'s
`startup_admits_a_frame_exactly_at_the_profile_page_ceiling` and its neighbours, so no separate
small agent is needed.

- In `template_bounds.rs`, after `landing::configure` and before `turn_bounds::apply`, set the
  engine's, each worker's and the application's `limits.profile` to `"standard"`. Leave
  `fixtures.rs`, `landing.rs` and every other real-loop file as they are.
- Rename the test and its module doc to what it checks now: init's shipped agent fits
  `standard`'s page with derived carriers (for example
  `the_shipped_agent_fits_standards_page_with_derived_carriers`; the agent is no longer
  two-call). Its `expect` words follow.
- If `standard` then refuses the fixture on another check (a relation, a carrier, a store bound),
  the pre-approval above holds, in this test alone: the least override that passes, listed in the
  report with the check's words. A refusal from `limits.rs`, jig's profile or a codec ceiling is a
  question as before.

Then the rest of the acceptance as written.

2026-11-04. **Four runs without landing: rerun, split or stop?** Rerun, as it is. Each run stopped
on a distinct, rightful question (the sockets, the escalation reason, B0's carriers,
`template_bounds`' profile), each is answered above, and each pushed its work: the branch holds the
profiles, agents, overrides, T and every repair but `template_bounds`' profile line. What is left is
that line and the acceptance's checks. To land in one more run:

- Start from the pushed `fabrig-scale/init-standard` (`046211ac6f` or later) and the answers above;
  redo nothing they settle and ask nothing they answer.
- Pre-approved, with no stop: in a file of the header, a test that fails only because init's
  documents now run `standard`, and that mends by an expected figure, by T's carriers derived with
  `turn_bounds::apply` in place of hand figures, or by selecting `standard` where the fixture
  stands for init's box, takes that mend; list each in the report with its old and new figure.
- Still a question: a refusal from `limits.rs` beyond `SOCKETS`, jig's profile or a codec ceiling;
  a failure in a file outside the header; a figure of the contracts' tables that would have to
  fall.
- Run the acceptance's checks in its order, the no-write `startup::turn_bounds` first, then the
  status lines, clippy and the push.

2026-11-04. **`welcome_policy.rs` pins `authority.projects` 2 (Small's); may this task mend it?**
Yes; the task is widened to that one test, `real_init_policy_fits_with_two_long_github_repositories`.
Its acceptance (`-p temper-service` passes) cannot hold without it, and `fresh-box`, which owns the
file, comes four tasks later; the trunk would stay red until then. It fails only because init's
document now runs `standard`, the mend this answer's pre-approval above names.

- The assertion on `limits.authority.projects` takes `standard`'s 8; the comment above
  `policy_bytes` says init runs the `standard` profile.
- The test's capacity proof keeps its meaning: after the two adopted projects, make the projects
  up to `limits.authority.projects` (a plain `MakeProject`, no adoption), then the next one is
  refused `LimitKind::Projects`; its `expect` words and the `eprintln!` line say the profile's
  project capacity instead of "two-project". Do not override `authority.projects` in the test.
- Another figure of that test that moves only for `standard` (`forge.name_bytes`, the joined
  policy's bytes, `policy_bytes`) takes its expected figure, listed with old and new. Nothing else
  in the file: `fresh-box` adds its own tests there.
- The same holds, with no stop, for any other test of `temper-service` (and of the acceptance's
  other packages) that fails only because init's documents run `standard` and mends by an
  expected figure alone: mend it, list it with its old and new figure and the file, and say in the
  report that the file is outside the header. A failure that needs more than an expected figure,
  or a test owned by `reference-trim` (`the_reference_profile_passes_startup_admission`), is left
  under `## Left` or asked as before.

Then the rest of `-p temper-service`, and the acceptance's remaining checks in order.

2026-11-04. **`box_limits()` projects init's engine overrides over a bare application profile: may
it read and apply init's `application.limits.overrides` too?** Yes; the helper is widened to that,
in `forge_routes.rs` alone. It says "Effective box limits, read through the shipped init
template", and init's box is both halves of that template: its application overrides
(`forge.client.op_bytes` 10,660, T's root carriers) are the ones this task's answers above made
init's document carry so the root is valid under `standard`. Without them `limits::engine` refuses
(512 < 10,660; a `root.journal.transcript_bytes` of 65,536 under the row), and the
engine-domain world's 26 failures all come from that one line, so the acceptance cannot hold
without it.

- In `box_limits()`: take `temper_service::limits::profile(Profile::Standard)` (it has `Small`
  on the trunk; `Standard`, as acceptance says), read the template's
  `application.limits.overrides` with the configuration reader `box_profile()` already uses,
  apply each numeric member with `temper_service::limits::apply` (expecting `Ok`, naming the
  member in the panic), then `limits::engine`. Read the document once if that keeps it plain (a
  private helper returning both halves); the signature of `box_limits()` and `box_joins()` stays,
  and nothing becomes public. Copy no carrier arithmetic and no T figure.
- A test of the world that still fails after that only because its figure was `small`'s takes the
  expected figure, listed with old and new. Anything else is a question as before.
- **Leave under `## Left`, without stopping**, the two failures outside this task's reach that
  the run listed: `temper-service/src/tests/limits.rs`, `the_standard_profile_passes_each_domains_validity`
  (the bare `standard` profiles, no init document; `brief-room` owns `limits.rs` and that test),
  and `temper-service/src/tests/room.rs`, `defaults_leaving_2048_bytes_start` (its hand
  run carriers; not an expected figure). Also the reference's, `reference-trim`'s. Run
  `-p temper-service` and `-p temper --test suite` excluding those three by name, and say so;
  every other test of the acceptance's packages passes.

Then the status lines, clippy, the push.

2026-11-04. **B2's real store keeps Small's 4 joins (`store.rs`, `component_limits`): may this task
widen the fixture to Standard's 16?** Yes; the task is widened to that one line of
`temper/tests/engine/domain/src/store.rs`, and `b2_tree.rs`'s expected figures are taken into the
header. B2 is in the engine-domain world this task must pass, its assertion "the real store uses
the box allowance" is the proof and stays as it is, and no task owns the fixture (`stock-scale` and
`fresh-box` come after this one and need it), so the acceptance cannot hold without it.

- In `component_limits`, start from `jig_protocol_config::profile(Profile::Standard)` in place of
  `Profile::Small`, the store profile init ships; keep every override after it as it is
  (budget, key, value, ops, commit, queued bytes, page bytes, page rows 8). Under the fixture's
  overrides and `temper_service::store::limits`' derivation, the only figures that move are
  `joins` (4 to 16) and `held` (16 to 64, Standard's client exchanges × 2); both only admit more.
  No new constructor, no profile parameter threaded through `World`, nothing else in the file.
- In `b2_tree.rs`, the expected figures and the rename the run made stand; the equality with
  `box_joins()` stays.
- Every engine-domain world now runs over that store. A test of
  `temper-engine-domain-world` that then fails only because a join or held figure was Small's
  takes its expected figure, listed with old and new and its file; one that fails otherwise
  (a refusal it relied on at 4 joins or 16 held) is a question, with its words.

Then the full `-p temper-engine-domain-world`, the acceptance's remaining checks with the three
exclusions the answer above names, the status lines, clippy and the push.

2026-11-04 (fabrig/qa via agents/loop, a standing answer). **May this task change a test, fixture,
golden or test helper outside its files to follow init's move to the standard profile?** Yes,
without asking: any test, fixture, golden file or test helper whose expected figures, bounds or
profile follow from init's profile moving to standard may be updated in place (the files above now
take temper's test, fixture and golden trees). Stop and ask only for product code outside the
files, or for a design change. Run the whole of each touched test binary before pushing, so the
next pinned figure is found in this run, not the next.
