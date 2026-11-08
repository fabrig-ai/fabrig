+++
id = "issuer-log"
part = "issuer"
kind = "feature"
needs = []
files = ["temper/crates/temper/tests/suite/fake.rs"]
effort = "medium"
minutes = 50
+++

# `temper fake oauth` logs each connection it turns away at its cap

## Goal

Half of what the process walk
`sign_in::a_chatgpt_sign_in_beside_a_browsers_idle_issuer_sockets_leaves_ready_able_to_create`
proves is that the fake issuer, past `temper_fake::oauth::CONNECTIONS`,
turns a connection away **and logs it**: the line `temper fake oauth:
rejected a connection at the cap of 8`, which `temper::fake`'s host
writes (`rejections`, `temper/crates/temper/src/fake.rs`, about 694 and
its call about 890). That line is the `temper fake oauth` process's own,
so its story goes beside the fake's other process stories in this
package's focused `suite` binary, `temper/crates/temper/tests/suite/fake.rs`,
which already starts `temper fake <kind> --document` and reads it. The
other half, held sockets not blocking a sign-in, is `issuer-sockets`'.
The walk itself is `walks-go`'s; you never touch it.

Acceptance:

- **`the_oauth_fake_logs_each_connection_it_turns_away_at_its_cap`**,
  new, in `suite/fake.rs`: start `temper fake oauth` on a document like
  `fakes_claim_their_inherited_listeners_before_the_first_drive`'s oauth
  one (a listen socket on `127.0.0.1:0`), read its ready line; open
  `temper_fake::oauth::CONNECTIONS` idle sockets to it and hold them,
  then one more; read the process's stderr, within a bounded patience
  (`support::patience` or `skein_world::Deadline`, never a bare sleep),
  until it holds exactly the line `temper fake oauth: rejected a
  connection at the cap of 8` (the cap read from `CONNECTIONS`, not
  written as 8); drop the sockets, stop the process with TERM and see it
  exit 0. `Running::stop` asserts an empty stderr: give this story its
  own stop that reads stderr, or a `Running` method beside `stop`.
- **Red first:** remove `self.rejections();` from the fake host's pass
  in `temper/crates/temper/src/fake.rs` by hand, run the story alone, see
  it fail for the missing line (its patience expiring with the stderr
  shown), restore, see it pass. The report quotes the failure.
- Clippy clean, fmt clean, the `suite` binary's `fake::` stories green.

## Inputs

- `temper/docs/plan/walks-lower/notes/common.md` (read first). This
  story is a focused one in the `temper` package's `suite` binary, not
  the `process` binary: run it.
- `temper/docs/plan/walks-lower/inputs/request.md`, the idle-sockets
  ruling.
- `temper/docs/design/domain/testing.md`, 2.6, the bullet **The fake box
  keeps two walks**, its **The fake issuer's held sockets** item.
- The fix it guards: commit `8cba1c07a3` ("Reserve issuer connections
  for browser sign-in and report cap rejections"), its
  `temper/crates/temper/src/fake.rs` half.

## Owns and may change

Owns:

- `temper/crates/temper/tests/suite/fake.rs`: the new story and any
  private helper it needs.

Pre-approved options:

- **The idle sockets:** `std::net::TcpStream::connect` is fine here: the
  fake is another process. If the issuer serves a socket that sends
  nothing for longer than the story holds it, so that fewer than
  `CONNECTIONS` are held at the extra one's arrival, open the extra ones
  in a loop until the line appears, bounded by `CONNECTIONS` more, and
  say so.

## Steps

1. Read `suite/fake.rs`'s `Running` and the oauth table row, and the
   fake host's `rejections`.
2. Write the story; run it alone: `heavy cargo nextest run -p temper
   --test suite -E
   'test(=fake::the_oauth_fake_logs_each_connection_it_turns_away_at_its_cap)'`.
3. The red: remove the call, run it alone, see it fail, restore, green.
4. Once, at the end: `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper
   --all-targets -- -D warnings`; `heavy cargo nextest run -p temper
   --test suite -E 'test(/^fake::/)'` (check the count). Commit
   (`walks-lower: the oauth fake's logged turn-away beside its process
   stories`, the body saying why: the line is the fake process's, so its
   story is the fake's, not a box walk's) and push.

## Report

As the implementer's text says: the first line; the commit pushed; each
run with its count; the red and its failure line; the story's wall and
CPU seconds; the minutes.
