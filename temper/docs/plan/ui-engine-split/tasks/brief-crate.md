+++
id = "brief-crate"
part = "shared-brief"
kind = "migration"
needs = []
files = ["temper/crates/temper-protocol-brief/**", "temper/crates/temper-engine-smith/src/start.rs", "temper/crates/temper-engine-smith/src/lib.rs", "temper/crates/temper-engine-smith/src/tests/brief_fit.rs", "temper/crates/temper-engine-smith/Cargo.toml", "temper/crates/temper-service/src/assignment.rs", "temper/crates/temper-service/src/worker_route.rs", "temper/crates/temper-service/src/tests/stock_brief.rs", "temper/crates/temper-service/Cargo.toml", "temper/docs/design/domain/agent.status.md", "temper/docs/design/protocol/README.status.md"]
effort = "medium"
minutes = 45
+++

## Goal

The brief's fitter, `fit_brief` and `fit_brief_bytes` with their private
helpers, moves out of `temper-engine-smith` into a new step crate,
`temper-protocol-brief`, which the engine's translation and the worker
service each link, so that the worker's route no longer needs the
engine's translation crate for it (the request's first item;
`temper/docs/design/draft/deployment-binaries.md`, 4, the first known
coupling).

Acceptance:

- `temper/crates/temper-protocol-brief` exists, a step crate (`#![no_std]`
  with `alloc`, `#![forbid(unsafe_code)]`, the `step` lint profile) on
  `smith-domain-run` and `skein-lib` alone, holding `fit_brief`,
  `fit_brief_bytes` and the helpers they call (`fit`, `shortened`,
  `note_omitted`, `continuation`, `cut_line` and whatever else of
  `start.rs` only they use), moved unchanged but for paths, with their
  doc comments; its crate doc cites `temper/docs/design/domain/agent.md`,
  4.3. Its signatures are `notes/contracts.md`, 1's.
- `temper-engine-smith` no longer defines or exports them: its `lib.rs`
  `pub use start::{…}` drops both names; `start.rs`'s own call calls the
  new crate.
- `temper-service`'s `assignment.rs` and `worker_route.rs` call
  `temper_protocol_brief::fit_brief_bytes` and `::fit_brief`;
  `src/tests/stock_brief.rs` likewise.
- `temper-engine-smith/src/tests/brief_fit.rs` stays where it is and calls
  the new crate: its rows build charters with `temper-engine-smith`'s
  `charter()`, which the new crate can't reach. This departs from
  `notes/common.md`'s "tests move with them"; the new crate takes no test
  of its own, since nothing new is pinned.
- Every test of `temper-engine-smith` and `temper-service` passes
  unchanged.
- The status entries close: in `temper/docs/design/domain/agent.status.md`
  the **Not yet** 4.3 entry "the fitter is `temper-engine-smith`'s …", and
  in `temper/docs/design/protocol/README.status.md` the **Not yet** 5 entry
  "`temper-protocol-brief`: the fitter is `temper-engine-smith`'s …",
  each its lines alone. This departs from the implementer's text, which
  closes `docs/design/` to you: these entries are this task's.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3, the bullet "The brief fits
  the room its charter leaves".
- `temper/docs/design/protocol/README.md`, section 5, the rows
  `crates/temper-protocol-brief` and `crates/temper-engine-smith`.
- `temper/docs/plan/ui-engine-split/notes/contracts.md`, 1.
- `temper/docs/plan/ui-engine-split/notes/common.md`.

## Owns and may change

- Owns the files in `files`.
- The root `Cargo.toml`: the member line
  `"temper/crates/temper-protocol-brief",` right after
  `"temper/crates/temper-protocol-store",`, and the dependency line
  `temper-protocol-brief = { path = "temper/crates/temper-protocol-brief" }`
  right after `temper-protocol-store = { … }`; nothing else there.
- `Cargo.lock`, as cargo writes it.
- A caller of `temper_engine_smith::fit_brief` or `fit_brief_bytes`
  outside these files (none is known: `rg 'fit_brief' --type rust`): its
  call path alone, and named in the report.
- Pre-approved: if a helper the fitter calls is also used by the rest of
  `start.rs`, keep it in `start.rs` and give the new crate its own copy
  only if it is under ten lines; otherwise export it from the new crate
  and have `start.rs` call it, and say which in the report.
- Not yours: `temper-engine-smith`'s other modules and tests (the
  `goal-call-faults` plan is changing `calls.rs`, `answers.rs` and
  `src/tests.rs`); any other module of `temper-service`.

## Steps

1. `python3 -I -B tools/lints/lints.py --set step
   temper/crates/temper-protocol-brief` after writing its manifest; the
   tree's `clippy.toml` covers it.
2. The move and the call sites. `heavy cargo check -p
   temper-protocol-brief -p temper-engine-smith -p temper-service --tests`
   while errors last.
3. `heavy cargo nextest run -p temper-engine-smith --lib -E
   'test(/brief_fit/)'` and `heavy cargo nextest run -p temper-service
   --lib -E 'test(/stock_brief/)'`; read the counts.
4. The status entries.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I
   -B tools/lints/lints.py`; `heavy cargo clippy -p temper-protocol-brief
   -p temper-engine-smith -p temper-service --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-protocol-brief -p
   temper-engine-smith -p temper-service`; `heavy cargo check -p temper
   -p temper-playground -p temper-system-world -p
   temper-engine-domain-world --tests`.
6. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit and push.

## Report

As the implementer's text says, with the helpers that moved and any that
stayed.
