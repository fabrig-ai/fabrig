+++
id = "forge-sockets"
part = "connection-worlds"
kind = "feature"
needs = ["forge-kit", "connection-residue"]
files = ["skein/testing/skein-forge-kit/src/sockets.rs", "skein/testing/skein-forge-kit/src/lib.rs", "skein/testing/skein-forge-kit/src/machine.rs", "skein/testing/skein-forge-kit/src/hooks.rs", "skein/testing/skein-forge-kit/src/joint.rs", "skein/testing/skein-forge-kit/src/joint/**", "skein/tests/forge/machine/src/memory.rs", "skein/tests/forge/machine/src/hand.rs", "skein/tests/forge/protocol/src/github.rs", "skein/tests/forge/setup/src/host.rs", "skein/tests/forge/setup/src/memory.rs", "skein/tests/forge/setup/src/closing.rs"]
effort = "high"
minutes = 70
+++

# One io harness for the forge worlds

## Goal

Skein D030's last clause. The forge worlds each repeat the same arms: a `Connect` becomes a `Connecting` and a `Connected` or refusal; a `Stream` reaches the peer; `Close` and `Abort` end it. `skein_forge_kit::sockets` holds them once (`skein/docs/design/fake-forge.md`, 10.6).

Acceptance:

- ```rust
  /// The owner -> peer table every forge world plays its io through.
  pub struct Sockets<P> { /* owner -> P */ }

  /// What a world's peer does with what the table hands it. The world keeps its own cuts, pauses and settlements.
  pub trait Peer { fn take(&mut self, /* the stream's bytes */); fn answer(&mut self) -> /* bytes the owner has room for */; }

  impl<P: Peer> Sockets<P> {
      pub fn request(&mut self, request: skein_io::Request, make: impl FnMut(Addr) -> Option<P>, /* the events out */);
      pub fn pump(&mut self, /* the events out */);
  }
  ```

  It dispatches and nothing else. The shape is yours inside that: worlds are ordinary Rust, and generics are fine here.
- Its readers: `machine.rs`'s `Machine`, `hooks.rs`'s `Listener`, `memory.rs`'s `World`, `joint.rs`'s `Joint` and `joint/github.rs`, `protocol/src/github.rs`'s `Github`, `setup/src/host.rs`'s `HostWorld`, `setup/src/memory.rs`'s `Metered`, `setup/src/closing.rs`'s `Closing`, and `hand.rs`'s `Harness`. Each one's copy of those arms goes.
- **A seed replays.** `Joint` draws from its rng at each `Connect`, and `HostWorld` pops its script there: `make` keeps each draw where it is, and no event moves. Any test that goes red on an order or a trace is this task's fault: fix the harness, never the test.
- **A harness that can't share dispatch without moving an event** keeps its own arms: at most two, each named in the report with the event that would move. Past two, stop and ask.
- Net lines fall by at least 100: the survey expects 200 to 300 from the arms.
- `heavy cargo nextest run -p skein-forge-kit -p skein-forge-machine-world -p skein-forge-protocol-world -p skein-forge-setup-world` and `--profile fuzzy` on the same pass, with no assertion changed.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/hand.md`
- `skein/docs/design/fake-forge.md`, 10.6
- `skein/docs/design/testing.md`, 6

## Owns and may change

- **Owns:** the files in the header.
- **`conformance/src/component/hosted.rs`** holds a real `Io` over a kernel, and `github/src/live.rs` is the live boundary. Neither is a forge world's scripted wire: both stay.
- **A world's own tests** don't change. If a reader's public method disappears, the world keeps it, calling `Sockets`.

## Steps

1. `Sockets`, with `Machine` on it; the tests.
2. One reader at a time, each a commit, each with its world's tests and fuzzy.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy` on the four packages `--all-targets -- -D warnings`. Commit and push.

## Report

The common note's items; each reader's lines before and after; any harness left, and why; fuzzy's time before and after. A refactor under the worlds' own tests: no new rule to break by hand.
