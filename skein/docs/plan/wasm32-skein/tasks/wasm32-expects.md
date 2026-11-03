+++
id = "wasm32-expects"
part = "wasm32-expects"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-llm-connection/src/**/*.rs",
  "skein/crates/skein-llm/src/**/*.rs",
  "skein/crates/skein-forge-setup/src/**/*.rs",
  "skein/crates/skein-oauth-accounts/src/**/*.rs",
  "skein/crates/skein-fake-peers/src/**/*.rs",
  "skein/crates/skein-fake-forge-protocol/src/**/*.rs",
  "skein/crates/skein-fake-forge-github/src/**/*.rs",
  "skein/crates/skein-tls/src/**/*.rs",
]
effort = "medium"
minutes = 55
+++

# skein's crates under temper-playground pass wasm32 clippy

## Goal

`cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings` fails in
skein's code: `#[expect(clippy::large_enum_variant)]` on `pub enum Request`
(`skein/crates/skein-llm-connection/src/boundary.rs:49`) is not met on wasm32, where sizes
differ. Make every skein crate that `temper-playground` builds pass wasm32 clippy, warnings
denied, by scoping each such expectation to the target where its lint fires, and keep the native
clippy of the changed packages green. Then try `skein-tls`, `skein-fake-forge-github` and
`skein-fake-peers` on their own and say which pass. Lint attributes, and target fences only
where the case below allows.

Acceptance:

- `heavy cargo clippy --keep-going --target wasm32-unknown-unknown -p temper-playground -- -D
  warnings` (step 4) prints no warning or error located under `skein/`. Errors under `jig/`,
  `smith/` or `temper/` may remain and are listed under `## Left`.
- Step 5's run (`-p skein-tls -p skein-fake-forge-github -p skein-fake-peers`, warnings denied)
  prints no warning or error located under `skein/`, or the report says, for each of the three,
  what it printed and why it is left.
- `heavy cargo clippy -p <every package you changed> --all-targets -- -D warnings` (step 6)
  passes, on the host.
- The diff of your commits (`git diff --stat <your base>..HEAD`) names only `.rs` files under
  `skein/crates/*/src/`, outside `src/tests/`; every changed line is a lint attribute, a
  `#[cfg(...)]` fence the case below allows, or their formatting. No `#[allow`, no `#![allow`.
  Each moved expectation keeps its lint and its reason word for word.
- The report lists, for each skein site step 1, 4 or 5 named, the lint, file, line and the form
  used; for each of the three crates of step 5, `passes` or what fails; and under `## Left` every
  warning step 1 or step 4 showed outside `skein/`.

## Inputs

- `skein/docs/plan/wasm32-skein/notes/common.md`: the standing rules, and this plan's departure
  from the implementer's text (no tests run).
- `skein/docs/plan/wasm32-skein/inputs/request.md`: the request and its command.
- `skein/docs/design/testing.md`, 10 ("The portable target"): what builds for wasm32, its
  bullets "Fenced by target" (what reads the system is under `cfg(unix)` where it sits) and
  "Checked in the gate" (clippy's lints as errors; "a fence that leaves code dead warns"). Read
  only.
- `skein/docs/design/tls.md`, 4, its bullet "Refused on the portable target": skein-tls builds
  for wasm32 and refuses there. Read only.

The size-dependent expectations in the skein crates `temper-playground` builds that the gate's
wasm32 step does not yet check, at the plan's base (lib code only; step 1's run decides which
are unmet on wasm32, and may name others):

- `clippy::large_enum_variant`: `skein/crates/skein-llm-connection/src/boundary.rs:49`
  (`Request`, measured unmet); `skein/crates/skein-forge-setup/src/reconcile.rs:56` (`Event`),
  `src/host/connection.rs:52` (`Speech`) and `src/host/boundary.rs:112` (`Request`);
  `skein/crates/skein-oauth-accounts/src/accounts.rs:11` (`Account`, a multi-line `expect`) and
  `:55` (`Source`); `skein/crates/skein-fake-peers/src/forge/layer.rs:41` (`Service`, multi-line)
  and `:103` (`Server`).
- `clippy::result_large_err`: `skein/crates/skein-llm/src/client.rs:634` (`next_call`);
  `skein/crates/skein-fake-forge-protocol/src/service.rs:104` (`target`).
- Already scoped by `target_pointer_width = "64"`, and left as they are (common note):
  `skein/crates/skein-fake-forge-domain/src/domain.rs:228`,
  `skein/crates/skein-fake-forge-github/src/auth.rs:66` and `src/writes.rs:233`,
  `skein/crates/skein-fake-peers/src/forge.rs:37`.

The skein crates of the gate list's graph (`skein-lib`, `skein-codec`, `skein-io`, `skein-http`,
`skein-shell`, `skein-sim`, `skein-sim-world`) already pass the gate's wasm32 step; a warning in
one of them under `temper-playground` is handled the same way.

## Owns and may change

- The lint attributes in the `src/` of the skein crates `temper-playground` builds for wasm32
  (`cargo tree --target wasm32-unknown-unknown -p temper-playground` lists 29), and of
  `skein-tls`, `skein-fake-forge-github` and `skein-fake-peers`: only the sites step 1, 4 or 5
  names; every other line stays as it is.
- A `#[cfg(...)]` fence on a private or `pub(crate)` item, `use` line or module of those crates,
  only in the dead-code case below.
- Nothing else: no type, variant, signature or body (boxing a variant, passing by value), no
  `Cargo.toml`, no `skein/clippy.toml` or threshold, no `.config/gate/wasm32.txt`, no `docs/`, no
  code under `jig/`, `smith/` or `temper/`, no `cfg_attr(target_pointer_width = ...)` site that
  holds.

Pre-approved, each the default for its case:

- **An expectation unmet on wasm32** (`unfulfilled_lint_expectations` at a skein site): wrap it,
  same lint, same reason. Outer:

  ```rust
  #[cfg_attr(
      not(target_arch = "wasm32"),
      expect(
          clippy::large_enum_variant,
          reason = "the owning protocol layer moves each Start directly into the component"
      )
  )]
  pub enum Request {
  ```

  Inner (a module's `#![expect(...)]`): `#![cfg_attr(not(target_arch = "wasm32"), expect(<lint>,
  reason = "<the same reason>"))]`, laid out by `cargo fmt`.
- **An `expect` naming several lints, some unmet on wasm32:** split it: the lints met on both
  targets stay in a plain `#[expect(..., reason = "...")]`, the others go into the `cfg_attr`
  above; both keep the original reason.
- **A lint that fires on wasm32 only** (a warning located under `skein/` that is not an unmet
  expectation, from a size- or pointer-width-dependent lint such as
  `clippy::trivially_copy_pass_by_ref`): `#[cfg_attr(target_arch = "wasm32", expect(<lint>,
  reason = "..."))]` on the narrowest item it names, the reason saying in the item's words why
  one form serves both targets (for example "the value is larger on 64-bit hosts, where it is
  borrowed"). The report names each.
- **Code a target fence leaves dead or unused on wasm32** (`dead_code` or `unused_imports` at a
  private or `pub(crate)` skein item, use line or module that only code under `cfg(unix)` or
  `cfg(not(target_arch = "wasm32"))` reaches):
  give it the same `#[cfg(...)]` fence its users carry (testing.md, 10, "Fenced by target"), so
  the host's code is unchanged. If its users carry different fences, or the item is `pub`, ask as
  the next case says.
- **Any other warning or error located under `skein/`** (a compile error, a lint that is not
  size- or target-dependent, an unused variable or `self` inside a body): commit and
  push what you have, then report `Question:` with the lint, file, line and item, and the fix you
  propose.
- **A warning or error outside `skein/`:** change nothing there; list it under `## Left` (common
  note). If there are more than 20, list the first 20 and the count per tree and lint.

## Steps

1. At your base, once, without `-D warnings` so every crate is checked and every warning shows:
   `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground`. From its log, by
   `grep`, list each warning's lint and location (`grep -n -E '^(warning|error)|^ +--> ' <log>`),
   split into those under `skein/` and the rest.
2. For each site under `skein/`, apply the pre-approved form for its case. Then `cargo fmt -p
   <each changed package>`; `git diff --stat` names only the sites' files.
3. `python3 -I -B scripts/old_forms.py` from `skein/` (no `--write`: nothing it counts changes).
   Commit (skein's size-dependent lint expectations hold on the host only; on wasm32 sizes
   differ).
4. The request's command, warnings denied, once: `heavy cargo clippy --keep-going --target
   wasm32-unknown-unknown -p temper-playground -- -D warnings`. A new error under `skein/`:
   handle it as step 2 does, commit, and run this step once more.
5. The three crates the request names, warnings denied: `heavy cargo clippy --keep-going
   --target wasm32-unknown-unknown -p skein-tls -p skein-fake-forge-github -p skein-fake-peers
   -- -D warnings`. A new error under `skein/`: handle it as step 2 does, commit, and run this
   step once more. Note, for each of the three, whether it was checked and passed, failed (with
   what), or went unchecked because a dependency failed.
6. On the host: `heavy cargo clippy -p <every package you changed> --all-targets -- -D
   warnings`.
7. Push your branch.

## Report

As the implementer's text says, and also: the host each clippy ran on and its wall time from
heavy's output; for each skein site, its lint, file, line and form (`cfg_attr` on the host, on
wasm32, split, or a fence); for `skein-tls`, `skein-fake-forge-github` and `skein-fake-peers`,
each one's result in step 5; and under `## Left`, every warning or error outside `skein/` by
tree, package, file, line and lint, smith's and jig's first, then temper's, since theirs are the
next to fail once skein's pass.
