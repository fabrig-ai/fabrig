+++
id = "wasm32-expects"
part = "wasm32-expects"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/**/*.rs",
  "jig/crates/jig-domain-people/src/**/*.rs",
  "jig/crates/jig-domain-tasks/src/**/*.rs",
  "jig/crates/jig-domain-hub-link/src/**/*.rs",
  "jig/crates/jig-protocol-store/src/**/*.rs",
]
effort = "medium"
minutes = 45
+++

# jig's crates under temper-playground pass wasm32 clippy

## Goal

`cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings` fails in jig's
code: `#[expect(clippy::large_enum_variant)]` on `pub enum Ask`
(`jig/crates/jig-domain-people/src/boundary.rs:225`) is not met on wasm32, where sizes differ, and
the same in `jig-domain-tasks`. Make every jig crate that `temper-playground` builds pass wasm32
clippy, warnings denied, by scoping each such expectation to the target where its lint fires, and
keep the native clippy of the changed packages green. Lint attributes only.

Acceptance:

- `heavy cargo clippy --keep-going --target wasm32-unknown-unknown -p temper-playground -- -D
  warnings` (step 4) prints no warning or error located under `jig/`. Errors under `skein/`,
  `smith/` or `temper/` may remain and are listed under `## Left`; where one is under `skein/` or
  `smith/`, step 5's warnings-only run prints no warning located under `jig/`.
- `heavy cargo clippy -p <every package you changed> --all-targets -- -D warnings` (step 6)
  passes, on the host.
- The diff of your commits (`git diff --stat <your base>..HEAD`) names only `.rs` files under
  `jig/crates/*/src/`, outside `src/tests/`; every changed line is a lint attribute or its
  formatting. No `#[allow`, no `#![allow`. Each moved expectation keeps its lint and its reason
  word for word.
- The report lists, for each jig site step 1 named, the lint, file and line and the form used,
  and under `## Left` every warning step 1 or step 4 showed outside `jig/`.

## Inputs

- `jig/docs/plan/wasm32-jig/notes/common.md`: the standing rules, and this plan's departure from
  the implementer's text (no tests run).
- `jig/docs/plan/wasm32-jig/inputs/request.md`: the request and its command.
- `temper/docs/design/playground.md`, 9 ("Testing", its bullet "The wasm32 builds"): why
  `temper-playground`'s wasm32 build is to pass clippy with its lints as errors. Read only.

The size- and target-dependent expectations in the jig crates `temper-playground` builds, at the
plan's base (lib code only; step 1's run decides which are unmet on wasm32, and may name others):

- `clippy::large_enum_variant`: `jig/crates/jig-domain-people/src/boundary.rs:225` (`Ask`) and
  `src/domain/requests.rs:26`; `jig/crates/jig-domain-tasks/src/proposals.rs:81`
  (`ProposalAction`); `jig/crates/jig-domain-core/src/vocabulary.rs:1241` (`Work`),
  `src/reads/vocabulary.rs:134` (`Read`) and `:421` (`EndedGroup`);
  `jig/crates/jig-domain-hub-link/src/lib.rs:344`; `jig/crates/jig-protocol-store/src/boundary.rs:234`.
- `clippy::result_large_err`: inner attributes in `jig/crates/jig-domain-tasks/src/holds.rs:5`,
  `batch.rs:5`, `moving.rs:5`, `procedure.rs:6`, `waits.rs:2` and
  `jig/crates/jig-domain-core/src/runs/delegation.rs:4`; outer ones in
  `jig/crates/jig-domain-core/src/routing/effect/procedure.rs:21`, `:159`, `:218`, `:244`,
  `src/runs/llm.rs:16`, `:36`, `src/runs/account_preference.rs:147` and
  `jig/crates/jig-domain-hub-link/src/down.rs:9`.
- `clippy::trivially_copy_pass_by_ref`: inner, `jig/crates/jig-protocol-store/src/codec/settings.rs:3`.

## Owns and may change

- The lint attributes in the `src/` of the jig crates `temper-playground` builds for wasm32:
  `jig-client-domain`, `jig-client-protocol-wire`, `jig-domain-accounts`, `jig-domain-authority`,
  `jig-domain-brief`, `jig-domain-core`, `jig-domain-fleet`, `jig-domain-host`,
  `jig-domain-hub-link`, `jig-domain-link`, `jig-domain-notes`, `jig-domain-people`,
  `jig-domain-tasks`, `jig-domain-views`, `jig-protocol-accounts`, `jig-protocol-charter`,
  `jig-protocol-config`, `jig-protocol-engine-clients`, `jig-protocol-engine-clients-http`,
  `jig-protocol-engine-workers`, `jig-protocol-store`, `jig-protocol-transcript`,
  `jig-protocol-worker-link`, `jig-protocol-worker-wire`. Only the sites step 1 or step 4
  names; every other line stays as it is.
- Nothing else: no type, variant, signature or body (boxing a variant, passing by value), no
  `Cargo.toml`, no `jig/clippy.toml` or threshold, no `.config/gate/wasm32.txt`, no `docs/`, no
  code under `skein/`, `smith/` or `temper/`.

Pre-approved, each the default for its case:

- **An expectation unmet on wasm32** (`unfulfilled_lint_expectations` at a jig site): wrap it,
  same lint, same reason. Outer:

  ```rust
  #[cfg_attr(
      not(target_arch = "wasm32"),
      expect(
          clippy::large_enum_variant,
          reason = "an amendment stays inline, as `Event` carries a keyed request whole, and is priced by size_of"
      )
  )]
  pub enum Ask {
  ```

  Inner (a module's `#![expect(...)]`): `#![cfg_attr(not(target_arch = "wasm32"), expect(<lint>,
  reason = "<the same reason>"))]`, laid out by `cargo fmt`.
- **An `expect` naming several lints, some unmet on wasm32:** split it: the lints met on both
  targets stay in a plain `#[expect(..., reason = "...")]`, the others go into the `cfg_attr`
  above; both keep the original reason.
- **A lint that fires on wasm32 only** (a warning located under `jig/` that is not an unmet
  expectation, from a size- or pointer-width-dependent lint such as
  `clippy::trivially_copy_pass_by_ref`): `#[cfg_attr(target_arch = "wasm32", expect(<lint>,
  reason = "..."))]` on the narrowest item it names, the reason saying in the item's words why
  one form serves both targets (for example "the value is larger on 64-bit hosts, where it is
  borrowed"). The report names each.
- **Any other warning or error located under `jig/`** (a compile error, an unused item, a lint
  that is not size- or target-dependent): commit and push what you have, then report
  `Question:` with the lint, file, line and item, and the fix you propose.
- **A warning or error outside `jig/`:** change nothing there; list it under `## Left` (common
  note). If there are more than 20, list the first 20 and the count per tree and lint.

## Steps

1. At your base, once, without `-D warnings` so every crate is checked and every warning shows:
   `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground`. From its log, by
   `grep`, list each warning's lint and location (`grep -n -E '^(warning|error)|^ +--> ' <log>`),
   split into those under `jig/` and the rest.
2. For each site under `jig/`, apply the pre-approved form for its case. Then `cargo fmt -p
   <each changed package>`; `git diff --stat` names only the sites' files.
3. `python3 -I -B jig/scripts/old_forms.py` (no `--write`: nothing it counts changes). Commit
   (jig's size-dependent lint expectations hold on the host only; on wasm32 sizes differ).
4. The request's command, warnings denied, once: `heavy cargo clippy --keep-going --target
   wasm32-unknown-unknown -p temper-playground -- -D warnings`. A new error under `jig/`: handle
   it as step 2 does, commit, and run this step once more.
5. Only if step 4 shows an error under `skein/` or `smith/` (a jig crate built on it would go
   unchecked): rerun step 1's command once; it must show no warning under `jig/`.
6. On the host: `heavy cargo clippy -p <every package you changed> --all-targets -- -D
   warnings`.
7. Push your branch.

## Report

As the implementer's text says, and also: the host each clippy ran on and its wall time from
heavy's output; for each jig site, its lint, file, line and form (`cfg_attr` on the host, on
wasm32, or split); and under `## Left`, every warning or error outside `jig/` by tree, package,
file, line and lint, temper's first, since temper's own code is the next to fail once jig's
passes.
