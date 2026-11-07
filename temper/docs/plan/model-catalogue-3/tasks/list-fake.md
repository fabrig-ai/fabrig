+++
id = "list-fake"
part = "provider-list"
kind = "feature"
needs = []
files = ["skein/crates/skein-fake-llm-protocol/src/**", "skein/crates/skein-fake-peers/src/llm.rs", "skein/testing/skein-llm-kit/src/fake.rs", "skein/tests/llm-connection/tests/suite/fake_protocol.rs", "skein/docs/design/fake-llm.status.md"]
effort = "high"
minutes = 60
+++

# The fake provider answers its list

## Goal

The fake's `GET /v1/models`, so that every world from skein's up can feed a list and prove what reads it. The body is bytes the world gives, never built from the fake's own types, so that a reader's parsing is proved and not assumed.

Acceptance:

- `contracts.md`, 3's fields and `Peer::set_list` exist, as `skein/docs/design/fake-llm.md`, 3.2 says.
- Rows of the fake's existing protocol table, each over the fake's route: a list answered, with `?limit=1000` too; a route with none answers `404`; `list_status` answers its status; a wrong credential, or none, answers unauthorized, as a call does; a `POST` to `/v1/models` is refused as today.
- `Observation::List` names the route and the credential as `Observation::Credential` does, holds no credential's value, and counts no call and spends no script: a script's first call is still answered first after a list.
- `set_list` between two requests changes the second's answer.
- `fake-llm.status.md`'s 3.2 line goes.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 3.
- `skein/docs/design/fake-llm.md`, section 3 and 3.2.

## Owns and may change

Owns the header's files. `provider.rs:572` is the entrance: today it requires a POST on the exact inference target.

- **Every `Config { .. }` literal of the fake** gains `list: None, list_status: None`, by the required-field exception, in any file of any tree: `temper/crates/temper-fake/src/wire.rs:79`, `temper/crates/temper-playground/src/fakes.rs:123`, `jig/testing/jig-channel-kit/src/processes.rs:503` and those `rg 'Config \{' -g '*fake*'` finds.

Options, each pre-approved:

- If `Config` is per peer and not per route, keep it per peer with one list, since a peer serves one route at a time in every world; say so in the report.
- If `set_list` can't reach a running `Service` by a direct call, take a `Down`-like entrance as its other commands do.

## Steps

The list needs no domain query: answer it in the protocol's `Server`, beside its unauthorized answer, with its own `Observation`. Test with the kit's raw exchange at `skein/testing/skein-llm-kit/src/fake.rs:336`, as a row of `fake_protocol.rs`'s table where one fits. Then `cargo fmt`, `heavy cargo clippy` of each package you changed with `--all-targets -- -D warnings`, `heavy cargo nextest run -p skein-fake-llm-protocol -p skein-llm-connection-world`, naming the world package as its manifest does, `heavy cargo check -p temper-fake -p temper-playground -p jig-channel-kit --tests`; commit and push.

## Report

As the implementer's text says.
