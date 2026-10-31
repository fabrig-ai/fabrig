+++
id = "echo-world"
part = "world-surfaces"
kind = "migration"
needs = ["echo", "world-kit", "counts"]
files = ["skein/tests/echo/**"]
effort = "medium"
minutes = 30
+++

## Goal

tests/echo's two `Process` dispatchers implement `skein_shell::Forward`
instead of forwarding `Host` method by method, and its referees drop the
overrides the trait's defaults now give
(`skein/docs/design/draft/component-kit.md, 3.5`).

Acceptance:

- `tests/echo/src/proc.rs` and `tests/echo/tests/end_to_end.rs`: `impl
  Forward for Process` with `host` and `host_mut`; no `impl Host for
  Process` left in tests/echo. The `forwarding_host` count falls by 2.
  `Proc`'s variants each hold one `Host`: the echo's `Echo`, and a
  world-owned client adapter in `tests/echo/src/proc.rs` (see Answers).
- Trivial `act`, `next_deadline` and `overdue` overrides in tests/echo's
  referees are gone.
- `skein-echo-world`'s tests pass.

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md` and `notes/migrations.md`
  (the world surfaces).
- `skein/crates/skein-shell/src/drive.rs` (`Forward`, `Host`);
  `skein/testing/skein-world/src/referee.rs` (the defaults).
- tests/world's dispatchers, already moved by task world-kit, as the model
  (`skein/tests/world/tests/suite/end_to_end.rs`).

## Owns and may change

- Owns `skein/tests/echo/`.

## Steps

1. Move both dispatchers onto `Forward` (proc.rs's matches `self`: give it
   the two accessors).
2. Drop the trivial referee overrides.
3. `cargo fmt`; `heavy cargo clippy -p skein-echo-world --all-targets --
   -D warnings`; `heavy cargo nextest run -p skein-echo-world`.
4. `old_forms.py --write`; commit and push.

## Report

As common.md says, with the lines each dispatcher had before and after.

## Answers

2026-10-31. `skein_echo_client::Client` is not a `Host`: may echo-world add
a client adapter and change `Proc::Client`'s payload to hold it, or keep
proc.rs's dispatcher? Add the adapter. `Proc` is the world's own type,
used only inside `skein/tests/echo/` (examples/echo/shell's dev-dependency
names no `Proc` variant; no other tree uses the crate), so its payload is
this task's to change, not a public interface under AGENTS.md 7. Do not
touch skein-echo-client: the step crate stays free of the shell. In
`tests/echo/src/proc.rs` write a concrete struct, say `ClientHost`, holding
the `Client` and its `client::Limits`, with an `impl Host` that calls the
client's own methods (`client::iterate`, `operations(&limits)`, and so on),
as `skein_echo_shell::Echo` does for the service; it is a host, not a
forwarder, and the `forwarding_host` form does not count it. `Proc` becomes
`Echo(Box<Echo>)` and `Client(Box<ClientHost>)`: the echo variant's
`limits` goes, `Echo::operations` already prices the ring from its own.
`impl Forward for Proc` matches the two variants; update the patterns in
`referee.rs` and `scenarios.rs` and the `as_echo`/`as_client` accessors.
Two forwarded methods change: `exit` now reaches `Echo::exit` (exit code 1
on a service failure) where the dispatcher took the default, which is the
drift the design names (3.5); `next_deadline` and `state_digest` already
forwarded. If a scenario then fails on its exit, report it with its log,
do not mask it. The end-to-end conversion already done stands. The
referees have no trivial `act`, `next_deadline` or `overdue` overrides at
this base, so that acceptance line holds with none removed: say so in the
report, naming the overrides kept and why, as the run did.
