+++
id = "join-worlds"
part = "join"
kind = "migration"
needs = ["join-steps"]
files = [
  "Cargo.toml",
  "Cargo.lock",
  ".config/nextest.toml",
  ".config/gate/always.txt",
  "hearth/crates/hearth/**",
  "hearth/crates/hearth-service-builder/**",
  "hearth/testing/hearth-*/**",
  "hearth/tests/**",
  "hearth/scripts/**",
]
effort = "high"
minutes = 150
+++

# hearth's service, binary, fakes and worlds join: the gate runs all of hearth

## Goal

`join-steps` made the 14 step crates members. Port the other 16 packages (`notes/contracts.md`, 1)
to today's skein and make them members, so that the gate's `default` and `fuzzy` profiles run every
hearth test but the tiers' binaries. This is the port's judgement: the host trait, the service's
bound and the memory ceilings. Behaviour doesn't change, and no bound is weakened to pass.

Acceptance:

- `cargo metadata --format-version 1 --locked --no-deps` lists all 30 hearth packages, and
  `rg -n 'fabrig-dev/skein' hearth --glob '!docs/plan/**'` finds nothing.
- `git diff --stat Cargo.lock` adds the 16, `sha1`, `minijinja` and `memo-map` (with anything only
  those bring), and no existing entry's version moves.
- `heavy cargo nextest run -p 'hearth*'` passes in `default`, and with `--profile fuzzy` in `fuzzy`.
  The report gives each one's count and wall time.
- **No test falls out of every profile:** the counts from `heavy cargo nextest list -p 'hearth*'`
  under `default` and under `fuzzy`, plus those of the `conformance_*` and `recorded_*` binaries,
  sum to the count with `--ignore-default-filter`. The report shows the sum.
- `heavy cargo clippy -p 'hearth*' --all-targets -- -D warnings` passes;
  `heavy cargo nextest run -p fabrig-boundary` passes.
- `cargo fmt --all --check` passes; `python3 -I -B scripts/old_forms.py`, from `hearth/`, passes
  with counts lowered by `--write` and none raised; `python3 -I -B tools/citations/citations.py
  --all` reports 0.
- `python3 -I -B scripts/fail_stop.py`, from `hearth/`, exits 0.

## Inputs

- `notes/common.md` and `notes/contracts.md`, sections 1, 2 and 5, in this plan's directory
  (`hearth/docs/plan/hearth-workspace/`).
- `hearth/docs/plan/hearth-workspace/inputs/request.md`, sections 1 and 2: the leads and their
  table. Nothing was built for them, nor for this spec: the compiler has the last word.
- `skein/docs/foundation/programming-model.md`, 6.3; `skein/docs/foundation/testing-strategy.md`,
  the `Memory` rule of section 6.
- `skein/crates/skein-shell/src/drive.rs`, the `Host` trait; `skein/testing/skein-world/src/heap.rs`,
  `Memory`; `skein/examples/echo/service/src/lib.rs`, how a service holds what io told it.
- `.config/gate/README.md`, `The gate's lists`.
- This cut wrote no design: the request changes none.

**Departures from the implementer's text:** you edit `.config/nextest.toml`,
`.config/gate/always.txt`, root `Cargo.toml` and `Cargo.lock`, as below.

## Owns and may change

**Owns:** the 16 packages; `hearth/scripts/`.

**May change, only as the steps say:** root `Cargo.toml` (the 16 members), `Cargo.lock`,
`.config/nextest.toml` (hearth's two clauses), `.config/gate/always.txt`.

**Leave alone:** the 14 step crates but for a compiler-forced line, and the report names it; every
other tree; `tools/`; `hearth/docs/`; test layout (`test-layout` moves the tests);
`env!("CARGO_MANIFEST_DIR")` (`manifest-paths`).

Pre-approved, each with its default:

- **`service::worst_case` can't stay as written:** the `io::worst_case` and `FileIo::worst_case` terms
  go, because skein no longer has them (`programming-model.md`, 6.3). It keeps its name, an allowed old
  form, and sums hearth's own layers and its queues, and its doc comment says that's what it sums.
  `hearth-service-builder/src/tests/memory.rs` stays as it is: 1 GiB is its limit over what the sum
  now holds. Don't invent terms for skein's layers, and don't drop the function.
- **A formula bound red at run time** (`with_capacity` no longer allocates, a realloc counts old and
  new): keep the formula as the contract it is if an exact check is what the rule allows
  (`notes/contracts.md`, 5); otherwise replace it, in that test, with a named ceiling set by measuring
  under that section's rule. Never raise a formula, or add slack to it.
- **`#[expect]`s and new lints:** as `join-steps` had them, and the report lists them.
- **`real`, `end_to_end` and `refusals` stay in `default`.** One red only because a builder lacks a
  program is a `Question:`, with what's missing; don't filter it out.
- **An over-budget hearth suite:** report the measurements. Budgets aren't yours.

## Steps

1. **Members:** the 16 lines at the end of root `members`.
   `hearth/tests/render/Cargo.toml`: `minijinja = { workspace = true, features = ["json"] }`.
2. **`.config/nextest.toml`:** hearth's clauses for after `join-worlds` (`notes/contracts.md`, 2).
3. **Port, from `heavy cargo check -p 'hearth*' --tests`.** The request's table holds:
   - **`Host::worst_case` (7 impls):** remove each impl, and each helper that only fed one.
     `service::worst_case`: the option above. Its `Queue::<T>::worst_case` calls become `price`.
   - **`Host::next_deadline` (7 impls):** the required method is `next_mechanism_deadline`, and the
     trait's default takes the minimum with policy. Check each impl: where it forwarded a
     `next_deadline`, it now forwards the mechanism's, and a host stating a policy deadline must not
     also return it as a mechanism one. `Service::next_deadline` keeps its name.
   - **`io::MaxOut.events` is gone:** `told` becomes `Queue::uncapped()`, as the echo service's is;
     `io_room` checks the submissions alone and `io_within` measures them alone.
     **Trap:** the loops `for _ in 0..service.told.capacity()` in `hearth-service-builder/src/lib.rs`
     (284, 310) end only by their `break`s once `told` is uncapped: they'd lose the per-pass bound.
     Keep the bound they had, `limits.queue`, so a pass does what it did.
     `tests/builder/src/fixtures/bounds.rs` keeps `QUEUE`; correct its comment if it's now false, and
     confirm the scenario still forces its later pass.
   - **`Stage::new`, `has_room` (`tests/builder/src/domain.rs`, 76, 129, 132):** `Stage::uncapped(limits)`;
     the `has_room` tests go, and the branches they guarded remain.
   - **`Outcome.heap`:** `.peaks()`.
   - **`Memory::Checked` in the builder world (`tests/builder/src/simulated.rs`):** it no longer enforces
     a bound, so `simulated::checked` and `checked_with` take a ceiling and run `Memory::Within(ceiling)`.
     Each caller in `tests/builder/tests/memory.rs` names its own, by `notes/contracts.md`, 5.
     `options.memory == Memory::Checked` becomes "memory is checked". Other `Memory::Checked` uses that
     meant the same get the same.
   - **Leave:** `tests/render/src/parent.rs` and `tests/render/tests/memory.rs` use `Memory::Checked` as a
     flag beside their own `Meter::check(.., bound, ..)`, which still enforces the bound.
4. **Run, then measure:** the worlds' `tests/memory.rs` files, 8 of them, and the helpers they use,
   are where lazy growth shows. Run each alone (`heavy cargo nextest run -p <world> --test memory`), and
   apply the option above to each red.
5. **`hearth/scripts/fail_stop.py` stays:** the root's gate has no such check; run over the root
   workspace at this plan's base, it names two binaries of temper's, and those are the user's to
   rule on, not this plan's. Now that `cargo metadata` lists the whole workspace, keep the script to
   hearth's packages: compare each `manifest_path` with the script's own tree. Its docstring says
   hearth's binaries.
6. **`.config/gate/always.txt`:** add `hearth-render-world`, in its own line: `tests/render/tests/decoders.rs`
   reads the other worlds' test files by path, and that's the list's rule. `hearth/testing/recordings`,
   `repos` and `mirror` belong to no package, and `tests/builder`'s reads of `tests/render` are within
   its dependencies: neither adds a line.
7. Narrow tests, fmt, old forms, clippy, citations, then commit and push. Each ceiling's commit gives
   its peak.

## Report

The implementer's report, and: the counts and the sum of the acceptance; that the binary no longer
calls `service::worst_case`, and any startup check that went with it; every ceiling with its
measured peak, and every formula you replaced with one and why; each host's deadline check; what
the compiler asked beyond the table; every test that was red after the port and what it was.
