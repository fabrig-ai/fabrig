+++
id = "box-draws-link"
part = "box-wiring"
kind = "feature"
needs = ["serve-takes-secret", "box-first-block"]
files = ["temper/crates/temper/src/box_command.rs", "temper/crates/temper/src/fake_backends.rs", "temper/crates/temper/src/tests/box_kinds.rs"]
effort = "high"
minutes = 50
+++

# `temper box` draws the first link, and hands over its opening lines

## Goal

The wiring. With its listeners open, the box draws the secret and builds the link. A first run's line goes to `box.log`, and `--fake-backends`' `box:` lines follow the block. After this, `temper box` behaves as the design says.

Acceptance:

- `first_link` draws one secret with `skein_shell::crypto_seed()`, after `open_listeners`, where `named.link` holds, `named.clients` is not at port 0, and no step is `Kind::Link`. Otherwise it gives `None`.
- The URL is `temper_service::clients::first_link_url(address, named.public.as_ref(), &secret)`.
- A URL past 4096 bytes refuses the start, in the words `serve: sign-in link too long`, before any block.
- A port taken still refuses before any block, with nothing on stdout: `suite/box_listeners.rs`, about lines 103-115, stays as it is and green.
- `First run: wrote the configuration, <path>` is no longer `eprintln!`ed: it reaches `Settings.log_lines`.
- `--fake-backends`' `box:` lines are no longer `eprintln!`ed in `prepare`: the same lines, in the same order, reach `Settings.after_block`. A refusal during `prepare` is still printed before any block.
- Tests of `first_link` in `temper/crates/temper/src/tests/box_kinds.rs`, for each rule above, name no secret in a message.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.2, *The sign-in block* and *Where the box draws none*; 7.5.
- `temper/docs/plan/box-link-first/notes/contracts.md`.
- `temper/docs/plan/box-link-first/notes/common.md`.

## Owns and may change

Owns `temper/crates/temper/src/box_command.rs` and `fake_backends.rs`, and `temper/crates/temper/src/tests/box_kinds.rs`.

Reader and neighbour files, call sites only:

- `temper/crates/temper/tests/suite/startup.rs`, about lines 1515 and 1529: `configuration`'s return value, if its shape changes.
- `temper/crates/temper/src/fake_backends/tests.rs`: `print`'s tests and `peer_lines`, if the function's name or return changes.

Left for later tasks, to report under `## Left`: reds in package `temper`'s tests reading moved words or taking the link for serve's being up (`box-readers-log`, `box-readers-early`).

Options, each pre-approved:

- `configuration` may return a third value, the first run's line, or a small struct. Its callers are the two above.
- `configuration_steps` has readers in three test files: keep its signature. If `first_link` needs `Named`, call `super::setup::check` again in `run_configuration`, or add a sibling function that returns it.
- The owned `Fd`s and refusals of `run_configuration` keep their shape: a refusal after `open_listeners` still closes them.

## Steps

The outer shape:

```rust
/// The first link, where the box prints one (protocol/setup.md, 7.2).
fn first_link(
    named: &temper_service::startup::Named,
    steps: &temper_forge_setup::Steps,
) -> Result<Option<temper_box::host::FirstLink>, String>;

pub(crate) struct Opening {
    pub log_lines: Box<[Box<[u8]>]>,
    pub after_block: Box<[u8]>,
}
pub(crate) fn run_configuration(
    directories: &super::Directories,
    bytes: &[u8],
    held: Vec<(Fd, std::num::NonZeroU16)>,
    first: Box<[temper_forge_setup::Step]>,
    echo: Echo,
    opening: Opening,
) -> Result<Exit, String>;
```

Write the `first_link` tests first, then wire it. Narrow runs: `heavy cargo nextest run -p temper --lib -E 'test(first_link)'` and `heavy cargo nextest run -p temper --test suite -E 'test(a_fresh_box_holds_serves_clients_port)'`. Then run `suite/box_listeners.rs`'s other tests alone, and read a box's output by hand once: `heavy cargo build -p temper`, then `timeout -s TERM 5 target/debug/temper box --config ... --data ... --cache ... >OUT 2>ERR` on an empty scratch directory in your slot, deleted before you push. Check that `OUT` and `ERR` hold the block alone, and `DATA/box/logs/box.log` the rest. Then fmt, clippy and the `--lib` tests of `temper`. Commit and push.

## Report

As your text says, with what the hand run printed, the secret cut from it, and every `temper` test you saw red, for `box-readers-log`.
