# debt-24: forge-connection's tests move to the machine world (skein D030)

Every `hand-*` and `connection-*` task names this note. A read-only survey at this plan's base found these numbers: re-measure first, and find code by name.

## The shape

`Harness` (`skein-forge-connection/src/tests/fixtures.rs`) plays io over a scripted wire, and 408 tests in the crate's `src/tests/` reach it. It moves almost verbatim to `skein/tests/forge/machine/src/hand.rs`, and its tests move onto it:

```
skein/tests/forge/machine/src/hand.rs                   Harness, CallHeap, and the crate's fixtures its tests read
skein/tests/forge/machine/tests/suite/hand.rs           mod forgejo; mod github;
skein/tests/forge/machine/tests/suite/hand/forgejo.rs   a mod line per moved Forgejo file, in hand/forgejo/
skein/tests/forge/machine/tests/suite/hand/github.rs    a mod line per moved GitHub file, in hand/github/
```

`Machine` keeps its scripted face beside it. `hand` keeps the crate's bytes: the crate's `response()` says `OK` with a `Date`, the world's says `Answer` with none, and a moved test keeps the crate's.

## The rules

- **A move preserves what it asserts.** `git mv` each file and change only imports: `crate::` to `skein_forge_connection::`, `super::fixtures` to `skein_forge_machine_world::hand`, `alloc` lines. One one-shot untracked script does it; don't hand-compress. Test names stay. Your report compares the sorted `assert` lines of the moved tests with the base's, and says they're the same.
- **What's left of a mixed file stays in the crate, under its name.** A test that uses `Harness` moves, and one on `ConnectionHarness`, `HookHarness` or neither stays. `ConnectionHarness` and `HookHarness` step one entity with events, which is what `skein/docs/foundation/testing-strategy.md`, 2.1 asks. They keep their names: the acceptance search is `rg -lw Harness`, and the item's looser search is exempted for them.
- **A helper a remaining crate test still reads stays in the crate** until `connection-residue`, with its copy in `hand.rs`. That copy is temporary, and `connection-residue` removes it.
- **Literal bytes stay scripted.** `status(503)`, a malformed body and `acme/widgets` are the test's point, and no fake serves them. What plays the fake goes through `skein_fake_forge_github::inline` (`skein/docs/design/forge-connection.md`, 10). So `github_files/fake.rs` merges into the world's `github::Fake`, and nothing else goes onto a fake: that would rewrite assertions.
- **Heap ceilings move with their tests** (`artifact`, `github_writes_heap`, `puller`). A peak that shrinks is updated, and one that grows stops and asks.
- **Over budget is red.** These tests stay in the focused suite, where `skein/docs/design/testing.md`, 7's figures hold (a world test under 1 s, `tests/forge` within half of each suite's). Your report gives the moved tests' total time. Moving them back to escape a budget is forbidden: if a figure is passed, stop and ask, with the measurements.
- **`trust()`'s `../../tests/tls/fixtures/root.der`** is relative: it follows the file. `.config/fixture-readers.toml` names the crate for that row, and the world joins it.

## Which file goes where

| File | Harness tests | Reach | Task |
|---|---|---|---|
| `abort`; `free_rate`'s `forgejo_ignores_github_rate_headers` | 5, 1 | public, by hand | `hand-face` |
| `contents`, `writes`, `reads`, `connection`, `artifact` | 23, 17, 22, 15, 1 | public, by hand; `reads` and `connection` hold four `drv` | `hand-forgejo` |
| `src/tests.rs`' six `Harness` sites | 6 | public | `hand-forgejo` |
| `github_start`, `github_installations`, `runtime` | 46, 17, 10 | public, by hand | `hand-github` |
| `github_installation_loss`, `github_renewal`, `puller` | 68, 39, 26 | read private state | `hand-rows` |
| `github_calls`, `github_unknown_repository`, `github_logs`, `github_files`, `github_writes_heap`, the rest of `free_rate` | 47, 37, 12, 7, 2, 13 | seed private state | `hand-seeded` |

Stays in the crate: `dialects`, `hooks`, `limits/tests.rs`, `installations/tests.rs`, `app/tests.rs`, `component/github/tests.rs`, `connection/**/tests.rs`, `component/tests.rs`, and what's left of the mixed files.

## The seam

`connection-seam` writes it. The items are `Component`'s `test_github`, `test_github_mut`, `test_calls_ready`, `test_local`, `test_last_connection`, `test_github_handoffs`, `test_open_call`; `crate::installations::{GitHub, Row, Tokens, Clock, Accepted, HeldToken, Reading, Diff}`; `puller`'s state; `crate::connection::{Connection, Phase}`; `hooks::connection::Hooked`; `component::entry::ForgeEntry`; `limits::installation_bounds`. `skein/docs/design/forge-connection.md`, 10 says what it's for: only `skein-forge-machine-world` turns it on.
