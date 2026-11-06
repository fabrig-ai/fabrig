+++
id = "refusal-reason"
part = "hello-refusal"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-fleet/src/history.rs",
  "jig/crates/jig-domain-fleet/src/needs.rs",
  "jig/crates/jig-domain-fleet/src/channel.rs",
  "jig/crates/jig-domain-fleet/src/sources.rs",
  "jig/crates/jig-domain-fleet/src/boundary.rs",
  "jig/crates/jig-domain-fleet/src/lib.rs",
  "jig/crates/jig-domain-fleet/src/tests/needs.rs",
  "jig/crates/jig-domain-core/src/vocabulary.rs",
  "jig/crates/jig-domain-core/src/routing/children.rs",
  "jig/crates/jig-domain-core/src/routing/stopping.rs",
  "jig/docs/design/domain/engine.status.md",
]
effort = "medium"
minutes = 35
+++

# The fleet's refusal hands its parent why

## Goal

The first half of the request's item 1. The fleet knows why it refuses a hello, and records it, but its `Request::Refuse { channel }` says nothing, and what each side holds of a shortfall goes with the hello. So nothing above it can name the reason. Carry the reason and the figures up through the fleet's request and the core's `Held::Refuse`. `refusal-words` turns them into words.

Acceptance:

- The shapes in `temper/docs/plan/landing-fixes-2/notes/contracts.md`, section 1, as written.
- A hello refused for a shortfall gives `Request::Refuse { hello: Some(..) }` with the right `why` and figures, for each row of the table, including after a later `Event::Needs` has raised a bound.
- A hello refused for any other recorded reason gives its `why` with `figures: None`. Malformed metadata and a later record's fault give `hello: None`.
- `figures` agrees with `shortfall`: every `why` that `shortfall` returns for a needs and bounds has figures whose `offered` falls short of `required`, or for `Checks` passes it.
- `HelloRefusal`, `HelloRefusalRecord` and every document, wire or stored form are untouched. If this seems to need one changed, stop: that's a question.
- The text sent to a worker is unchanged by this task.

## Inputs

- `jig/docs/design/domain/engine.md`, section 8, **A host's agent must run the engine's charters**.
- `jig/docs/design/protocol/channel.md`, 4.3, **The fleet may refuse the hello**.
- `temper/docs/plan/landing-fixes-2/notes/contracts.md`, section 1.
- `temper/docs/plan/landing-fixes-2/notes/common.md`.

## Owns and may change

Owns the files in the header.

Reader and neighbour files, for the new field only: a `..` in a pattern or a literal's new field, no other edit.

- `jig/crates/jig-domain-hub-link/src/down.rs`: pass `hello` through both lines.
- `jig/testing/jig-test-service/src/route.rs`: `core::Held::Refuse { channel, .. }`. This test service keeps its fixed text.
- `jig/tests/fleet/src/world.rs`, `jig/tests/fleet/tests/suite/{models,jobs,host_kinds,program_sources}.rs`, `jig/crates/jig-domain-fleet/src/tests.rs`, `jig/crates/jig-domain-core/src/tests/effects/deployment_connectors.rs`: a test not about the reason matches `Request::Refuse { channel, .. }`.
- `temper/crates/temper-engine-domain/src/route/held.rs`, line 108: `core::Held::Refuse { channel, .. }`. This is the one temper line this branch carries, by the rule that a required field's producer carries its readers' names. `refusal-words` uses the field.

Departs from the implementer's text: delete from `jig/docs/design/domain/engine.status.md` the `Not yet` bullet beginning `8: the fleet's refusal hands its parent no reason`.

Options, each pre-approved:

- `refuse` in `channel.rs` already has the hello, so compute the figures there. The `why` is copied into both the record and the request.
- `Request` or `Held` may have a size or owned-bytes check. `HelloRefused` is small and `Copy`, so if a check fails on it, raise nothing and say so in your report: that's a question.
- `jig/tests/fleet/src/world.rs` may keep a model of refusals. If a model's rule reads the channel alone, leave it so.

## Steps

1. **Reproduce.** In `jig-domain-fleet/src/tests/needs.rs`, beside the rows at line 221, add `a_refused_hello_says_why_and_what_each_side_holds`: one row per `AgentShortfall` that has figures, built as the existing rows are, asserting the exact `Request::Refuse`. It doesn't compile against today's shape, so write the shape first with `hello: None` everywhere and see it fail on the value.
2. **Add the bound-later row:** `a_bound_set_after_the_start_refuses_with_its_figures`, which steps `Event::Needs` with a higher `turns`, then says hello, and finds the new `required`.
3. **Add** `a_model_the_agent_does_not_declare_has_no_figures`, and rows for `JobKinds`, `Workers`, `Position`, `StopBound`, `ReservedChannel` with `figures: None`.
4. **Add** `a_refusal_with_no_reason_hands_up_none`, for malformed metadata and a source fault.
5. In `jig/tests/fleet/tests/suite/models.rs`, beside the `Window` and `Output` rows at line 105, assert the figures through the world, by the model's name.
6. In the core, a story that a hello's reason reaches `Held::Refuse`: `jig-domain-core/src/tests/effects/deployment_connectors.rs` is nearest. Add it there, as small as the tests beside it.
7. Break `figures` once (swap `required` and `offered` for `Turns`), see the row fail, and restore.
8. `cargo fmt`, `python3 -I -B jig/scripts/old_forms.py`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p jig-domain-fleet -p jig-domain-core -p jig-domain-hub-link -p jig-test-service -p jig-fleet-world --all-targets -- -D warnings`.
9. Tests: `heavy cargo nextest run -p jig-domain-fleet -p jig-fleet-world -p jig-domain-core -p jig-domain-hub-link -p jig-test-service`, then `heavy cargo check -p temper-engine-domain --tests` and `heavy cargo check -p temper-service --tests`. Commit and push.

## Report

The implementer's report, plus any reader of `Request::Refuse` or `Held::Refuse` you found that this spec doesn't name.
