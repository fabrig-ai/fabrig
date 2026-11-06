+++
id = "trial-wire"
part = "trial-wire"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema",
  "temper/crates/temper-client-protocol-wire/src/**",
  "temper/crates/temper-client-protocol-wire/golden/**",
]
effort = "medium"
minutes = 50
+++

# The trials on the forge's wire, and the root's vocabulary

## Goal

The surface other tasks build on, and no behaviour: `notes/contracts.md`, *The wire*. `Request::try_account` is decoded
and refused, and `Watch::trials` is watched and holds nothing, until `trial-root` fills them. Formats change in place,
before v0.1.0: no new wire version.

Acceptance:

- The schema holds the note's records, enums and arms; the generated code and goldens come from their own tools.
- `temper-service`'s `family.rs` compiles with the new arms: `w::Request::TryAccount(_)` becomes
  `root::Application::Try { account, again }`; `w::Watch::Trials` becomes `Watched::Application { scope: Own, connector:
  wire::FORGE, subject: Token::new(TRIALS_SUBJECT) }`; `ApplicationAnswer::Try(Tried)` encodes `w::Outcome::Tried`,
  `Try(NotTheirs)` `w::Outcome::NotTheirs`. Each has a round-trip test beside its neighbours.
- `temper-engine-domain`'s `Application::Try` is routed in `route/applications.rs`, for now answering `Try(NotTheirs)`
  at once, as a comment-free stub that `trial-root` replaces; the scope arm is `Own`.
- `temper-client-domain` compiles: an `Outcome` it doesn't read goes where its neighbours do.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`, `temper/docs/plan/model-check/notes/contracts.md`, *The wire*.
- `temper/docs/design/domain/trial.md`, 6.
- A recent precedent in `git log`: `054953a701`, `program_login` added to a read in place; `08b8e65d7b`, watches and
  requests added.

## Owns and may change

Owns, as `files` lists them.

May also change, each for the new arms alone, and named in your report:

- `temper/crates/temper-service/src/family.rs`, `tests/family.rs`.
- `temper/crates/temper-engine-domain/src/boundary.rs`, `route/applications.rs`.
- `temper/crates/temper-client-domain/src/forge/**`, `route.rs`, where a `match` on `Outcome`, `Watch` or `Request` is
  forced.
- `temper/crates/temper-client-shell-*/src/**`, `temper/tests/**`: arms the compiler forces.

`TRIALS_SUBJECT` belongs to `temper-engine-domain-trials`, `trial-machine`'s, which may not have merged: define it in
`temper-engine-domain/src/boundary.rs` as `pub const TRIALS_SUBJECT: u64 = (1 << 62) | (1 << 36);`. `trial-root` moves
the use to the child's when both stand.

**`sign-in-new-tab/return-goes` removes `Request::return_`** on its own trunk: don't touch that arm. On a merge the schema
is hand-merged and the goldens and generated code regenerated, by their own tools.

## Steps

Add the note's lines beside `Dismiss`'s and `Dismissals`' in the schema. Regenerate, by the crate's generator as its
`README` or `source.rs` says; the goldens the same way. The schema's `list 64` bounds the rows; `temper-service`'s
`Limits` needn't move.

Tests: `src/tests.rs`'s round-trip list gains `Trials`, `TryAccount` and each `TrialWhy` arm; `temper-service`'s
`tests/family.rs`'s decode and `encoded` tables gain their rows (bite: remove the arm's mapping).

The checks: fmt, `old_forms.py` for `temper`, clippy on `temper-client-protocol-wire`, `temper-service`,
`temper-engine-domain` and `temper-client-domain`; `heavy cargo nextest run -p temper-client-protocol-wire -p temper-service --lib`.
`heavy cargo check -p temper --tests` and `-p temper-client-shell-web --tests`, the wire's readers. Citations. Push.

## Report

As the implementer's text asks: what the compiler forced where.
