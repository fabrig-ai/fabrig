+++
id = "repairs-call"
part = "repairs"
kind = "feature"
needs = ["store-surface"]
files = ["temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs", "temper/tests/engine/domain/tests/suite/forge_routes/repairs.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs"]
effort = "high"
minutes = 150
+++

# `change` takes `repairs`, the root checks it, and the change keeps it

## Goal

The coordinator's `change` names the red landing a change repairs (`repairs`, its commit's
digits); the root refuses it unless it names a red landing of temper's in the change's
repository on its landing branch, and the change keeps it in its row. Clearing by it is
`repairs-verdict`'s.

Acceptance (`notes/contracts.md`, section 2; `temper/docs/design/domain/agent.md`, 4.3.1):

- `PLAN_CHANGE` declares `repairs` in each change's object, alone and in `changes`, with the
  exact schema text of contracts.md, 2; `SPECS`' chat version does not.
- `fn change` (`calls.rs`) reads it into `ChangeCall.repairs`; digits that are not 40 or 64
  lower-case hexadecimal are a problem naming `repairs`, as the function's other members'
  problems are worded.
- `ChangeCall` gains `repairs`, last; `ChangeCall::words` sets `None`.
- `change_batch`: `repairs` is a plan member (the predicate, and the refusal's words list it:
  `change's landing, branch, name, after, repairs and changes are a plan's: …`, the rest of the
  sentence as it is); once the change's repository and landing branch are resolved,
  `domain.forge.landing_red(repository, &commit, &base)` decides it, each refusal worded exactly
  as contracts.md, 2 has it, refusing the call.
- `chat_change` writes `Parameter::Bytes { name: 12, value: <the digits> }`; `start_change`
  reads parameter 12 into `ChangeRow.repairs` (and no longer refuses 12 as unknown); a change
  without it keeps `None`. Restart reads it back as the row's other fields are.
- Tests:
  - in `temper-engine-smith/src/tests.rs`: `PLAN_CHANGE`'s `repairs` is declared and read, 40
    and 64 digits, alone and in `changes`; upper-case or 39 digits are a problem;
  - in `forge_route/tests.rs`: a call naming `repairs` from a run without the control family is
    refused with the plan's words; a commit no landing has is refused `is no live landing of
    temper's in OWNER/NAME.`;
  - a new world file `temper/tests/engine/domain/tests/suite/forge_routes/repairs.rs` (its
    `mod` line in `forge_routes.rs`), on the tiers world (`temper/tests/engine/domain/src/tier_runs.rs`,
    the red range `tiers.rs` builds; reuse its helpers by moving what you need into
    `tier_runs.rs` if they are private to `tiers.rs`): a goal's change naming the red landing's
    commit is made and its row keeps it; one naming a landing that is cleared is refused `is
    red in no tier.`; one into another branch than the red landing's is refused `landed into
    main, not BRANCH.`.

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`; `notes/contracts.md`, sections 1 and 2.
- `temper/docs/design/domain/agent.md`, 4.3.1 (`repairs`, who may set the plan's members);
  `temper/docs/design/domain/after-landing.md`, section 7 (`repairs`).

## Owns and may change

- Owns the files of this spec's header for what acceptance names; in `forge_route.rs`,
  `change_batch`, `chat_change` and `start_change` alone (`repairs-verdict` edits
  `change_verification_words` in the same file at the same time).
- Reader files, compiler-forced only: every `ChangeCall {` literal (`smith/src/tests.rs`,
  `temper/tests/engine/domain/tests/suite/forge_routes/goal_branch.rs`, any the compiler names
  in temper, `repairs: None`).
- `temper/tests/engine/domain/src/tier_runs.rs`: helpers moved from `forge_routes/tiers.rs`
  unchanged, and their uses there; nothing else in `tiers.rs`.
- reference-m3, m5 and m7b add their own members to `PLAN_CHANGE`, `ChangeCall` and
  `change_batch` at the same time: add yours beside theirs, last.
- `temper/docs/help/` is not yours: the help task gives `repairs` its words.

## Steps

1. `ChangeCall`, the declaration and the parse, with their tests.
2. `change_batch`'s check, `chat_change`'s parameter, `start_change`'s read.
3. Tests: `heavy cargo nextest run -p temper-engine-smith --lib`;
   `-p temper-engine-domain --lib -E 'test(/forge_route/)'`;
   `-p temper-engine-domain-world --test suite -E 'test(/forge_routes::(repairs|tiers|change_tool|goal_branch)/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-smith -p temper-engine-domain
   -p temper-engine-domain-world --all-targets -- -D warnings`; commit and push.

## Report

As the implementer's text asks.
