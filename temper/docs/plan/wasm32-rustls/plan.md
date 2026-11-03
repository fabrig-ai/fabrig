# wasm32-rustls: plan

Draft: `temper-playground` builds and passes clippy for `wasm32-unknown-unknown`
(`inputs/request.md`). `temper/docs/design/playground.md`, 3 and 9 put that build in the gate.
It fails in rustls 0.23.41 (E0599, no `UnixTime::now`, `src/ticketer.rs`), which is rustls's
`std` path.

## Parts and tasks

One part, one task, released in the first and only push.

- **rustls-std** (slice 1): `tasks/rustls-std.md`, no needs. It removes `std` from the root
  `Cargo.toml`'s rustls line. The two temper crates whose tests use rustls's std API ask for it
  on their dev-dependency lines. Then it checks `skein-tls`'s wasm32 clippy and
  `temper-playground`'s, and natively `temper-playground`, `skein-tls` and the two std users'
  tests. If `temper-playground`'s wasm32 clippy then stops elsewhere, the task keeps the cut,
  reports the next blocker in full and fixes none of it.

## Decisions

- **The edge** (`cargo tree --target wasm32-unknown-unknown -p temper-playground -i rustls -e
  features`): `skein-tls` is the only crate of the wasm32 normal graph that names rustls, and it
  inherits the root's `rustls = { ..., features = ["ring", "std", "tls12"] }`. Fourteen crates of
  that graph reach `skein-tls`: jig's configuration, skein's forge and LLM connections, OAuth and
  fakes, smith's agent service and shell, and temper's service. It is not dead weight to gate
  away: `tls.md`, 4 has it build on the portable target and refuse there, and it holds RS256.
  So the request's option 1 applies to the feature, not the crate: `std` becomes native only.
- **The fault is the workspace merge.** skein's own line, before the trees became one
  workspace (commit 199faf550e), was `features = ["ring", "tls12"]`, with the comment "rustls is
  pinned and built without std". Temper's line had `std`, and the merged line kept it. The
  design never changed: `skein/docs/design/tls.md`, 1 ("builds without std") and 4 ("rustls's
  `std`, whose clock that target lacks, is native only"). The task brings the code back to it.
- **Natively nothing changes.** `skein-fake-peers` asks for `std` on `cfg(unix)`, and skein's
  test worlds (`skein-tls-world`, `skein-llm-world`, `skein-fake-peers-tests`) ask for it
  explicitly. With the root line's `std` removed in a throwaway edit, `cargo tree` shows `std`
  natively for `temper-playground`, `temper` and `temper-forge-setup-shell`, none for
  `skein-tls` alone, and none on wasm32 without dev-dependencies. `Cargo.lock` did not change.
  `skein-tls` passes its own time provider and disables resumption, and its tests use
  `builder_with_details` and the unbuffered connections, so it needs no `std`. That was already
  true before the merge.
- **The two temper dev-dependency lines ask for `std`** (`temper`, `temper-forge-setup-shell`):
  their tests use `StreamOwned`, `ServerConnection` and `ClientConnection`. Today they get `std`
  only by unification through `skein-tls-world`. Naming it is skein's form.
- **The gate's form is without `--all-targets`.** Among `temper-playground`'s dev-dependencies,
  `skein-tls-world` asks for `std` on every target. The gate's step clippies `-p <package>`
  without `--all-targets`, as does the request's command, so dev-dependencies are not built for
  wasm32.
- **No design, no status.** `tls.md` names the dependency, and it already says what the fix
  makes true. `tls.status.md` recorded no gap for it, and a status file records only gaps, never
  history (`DESIGN.md`, 2), so it stays as it is. `.config/gate/wasm32.txt` and
  `playground.status.md`, 9 stay: the request leaves the list's line to agents/loop, and plan
  playground-browser's status-nine edits that entry.
- **A next blocker is reported, not fixed.** wasm32-temper and wasm32-temper-2 each found one
  blocker. If this one is cut and clippy stops in another crate, the task reports every error
  and warning of a `--keep-going` run, so a further try is cut from the whole list.
- **rustls without `std` on wasm32 is the one risk the survey could not measure**, since a lead
  builds nothing. If it fails to build, the task asks, with the options `web` time (a departure
  from `tls.md`, 4) or another pin.

## Overlaps with the drafts beside this one

- **playground-browser** (integration): its merged task playground-worker (commit 64bac5b5a3)
  works around the same error another way. It adds `rustls-pki-types = { version = "1.15" }`
  to the root `Cargo.toml`, on the line right after rustls's, and gives `skein-tls` on wasm32
  `rustls-pki-types` with `web`, a `Date.now` clock for rustls's `std` path. This task's edit of
  the rustls line is adjacent to that insertion, and `git merge-file` reports a conflict there.
  The resolution keeps both: this task's rustls line, then theirs. Unavoidable, because the
  root line is the fault. Once both land, their `web` line is redundant: rustls without `std`
  never calls `UnixTime::now`. It also brings `web-time` into step code on wasm32, which
  `tls.md`, 4's list of the exception's crates does not hold. Removing it is for after both
  land; the report says so for the director. `skein/crates/skein-tls/Cargo.toml` is not touched
  here. Its `browser-walk` task (not merged) also lists the root `Cargo.toml`, with the same
  rustls lines as its trunk. `temper-client-shell-web` and `tests/playground` are left alone, as
  the request says.
- **configuring-p3, public-reading-build** list no file this task touches. Neither
  `temper/crates/temper/Cargo.toml` nor `temper/crates/temper-forge-setup-shell/Cargo.toml` is
  in any of their lists. The queued drafts list no files yet.

## Critical path

rustls-std alone: about 45 implementer-minutes, most of them two wasm32 clippy builds and the
native test builds (`temper-playground`, `skein-tls`, `temper`'s suite binary,
`temper-forge-setup-shell`'s).
