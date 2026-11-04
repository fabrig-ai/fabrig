+++
id = "link-oauth"
part = "transport"
kind = "feature"
needs = ["link-crate"]
files = ["skein/crates/skein-oauth-accounts/**", "skein/tests/oauth-accounts/**", "skein/docs/design/oauth.status.md"]
effort = "high"
minutes = 70
+++

# skein: oauth-accounts on `Link`, the thin story

## Goal

`skein-oauth-accounts/src/exchange.rs`' `Web` stands on `Link`, with `Upload`. This is the first owner, as it already has the
right sequence, so nothing a test sees should move but the close's bound.

Acceptance:

- `rg -n 'cipher|tls_events|close_sent' skein/crates/skein-oauth-accounts/src -g '!*test*'` finds nothing. `tls::Client::new`
  stays where `Web::new` builds the client it hands to `Link::new`.
- `close_progress` is gone, folded into one pump.
- A graceful close's `close_notify` reaches a TLS server, pinned, and the component's `next_deadline` holds the link's.
- `exchange.rs`'s non-test lines fall.
- `oauth.status.md`'s 6.6 entry is gone.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `temper/docs/plan/debt-9/notes/link.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/link.md`, 3; `skein/docs/design/oauth.md`, 6.6.

## Owns and may change

- Owns: `skein-oauth-accounts/src/**`, `skein/tests/oauth-accounts/**`.
- `skein-oauth-accounts/Cargo.toml`: one line, `skein-link.workspace = true`, and `Cargo.lock`.
- `skein/docs/design/oauth.status.md`: the 6.6 sentence goes, and the date line.
- `component/accounts.rs` reads `exchange.web`'s socket: it uses `web.socket()` through a `pub(crate)` method on `Web` or
  on `Link`, whichever reads shorter.

## Steps

1. `Web` per `notes/link.md`. `Phase::Closing` stays while HTTP's side reads it, and `sent` stays yours, on plain `Send`s.
2. `Web::close` is `self.link.close(link_env(env), abort, io)`, keeping `abort`'s clearing of `answer` and `failure`.
3. `progress`' order is http events, `link.pump`, plain; once closing, `link.pump` alone.
4. `Upload` for `upload` and `offset`.
5. The component's `next_deadline` and `fire` per the note.
6. The pin: a story in `skein/tests/oauth-accounts`, which already reaches `skein-tls-world`.
7. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-oauth-accounts --all-targets -- -D
   warnings`; `heavy cargo nextest run -p skein-oauth-accounts` and that world's package; `heavy cargo check --tests -p
   jig-protocol-accounts`, its reader.
8. The citations check. Commit and push.

## Report

`notes/link.md`'s break, the common note's items, and the net lines of `exchange.rs`.
