+++
id = "setup-unusable"
part = "forge-setup"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-setup/src/host/limits.rs", "skein/crates/skein-forge-setup/src/tests/host.rs", "temper/crates/temper-service/src/setup/settings.rs", "temper/crates/temper-service/src/tests/setup.rs"]
effort = "medium"
minutes = 30
+++

# forge-setup names an unusable io or TLS limit

## Goal

`skein-forge-setup`'s `host/limits.rs` answers `Unusable::Zero` for an unusable io or TLS limit, and
judges io by `is_usable` alone, so a limit whose capacities overflow passes. (Bug 8's third half,
re-checked: holds; and bug 15's forge-setup line.)

Acceptance:

- `Unusable` gains `Io` and `Tls`, unit variants with doc comments.
- `limits.rs` splits the check: `!skein_io::valid(&l.io)` is `Unusable::Io`, `!skein_tls::client::valid(&l.tls)`
  is `Unusable::Tls`. Other `Zero`s stay.
- The test below fails on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/forge-setup.md`, 6.1, 6.5.
- `skein/docs/design/io.md`, 2 (`skein_io::valid`).

## Owns and may change

- Owns: `src/host/limits.rs`, `src/tests/host.rs`.
- temper, the lines a new required variant forces (the producer's branch carries them; AGENTS rule "a
  branch stays in its tree, but for a required field"):
  - `temper/crates/temper-service/src/setup/settings.rs`: near 229 add `Unusable::Io => &[b"io"]` and
    `Unusable::Tls => &[b"forge_component.tls.read", b"forge_component.tls.send"]`; near 264-276 both
    variants in the inner match.
  - `temper/crates/temper-service/src/tests/setup.rs`: near 572, two rows in
    `every_host_unusable_arm_has_an_operator_path`.
- No other reader: `host/tests.rs`'s `valid_refuses_unrepresentable_tls_intake` only asserts agreement;
  `states = 0` and `runs = 0` stay `Zero`. skein-forge-connection's like shape (`src/limits.rs`) is
  not this bug: `## Left`.

## Steps

1. In `the_host_refuses_unusable_settings_naming_each` (`src/tests/host.rs`), raise the list's capacity
   and add `l.io.intake = 0` → `Unusable::Io`, `l.tls.read = 0` → `Unusable::Tls`, and `l.io.sockets =
   1 << 29` → `Unusable::Io`. The variants do not exist yet, so add them first, with the check unchanged:
   run on that and quote the failure (`Zero`, and `Ok` for the sockets row).
2. Split the check; add temper's arms.
3. `heavy cargo nextest run -p skein-forge-setup`, `-p skein-forge-setup-world`, and `heavy cargo
   nextest run -p temper-service --lib -E 'test(/setup/)'`. `cargo fmt`, `python3 -I -B
   skein/scripts/old_forms.py`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p
   skein-forge-setup -p temper-service --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the test showed on the old code.
