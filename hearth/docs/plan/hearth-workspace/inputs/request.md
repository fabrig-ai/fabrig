# Request: hearth joins the workspace

From fabrig/hearth, 2026-11-04. `hearth/` landed on main at `a829ee545c` with its history, as a fifth tree
outside the workspace, with its own `Cargo.toml` and `Cargo.lock` and a git dependency on an old skein
(`6fc0020d5`). The gate builds none of it. This plan makes it a member of the root workspace, built against
`skein/` by path and checked by the gate like any tree. **No behaviour of hearth changes,** and no design does:
this is a port, not new work.

The whole of it is one change the gate can only prove at the end: hearth isn't built until it joins, and it can't
join until it compiles against today's skein. Cut the port and the join as one task, unless you find a split the gate
can check at each step. The test layout can follow as its own task.

## 1. Port to current skein

A survey read the diffs from the pin to `1c76f0cdf`, on 11-04; **nothing was built**, so treat this as a list of leads
and let the compiler have the last word. It sized the port at about 150 lines in about 30 files, about 90% mechanical.
All 9 skein crates hearth uses still exist under the same names, in `skein/crates` or `skein/testing`.

| Item | Uses | What changed | Fix |
|---|---|---|---|
| `io::worst_case`, `FileIo::worst_case`, `Host::worst_case` | 1, 1, 7 impls | Removed; only `skein_io::valid` remains, so `service::worst_case` can't stay as written (`hearth-service-builder/src/lib.rs`, 475) | Small redesign |
| `Memory::Checked` | 10 | Compiles, but no longer enforces a bound: use `Memory::Within(ceiling)` | Small rewrite |
| Lazy container growth | 10 `memory.rs` | `with_capacity` no longer allocates, and a realloc counts old + new: exact bounds may fail at run time | Measure, then set ceilings |
| `io::MaxOut.events` | 2 fns | Field gone; `told` is capped by `limits.queue` and `push` asserts room, so it should become `Queue::uncapped()` (`tests/builder/src/fixtures/bounds.rs` assumes `MAX_OUT`) | Small rewrite |
| `Stage::new`, `has_room` | 1, 2 | `Stage::uncapped(limits)`; `has_room` is gone | Small rewrite |
| `tk::MaxOut` | 1 | Gone: a local struct | Small rewrite |
| `List/Queue/Stack/Map/Deadlines::worst_case` | 83 in 21 files | Renamed `price` | Rename |
| `Host::next_deadline` | 7 impls | The required method is `next_mechanism_deadline`; the default takes the min with policy | Rename; check each impl |
| `tk::worst_case`; `Outcome.heap` | 2; 1 | `tokenizer_bound`; `.peaks()` | Rename |

Take the new forms where a rename gets you there, and go no further: hearth's remaining `worst_case` and `MAX_OUT` uses
(`hearth/scripts/old-forms.txt`) are allowed old forms that may only fall (`skein/docs/foundation/programming-model.md`,
6.3). Adopting the newer model in full is out of scope. Lower the counts with `--write` for what the port removes.

## 2. Join the workspace

- **Root `Cargo.toml`:** the 30 hearth packages as members, and their workspace dependencies: hearth's path entries, plus
  `sha1` and `minijinja` (21 of hearth's 25 registry packages are already pinned identically; `memo-map` comes in with
  minijinja). Check each member's manifest against the root's `[workspace.dependencies]`, `[workspace.package]` and
  `[workspace.lints]`. Lints match key for key on the survey's reading.
- **Go:** hearth's `Cargo.toml`, `Cargo.lock`, `.config/nextest.toml`, and the git dependencies on skein. Keep
  `hearth/clippy.toml`, `hearth/tests/clippy.toml` and `hearth/rustfmt.toml` as the other trees do. Check whether
  `hearth/scripts/fail_stop.py` has anything the root's gate lacks; if it duplicates, it goes.
- **`.config/nextest.toml`:** the filters are package allowlists, so hearth's tests would run in **no** profile, silently,
  until they're added. Add a hearth clause to `default` and `fuzzy`, each with its own budget as the file's header
  describes, and a tier profile for `conformance_*` and `recorded_*`, which stay out of the gate
  (`domain/testing.md`, 2.8). Name it `<tier>-hearth`, so `tests/boundary` and the tier tools read it.
- **Tiers:** the package map (`tools/tiers/packages.toml`, generated: `tools/tiers/design.md`, 6) and `.config/fixture-readers.toml`
  (119 tracked `/fixtures/` paths under `hearth/`). **`tools/tiers/budgets.toml` and the root's timeouts are the user's:** see
  section 4.
- **`tests/boundary/tests/direction.rs`:** `TREES` is a line, each tree naming those before it. hearth names skein alone and
  nothing names hearth, which a line can't say. Make the check state it, with a test that catches a breach both ways.
- **`.config/gate/README.md`:** `hearth/clippy.toml` in its list of rules.
- **Root `AGENTS.md`:** five trees in one workspace, and `hearth/` beside the others. **`hearth/AGENTS.md`:** the "not a
  workspace member" bullet goes.
- **`env!("CARGO_MANIFEST_DIR")`:** about 12 sites in hearth; fix them as `skein_scratch::paths` does, where the
  rule applies to them (`skein/docs/foundation/testing-strategy.md`).

## 3. The test layout

The monorepo's rule is `tests/suite/` with `tests/suite/fuzzy/`, one counting allocator per suite
(`skein/docs/design/testing.md`), and the root's fuzzy profile selects "the `fuzzy` module of a package's `suite`
binary". hearth has 75 top-level test binaries and 20 `fuzzy_*.rs` across 9 crates. Move them: about 75 file moves, 24
`#[path]` or `mod` fixes and 9 `main.rs`. The tier binaries (`conformance_*`, `recorded_*`, `real`) stay separate. This also
cuts what hearth adds to the gate's link time, which is why it belongs in this plan. Tests move; they don't change.

## 4. Measurements for the user

Budgets are the user's, so this plan sets none on its own. Report, in the plan's final status:

- hearth's CPU and wall time for `default` and `fuzzy`, cold and warm, and its added build time on a full gate;
- what a skein-only change now costs, since nearly any skein change selects hearth by reverse dependency.

Until the user rules, follow the file's own rule: the workspace's `global-timeout` is the sum of each tree's, and hearth's
are 30 s focused and 120 s fuzzy (hearth's `.config/nextest.toml`, doubled provisionally on 10-22). Name this in the final
status as a figure awaiting the user.

## Rules

- A red test after the port is a finding about the port first: don't weaken a bound or a test to get past it. A bound that
  can only be set by measuring is set by measuring, with the measurement in the commit. If a test shows a skein bug,
  stop and ask.
- Nothing in `hearth/docs/design/` changes but the citations a move breaks, and the status where the port touches it.
- `skein/`, `smith/`, `jig/` and `temper/` change only where this request says. A change to skein that hearth seems to need
  is a question, not a task.
- `tools/citations/citations.py --all` stays at 0, and `tests/boundary/tests/outward.rs` passes.
- All parts must be in the plan when the cut ends.
