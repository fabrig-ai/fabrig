# hearth-workspace: contracts

The run's binding calls: what crosses tasks. A task's sketches build on these.

## 1. The members, in two stages

The port and the join are cut in two, because the gate can check each stage: the 14 step crates
below name no fake, no world and nothing above them, and their port is renames.

- **`join-steps` adds 14:** `hearth/crates/` `hearth-client-protocol-records`, `hearth-domain-build`,
  `hearth-domain-builder`, `hearth-domain-render`, `hearth-domain-store`, `hearth-domain-tree`,
  `hearth-lib`, `hearth-protocol-accounts`, `hearth-protocol-builder`, `hearth-protocol-config`,
  `hearth-protocol-git`, `hearth-protocol-mmdebstrap`, `hearth-protocol-template`,
  `hearth-protocol-tree`.
- **`join-worlds` adds 16:** `hearth/crates/hearth`, `hearth/crates/hearth-service-builder`;
  `hearth/testing/` `hearth-fake-git`, `hearth-fake-machine`, `hearth-fake-mmdebstrap`,
  `hearth-recordings`, `hearth-test-mirror`; `hearth/tests/` `accounts`, `builder`, `config`,
  `conformance`, `git`, `mmdebstrap`, `render`, `template`, `tree`.

`join-steps` removes `hearth/Cargo.toml`, `hearth/Cargo.lock` and `hearth/.config/nextest.toml`, and
adds every workspace dependency hearth had to the root. Between the stages the 16 sit unbuilt, and
their manifests' `workspace = true` lines read the root's.

## 2. `.config/nextest.toml`

One clause per profile, the last before `package(=fabrig-boundary)` in `default`, the last in
`fuzzy`. Its form moves with the stages, because a filter's `binary()` that matches no binary may
be refused:

| After | `default`'s clause | `fuzzy`'s clause |
|---|---|---|
| `join-steps` | `package(/^hearth(-\|$)/) & not (kind(test) & test(/^fuzzy::/))` | `package(/^hearth(-\|$)/) & (kind(test) & test(/^fuzzy::/))` |
| `join-worlds` | the same `& not binary(/^fuzzy_/) & not binary(/^(conformance\|recorded)_/)` | `package(/^hearth(-\|$)/) & ((kind(test) & test(/^fuzzy::/)) \| binary(/^fuzzy_/))` |
| `test-layout` | `package(/^hearth(-\|$)/) & not (kind(test) & test(/^fuzzy::/)) & not binary(/^(conformance\|recorded)_/)` | as after `join-steps` |

(`\|` is `|`; the table escapes it.) `real` and hearth's own `end_to_end` and `refusals` stay in
`default`, where hearth's own filter has them.

`join-steps` sets `default`'s `global-timeout` to `"150s"` and `fuzzy`'s to `"600s"`: the file's own
rule, each tree's budget summed, with hearth's 30 s and 120 s. The request sets these, and they
await the user's ruling (request, section 4). hearth's slow periods are `1000ms` focused and `10s`
fuzzy, as jig's are.

## 3. The tier profile

`conformance-hearth`, added by `conformance-tier`, selects
`(binary(/^conformance_/) | binary(/^recorded_/)) & package(/^hearth(-|$)/)`, with hearth's old
`conformance` profile's other keys. hearth's old `measure` profile goes with its file: nothing
reads it, and the request names no replacement.

## 4. The test layout

`skein/docs/design/testing.md`, 6, as `skein/tests/oauth/tests/suite/main.rs` has it:
`tests/suite/main.rs` holds the suite's one counting allocator, a `mod` for each focused file and
a `mod fuzzy { mod <name>; ... }`; `tests/x.rs` moves to `tests/suite/x.rs`, and
`tests/fuzzy_x.rs` to `tests/suite/fuzzy/x.rs`. `conformance_*`, `recorded_*` and `real` stay
top-level binaries. `hearth/crates/hearth/tests/` stays as it is.

## 5. Ceilings

`skein/docs/foundation/testing-strategy.md`, the `Memory` rule of its section 6: a ceiling is a named
constant beside the one test that checks it, set a quarter above the peak measured when it is made,
rounded up to two significant digits (a 79,584-byte peak takes 100,000), with no write mode. Exact
checks are for contracts only. The commit message gives each measured peak.

## 6. The plan's final status

`hearth/docs/plan/hearth-workspace/status.md`, written by `costs`, the plan's last task.
