+++
id = "forge-surface"
part = "engine"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/connect.rs",
  "temper/crates/temper-engine-domain-forge/src/make.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/make.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/connect.rs",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/domain.rs",
  "temper/crates/temper-engine-domain/src/admission.rs",
  "temper/crates/temper-engine-domain/src/route/applications.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/tests/setup_github.rs",
  "temper/crates/temper-service/src/tests/fixtures.rs",
  "temper/crates/temper-service/src/tests/engine.rs",
]
effort = "medium"
minutes = 60
+++

# The forge domain's surface for a connected Forgejo and Replace

## Goal

Add the types `notes/contracts.md`, C3 names, with the smallest bodies that keep every behaviour
as it is, so that the wire's mapping, the forge domain's admission and the routing can each be
built on them at once by later tasks. Nothing is offered yet: a tokens connect or a Replace is
refused `NotTaken` at admission, as a request the forge does not take.

Acceptance:

- `temper-engine-domain-forge`'s boundary has `ForgeAsk::Connected { place, git_program }`,
  `How::ForgejoTokens`, `Ask::Replace { forge }`, `Request::Replace { forge, request, carried }`
  and `Event::BotLogin { forge, login }`, each with the doc comment C3 gives; the root's
  `Request` has `Replace { forge, request, carried }`, forwarded from the forge domain's as
  `Request::Connect` is.
- Admission: a `Connect` with `How::ForgejoTokens`, of a kept forge or a new one, and a `Connect`
  of `ForgeAsk::Connected` with any `How`, answer `ConnectAnswer::Refused(ForgeRefusal::NotTaken)`
  with no record made; an `Ask::Replace` answers the same, as `Answered::Connect`; each names its
  carried value unused, as every refusal at admission does. `Event::BotLogin` changes nothing.
- The root gives `Ask::Replace` the deployment's scope, as `Connect`'s.
- Two forge-domain unit tests pin the stubs: a tokens connect (new and kept) and a Replace are
  refused `NotTaken`, their carried values named unused, no record made (`tests/connect.rs`);
  every existing test passes.

## Inputs

- `temper/docs/plan/forgejo-connect/notes/common.md`, and `notes/contracts.md`, C3.
- `temper/docs/design/client/README.md`, 4.6 (`Making`, `Connect`, `Replace`, `ForgeRefusal`).
- `temper/docs/design/protocol/forge.md`, 2.2.

## Owns and may change

- `temper-engine-domain-forge/src/boundary.rs`: the five additions, each last in its enum.
- `temper-engine-domain-forge/src/connect.rs`: in `make` (`:177`), `ForgeAsk::Connected` answers
  `NotTaken` before a record is made, for every `How`; `How::ForgejoTokens` answers `NotTaken` in
  every arm that matches `How` (`:191-245`, `:312-331`) and on the kept paths (`start`, `admin`);
  `Ask::Replace` joins `start`'s early answers (`:51-61`) as `Answered::Connect(ConnectAnswer::
  Refused(ForgeRefusal::NotTaken))`, its carried value named unused as `start` names others'. Every
  other match over `Ask` (`:74-88`, `:160-174`, `:264-308`) gets its arm, unreachable where
  `start` has answered already, with that reason.
- `temper-engine-domain-forge/src/make.rs:49-59` and `domain.rs:1237-1254`: `Ask::Replace` routed
  to `connect::start` with `Connect`, `Install`, `Accept` and `Reachable`; `domain.rs`'s `Event`
  match gets `Event::BotLogin { .. } => {}`, with a comment that it is the connect's to fill.
- `temper-engine-domain-forge/src/tests/make.rs:131-140`: the forced `How` arm;
  `tests/connect.rs`: the two tests.
- `temper-engine-domain/src/boundary.rs` (`Request::Replace`, beside `Connect` at `:594`),
  `route/forge_route.rs:3127` (forward `forge::Request::Replace` as `Connect` is forwarded),
  `domain.rs:786` (list it with `Connect`), `admission.rs:95-103` (list `forge::Event::BotLogin`
  with `ConnectEnded`), `route/applications.rs:24-36` (`forge::Ask::Replace` in the deployment's
  scope).
- `temper-service/src/route.rs:41-49`: `How::ForgejoTokens => unreachable!("a tokens connect is
  refused at admission until the routing stages Check")`, and a `root::Request::Replace { .. }`
  arm with the same kind of reason; `tests/setup_github.rs:98-112`, `tests/fixtures.rs:795-808`
  and `tests/engine.rs:490-503`, each list of `root::Request` gaining `Replace`.

Pre-approved: any other exhaustive match over these enums that the compiler names, in temper's
crates or worlds (`temper/tests/**`), gets its forced arm, as the nearest arm of its kind does,
and is listed in the report. No wire, client or store file changes in this task.

## Steps

1. The boundary and the forge domain's stubs and tests: `heavy cargo nextest run -p
   temper-engine-domain-forge --lib -E 'test(connect) | test(make)'`.
2. The root: `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(setup)'`.
3. The service's forced arms: `heavy cargo check -p temper-service --tests`, then `heavy cargo
   nextest run -p temper-service --lib -E 'test(setup_github) | test(engine)'`.
4. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-service --all-targets -- -D warnings`; the focused tests of
   those three packages, and `heavy cargo check --tests` for `temper-engine-domain-world`
   (`temper/tests/engine/domain`) and `temper-client-world` (`temper/tests/client`), which match
   the root's requests. Commit; push.

## Report

As the implementer's text says, with every forced arm outside the listed files.
