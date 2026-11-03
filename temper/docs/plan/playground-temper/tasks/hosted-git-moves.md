+++
id = "hosted-git-moves"
part = "hosted-git"
kind = "migration"
needs = []
files = [
  "temper/crates/temper-fake-git-hosted/**",
  "temper/testing/temper-fake-git-hosted/**",
  "temper/tests/worker/checkout/src/hosted.rs",
  "temper/tests/worker/checkout/src/lib.rs",
  "temper/tests/worker/checkout/Cargo.toml",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "medium"
minutes = 60
+++

# The hosted fake git moves to `crates/`, on product crates alone

## Goal

`temper-playground`, a product crate built for wasm32, will host the fake git program, so its
hosted half moves from `temper/testing/` to `temper/crates/` and depends on product crates
alone; its fixtures stay with the tests, in the checkout world
(`temper/docs/design/playground.md`, 3; `temper/docs/design/domain/testing.md`, 4.3 and 7).
Nothing changes meaning: every reader's tests pass as before.

Acceptance:

- `temper/crates/temper-fake-git-hosted` exists, `temper/testing/temper-fake-git-hosted` does not.
  Its `[dependencies]` name no crate under `*/tests/*` or `*/testing/*`: they are `temper-fake-git`,
  `skein-io`, `skein-sim-world` and `skein-fake-checkout` (and nothing else unless the compiler
  needs a product crate; say so). Test kits are dev-dependencies only.
- Its `lib.rs` holds the program alone: `pub use temper_fake_git::{...}` as now, `start`, `git`,
  and `pub use skein_fake_checkout::git::Remote;` (the trait the one-process world's stories
  take from `tests/fake-checkout` today). `Inherited` comes from `skein_sim_world`, not
  `skein_world`. No `pub mod fixtures`.
- Its fixtures (`TOKEN`, `identity`, `spawn_of`, `CONFIG`, `Fault`, `Scenario`, `scenario`,
  `table`, `named`, `answer_case`, `masked`, ...) live in
  `temper/tests/worker/checkout/src/hosted.rs` as `temper_worker_checkout_world::hosted`, word for
  word but for their imports.
- The crate's own tests, the checkout world's, the one-process world's and the real loop's pass,
  with the same counts as on the trunk.
- The status entries and design lines below say `crates/`.

## Inputs

- `temper/docs/plan/playground-temper/notes/common.md`.
- `temper/docs/design/playground.md`, 3 (the table's `temper-fake-git-hosted` row).
- `temper/docs/design/domain/testing.md`, 4.3 ("Its crates", ~line 1783) and 7 (the layout).

## Owns and may change

- `git mv temper/testing/temper-fake-git-hosted temper/crates/temper-fake-git-hosted`, then:
  - `Cargo.toml`: `[dependencies]` as above; the rest of today's dependencies move to
    `[dev-dependencies]` only where the crate's own tests (`tests/suite/check.rs`,
    `tests/suite/transcripts.rs`) still use them, among them `temper-worker-protocol-git-world`,
    `skein-world`, `skein-shell`, `skein-lib`, `temper-worker-checkout-world` (for `hosted::*`).
    The `skein-fake-machine` path still resolves from `temper/crates/`. Keep its lints table.
  - `clippy.toml`: keep it, its header in `temper/crates/temper-fake-git/clippy.toml`'s words
    ("Ordinary Rust (programming-model.md, 10.2) that the binary ships ..."), its bans as they are
    now. If clippy then denies an item of `lib.rs`, a scoped `#[expect(..., reason = "...")]`.
  - `src/lib.rs` as above; `src/fixtures.rs` leaves the crate (moved, below).
  - `tests/suite/check.rs`, `tests/suite/transcripts.rs`: their `use` lines take fixtures from
    `temper_worker_checkout_world::hosted` and `Inherited` from `skein_sim_world` where
    `skein_world`'s re-export was used; nothing else changes.
- `temper/tests/worker/checkout/src/hosted.rs`: `fixtures.rs` moved, with `CONFIG` as
  `crate::forge::CONFIG` (it is that value today, `src/forge.rs:63`) and its `crate::` import of
  the hosted crate's re-exports taken from `temper_fake_git` directly. `src/lib.rs` gains
  `pub mod hosted;` with a one-line doc ("the hosted fake git's fixtures, shared by the worlds
  that host it"). `Cargo.toml` gains the normal dependencies `hosted.rs` needs (`temper-fake-git`
  and those of `skein-io`, `skein-fake-peers`, `skein-fake-forge-protocol`,
  `temper-worker-protocol-git` it does not already have).
- Root `Cargo.toml`: the member line (~245) and the `[workspace.dependencies]` path (~473) say
  `temper/crates/temper-fake-git-hosted`. `Cargo.lock` as cargo rewrites it.
- Reader files, each edited only in a path (`temper_fake_git_hosted::fixtures::X` becomes
  `temper_worker_checkout_world::hosted::X`) or a `use` line:
  - `temper/tests/real-loop/src/forge.rs` (~200 `table`, ~214 `CONFIG`), `src/github.rs` (~282),
    `src/cluster.rs` (~38), `src/case.rs` (~264), `src/fixtures/kept.rs` (~22); its `Cargo.toml`
    gains `temper-worker-checkout-world.workspace = true` if it lacks it.
  - `temper/tests/one-process/src/fixtures.rs` (~199), `src/stories.rs` (~139, ~141, ~179, and
    line 9: `use temper_fake_git_hosted::Remote;`), `src/lower/github.rs` (~137),
    `src/restart_control.rs` (~9); its `Cargo.toml` gains `temper-worker-checkout-world` and drops
    `temper-fake-checkout` if nothing else uses it (if something does, keep it and say so).
  - Any other site the compiler names, in the same way.
- Design lines, each edited only in the path (a departure from the implementer's text, stated):
  - `temper/docs/design/README.md`, ~264: the example `(\`testing/temper-fake-git-hosted\`)`
    becomes `(\`testing/temper-github-app\`)`.
  - `temper/docs/design/protocol/git.md`, ~679: `testing/temper-fake-git-hosted` becomes
    `crates/temper-fake-git-hosted`.
  - `temper/docs/design/protocol/README.md`, ~150: the same.
- Status, as `notes/common.md` says:
  - `temper/docs/design/domain/testing.status.md`: remove `4.3, 7:
    \`temper-fake-git-hosted\` in \`crates/\`; it is in \`testing/\`.` and nothing else.
  - `temper/docs/design/playground.status.md`: remove `; \`temper-fake-git-hosted\` is still in
    \`testing/\`` from the 3 entry, so it reads "3: `temper-playground`,
    `temper-playground-world` (`tests/playground`) and `temper-client-shell-web`'s feature
    `playground`. 4: ...".

If the move would need a change of behaviour anywhere (a fixture that cannot be moved word for
word), stop: that is a question.

## Steps

1. The move, the manifests and `lib.rs`; `heavy cargo check -p temper-fake-git-hosted --tests`.
2. `hosted.rs` and the readers; `heavy cargo check -p temper-worker-checkout-world -p
   temper-one-process-world -p temper-real-loop-world --tests`.
3. `heavy cargo nextest run -p temper-fake-git-hosted`; `-p temper-worker-checkout-world`;
   `-p temper-one-process-world`; `-p temper-real-loop-world`; `-p fabrig-boundary`. Compare each
   count with the trunk's.
4. Proof of the portable target, once: `heavy cargo build --target wasm32-unknown-unknown -p
   temper-fake-git-hosted`. If it fails in a dependency, report the first error under `## Left`
   with the crate; it is not this task's (`temper-playground`'s task takes it). If the builder has
   no wasm32 target, say so.
5. The design and status lines; the citations check.
6. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-fake-git-hosted -p
   temper-worker-checkout-world -p temper-one-process-world -p temper-real-loop-world
   --all-targets -- -D warnings`. Commit the move and readers, then the docs; push.

## Report

As the implementer's text says, and: the final `[dependencies]` of the moved crate; each reader
file edited; the wasm32 build's result.
