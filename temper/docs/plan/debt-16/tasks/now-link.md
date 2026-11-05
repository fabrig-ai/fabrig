+++
id = "now-link"
part = "now-link"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-domain-core/src/routing/**", "jig/crates/jig-domain-core/src/tests.rs", "jig/crates/jig-domain-core/src/tests/**", "jig/crates/jig-domain-hub-link/src/down.rs", "jig/crates/jig-domain-hub-link/src/tests.rs", "jig/crates/jig-protocol-store/src/ended.rs", "jig/examples/ops/domain/src/route.rs", "jig/testing/jig-test-domain/src/routes.rs", "jig/testing/jig-test-store/src/route.rs", "jig/testing/jig-test-service/src/route.rs", "jig/testing/jig-test-service/src/clients/translation.rs", "jig/tests/core/src/client_peer.rs", "jig/tests/core/tests/suite/procedure_steps.rs", "jig/tests/ops/src/joint/engine.rs", "jig/tests/client/src/joint/engine.rs", "temper/crates/temper-engine-domain/src/route/now.rs", "temper/crates/temper-engine-domain/src/tests/budget.rs"]
effort = "high"
minutes = 80
+++

# jig: the six link outputs nest in `Now::Link`

## Goal

Finding 3. `hub_link::now` costs more than it saves: it takes a whole `Now`, maps six variants and returns the other 35
through a list of every name, and each root then matches the rest. The core nests the six in `Now::Link(LinkNow)`, and
`hub_link::now` takes a `LinkNow` and can't fail.

This task is one atomic change: adding the variant breaks every exhaustive reader, so no halfway state compiles. It's a
change to a type in this workspace, and this cut judged it no public break (`temper/docs/plan/debt-16/plan.md`), because
nothing outside the workspace can read it and nothing stored or on a wire moves. If you find a stored or wire form, or
a golden text that prints a `Now`, that moves, stop and ask.

Acceptance:

- `rg -n '\bNow::(Relayed|DropPayload|TurnPayload|AnswerPayload|AcceptedTurn|RefusedPayload)\b' jig temper --glob '*.rs'`
  finds nothing.
- `hub_link::now` is `pub fn now(value: core::LinkNow) -> Event`, with no `Err` and no `clippy::result_large_err`.
- `hub_link::Event` keeps its six variants: temper and the roots build them directly.
- `core_now_translates_the_links_six_outputs_without_changing_fields` passes, reading `LinkNow`.
- `core_now_returns_a_roots_output_without_consuming_its_bytes` is gone with the `Err` it tested.
- Production net about -46 across jig and temper; a result worse than -30 is worth a line in the report.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `jig/docs/design/domain/root.md`, 5 (this cut wrote the sentence you build) and 3.
- `jig/docs/design/domain/engine.md`, 3.

## Owns and may change

- **Owns:** `jig-domain-core` (`vocabulary.rs`, `lib.rs`, `routing/children.rs`, `routing/task/terminals.rs`,
  `routing/task/refused.rs`, `routing/run/payloads.rs`); `jig-domain-hub-link/src/down.rs`.
- **Readers that collapse to a `Now::Link(..)` arm:** `jig/crates/jig-protocol-store/src/ended.rs` (two lists),
  `jig/testing/jig-test-store/src/route.rs`, `jig/testing/jig-test-service/src/clients/translation.rs`,
  `jig/tests/core/src/client_peer.rs`, `jig/tests/ops/src/joint/engine.rs`, `jig/tests/client/src/joint/engine.rs`,
  `jig/crates/jig-domain-core/src/tests/runs/completion_retry.rs`.
- **Readers of one variant, spelling only:** `jig-domain-core/src/tests.rs`, `src/tests/reads/activity.rs`,
  `src/tests/routing/flight.rs`, `src/tests/runs/run_choice.rs`, `jig/tests/core/tests/suite/procedure_steps.rs`,
  `jig-domain-hub-link/src/tests.rs`.
- **The other tree, the lines that name the variants only:** `temper/crates/temper-engine-domain/src/route/now.rs`,
  `temper/crates/temper-engine-domain/src/tests/budget.rs`.
- **The one approved test deletion:** `core_now_returns_a_roots_output_without_consuming_its_bytes`, with anything only it
  used. No other assertion moves.
- **Options:** if `#[expect(clippy::too_many_lines, ..)]` on `core_now` in `jig/examples/ops/domain/src/route.rs` goes
  unfulfilled, remove it. If `LinkNow` trips `clippy::large_enum_variant`, take a scoped expectation.

## Steps

1. `vocabulary.rs`: the six variants move, unchanged and with their doc lines, into:

   ```rust
   /// The core's outputs that are the link's (domain/root.md, 5).
   #[derive(Debug)]
   pub enum LinkNow { /* Relayed, DropPayload, TurnPayload, AnswerPayload, AcceptedTurn, RefusedPayload */ }
   ```

   `Now` gains `Link(LinkNow)`. `LinkNow` is exported where `Now` is. The 13 construction sites in `routing/` wrap theirs.

2. `down.rs`: `now` has six arms and returns `Event`.

3. The roots:

   ```rust
   // jig/examples/ops/domain/src/route.rs, core_now; its dead arm in route_now goes
   core::Now::Link(value) => {
       crate::link::route(domain, env, decision, jig_domain_hub_link::now(value), false, work);
   }

   // jig/testing/jig-test-domain/src/routes.rs keeps its AnswerPayload case; its dead arm goes
   core::Now::Link(core::LinkNow::AnswerPayload { run, attempt, payload }) => {
       crate::link::answer(domain, run, attempt, payload, work);
   }
   core::Now::Link(value) => crate::link::route(domain, env.now, jig_domain_hub_link::now(value), work),

   // temper's route/now.rs: its six arms, bodies unchanged, in a private fn
   core::Now::Link(value) => link_now(domain, value),
   ```

4. **A trap:** `jig/testing/jig-test-service/src/route.rs` doesn't treat the six alike: `Relayed` goes to
   `stop::unavailable` and the other five to `stop::invalid_restore`. Keep the split with nested patterns. A single
   `Now::Link(_)` arm there changes behaviour.

5. Break by hand: swap `run` and `attempt` in `now`'s `TurnPayload` arm and see
   `core_now_translates_the_links_six_outputs_without_changing_fields` fail. Restore.

6. Tests, `TEMPER_BUNDLE=none` as the machines set it: `heavy cargo nextest run -p jig-domain-core -p jig-domain-hub-link
   -p jig-protocol-store -p jig-ops-domain -p jig-test-domain -p jig-test-store -p jig-test-service`, then `heavy cargo
   nextest run -p jig-core-world -p jig-ops-world -p jig-client-world -p jig-conformance-world`, then `heavy cargo nextest
   run -p temper-engine-domain -p temper-engine-domain-world`, then `heavy cargo check --tests -p temper-service -p
   temper-system-world`.

7. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p jig-domain-core -p jig-domain-hub-link -p jig-protocol-store -p jig-ops-domain -p jig-test-domain -p jig-test-store
   -p jig-test-service -p temper-engine-domain --all-targets -- -D warnings`; the citations check. Commit and push.

## Report

The common note's items, with the break above, the 13 sites confirmed, and the temper lines changed.
