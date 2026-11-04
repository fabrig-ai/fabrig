+++
id = "link-forge-setup"
part = "transport"
kind = "feature"
needs = ["link-crate"]
files = ["skein/crates/skein-forge-setup/**", "skein/tests/forge/setup/**", "skein/docs/design/forge-setup.status.md"]
effort = "high"
minutes = 75
+++

# skein: forge-setup's host on `Link`

## Goal

`skein-forge-setup/src/host/connection.rs`' `Connection` stands on `Link` and `Upload`, and sends `close_notify` on a graceful
close.

Acceptance:

- `rg -n 'tls_up|tls_down|close_sent' skein/crates/skein-forge-setup/src -g '!*test*'` finds nothing.
- A plaintext connection's `Close` still comes from `pump`, as `host/tests.rs` pins it.
- `pump` still makes at most one io request (`host/connection.rs` says room for one is reserved): it returns after
  `link.close`.
- The pin holds, and an ended run's slot comes back within `skein/docs/design/forge-setup.md`, 6.5's bound.
- `connection.rs`'s non-test lines fall.
- `forge-setup.status.md`'s 6.5 sentence is gone.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `temper/docs/plan/debt-9/notes/link.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/link.md`, 3; `skein/docs/design/forge-setup.md`, 6.3 and 6.5.

## Owns and may change

- Owns: `skein-forge-setup/src/**`, `skein/tests/forge/setup/**`.
- `skein-forge-setup/Cargo.toml`: one line, `skein-link.workspace = true`, and `Cargo.lock`.
- `skein/docs/design/forge-setup.status.md`: that sentence goes, and the date line.
- `closing: Option<bool>` and `closed` stay yours, as the host reads them. `close_sent` gives way to `asked: Option<bool>`:
  what the link was last told, so a close goes once and an abort follows a graceful close. `abort` has no `env`, so it still
  only sets `closing`.

## Steps

1. `Connection` per `notes/link.md`.
2. `pump` while closing:

   ```rust
   if let Some(abort) = self.closing {
       if self.asked != Some(abort) {
           self.asked = Some(abort);
           self.link.close(&link_env(env), abort, io);
           return true;
       }
       return match self.link.pump(io) {
           Moved::Nothing => false,
           Moved::Within | Moved::Sent | Moved::Above(_) => true,
       };
   }
   ```

3. `ready`, closing: `self.asked != self.closing || self.link.has_work()`. If `ready` stayed true with nothing to move, the host
   would spin, so read it twice.
4. `next_deadline` in `host.rs`: add each connection's `link.next_deadline()`. `fire` in `drive_connection`.
5. The pin: `skein/tests/forge/setup/src/tls.rs`' `Carrier` holds the server. A story in that world asserts `closed` once a
   reply is taken.
6. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-setup --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-forge-setup` and `skein/tests/forge/setup`'s package; `heavy cargo check --tests -p
   temper-service`, its reader.
7. The citations check. Commit and push.

## Report

`notes/link.md`'s break, the common note's items, the heap ceiling's figure before and after, and the net lines.
