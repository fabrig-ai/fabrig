+++
id = "playground-world"
part = "world"
kind = "feature"
needs = ["story-a", "one-process-composed"]
files = [
  "temper/tests/playground/**",
  "temper/docs/design/playground.status.md",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "high"
minutes = 90
+++

# `tests/playground`: the playground's world, harness-free

## Goal

`temper-playground-world` (`tests/playground`) is the playground's world walked natively
(`temper/docs/design/playground.md`, 3 and 9). Its walk through the web client's host waits for
skein's visitor and bridge; this task makes the crate and checks section 4's world as the browser
will run it: the composition without a harness, no box, calm, one seed, the printed link on the
service's standard output, the clients' listener at the document's address.

Acceptance:

- `temper/tests/playground`, package `temper-playground-world`, a test world (`AGENTS.md`, 2:
  ordinary Rust under `temper/tests/clippy.toml`), its library `src/lib.rs` with a module doc
  citing `temper/docs/design/playground.md, 9` and the helpers its tests share (a world from a
  story stepped until it waits idle, its printed lines read).
- Tests in `tests/suite/world.rs` (binary `suite`) pass:
  - `story_a_starts_with_no_box_calm_and_prints_its_link`: `temper_playground::world(A.start(7,
    wall))` stepped until it waits idle with nothing due (bounded; past the bound, fail naming the
    step): the service's standard output holds exactly one line, the sign-in link; the world holds
    no box process; no process ended; the simulator's configuration has no fault and its trace
    off.
  - `the_clients_listener_is_at_the_documents_address`: the address the service's clients'
    listener is bound at is the one story A's `deployment.json` names, and the printed link's
    authority is that address.
  - `one_seed_makes_one_world`: two worlds from seed 7 end idle with equal digests and equal
    printed lines; a world from seed 8 prints a different link.
- Root `Cargo.toml`: its member line right after `"temper/tests/fake-checkout",`.
- The status entry below is edited.

## Inputs

- `temper/docs/plan/playground-temper/notes/common.md` and `notes/contracts.md`, C3 and C4.
- `temper/docs/design/playground.md`, 4 and 9.
- `temper/docs/design/domain/testing.md`, 7 (the layout's `tests/playground` line).

## Owns and may change

- The new crate: `Cargo.toml` (`[lib] test = false` as the other worlds; lints as
  `temper/tests/one-process/Cargo.toml`'s table; dependencies `temper-playground`,
  `skein-sim-world`, `skein-sim`, `skein-lib`, `skein-io` and what the tests read), `src/lib.rs`,
  `tests/suite/main.rs`, `tests/suite/world.rs`.
- Root `Cargo.toml` (the member line), `Cargo.lock`.
- If a check needs a read the composition does not give (the listener's bound address, the box's
  absence), take it from the world (`skein_sim_world::World`'s machine or sockets) as other worlds
  do; if no product API gives it, stop and ask, naming the read.
- Status, as `notes/common.md` says: `temper/docs/design/playground.status.md`, **Not yet**: the
  3 item's `\`temper-playground-world\` (\`tests/playground\`) and ` goes, so it names
  `temper-client-shell-web`'s feature `playground` alone (keep the rest word for word); 9's
  "the native walk" stays.

## Steps

1. The crate and its helpers; `heavy cargo nextest run -p temper-playground-world`.
2. The status edit; the citations check.
3. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-playground-world
   --all-targets -- -D warnings`. Commit; push.

## Report

As the implementer's text says, and: each test's time, and the world's steps to idle.
