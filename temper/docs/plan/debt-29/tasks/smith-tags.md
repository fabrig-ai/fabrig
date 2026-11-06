+++
id = "smith-tags"
part = "smith-tags"
kind = "feature"
needs = []
files = ["smith/crates/smith-host-protocol/src/process.rs", "smith/crates/smith-local-service/src/host_io.rs", "smith/crates/smith-local-service/src/host_io/delivery.rs", "smith/crates/smith-local-service/src/service/program_check.rs", "smith/crates/smith-local-service/src/service/program_check/binding.rs", "smith/crates/smith-local-service/src/service/program_check/tests/fixture.rs", "smith/docs/design/protocol/hosts.status.md"]
effort = "medium"
minutes = 30
+++

# smith's token tags all go through `Token::tagged`

## Goal

Finding 3 of `temper/docs/plan/debt-29/inputs/debt27-review.md`. smith's D020 closed with hand-set tags left. Four production sites set a token's top bits by hand, where `Token::tagged` holds the namespace rule (`skein/docs/design/lib.md`, 4). Each converts, and **every token keeps its value**.

| Site | Today | Becomes |
|---|---|---|
| `smith-host-protocol/src/process.rs`, `Process::new` | `assert!(agent.raw() < (1 << 63))`, then `Token::new(agent.raw() \| (1 << 63))` | `Token::tagged(IO_TAG, agent.raw()).expect("domain owner fits the process-owner namespace")`, `IO_TAG` 8; the assert goes |
| `smith-local-service/src/host_io/delivery.rs`, the `GitChild`'s io owner | `Token::new(1 << 62)` | `Token::tagged(GIT_TAG, 0)`, `GIT_TAG` 4 |
| `service/program_check/binding.rs` and `program_check.rs`, `next_owner` | a counter from `1 << 60`, and `Token::new(self.next_owner)` | a counter from 0, and `Token::tagged(PROGRAM_TAG, self.next_owner)`, `PROGRAM_TAG` 1 |
| `smith-local-service/src/host_io.rs`, `PROGRAM_ROOT` | `const PROGRAM_ROOT: Token = Token::new(1 << 60)` | a function or a field holding `Token::tagged` of tag 1, value 0 |

The cut checked the review's doubt: `process.rs` needs its owners below 2^60. Its shipping caller takes `host::process::Request::Spawn`'s `owner`, which `smith-host-domain/src/domain.rs` sets to `id.token()`, and `Id::token` is `slot << 32 | generation`. So an owner reaches 2^60 only from slot 2^28 up, past any agent limit; the tests pass 7 and 8.

`PROGRAM_ROOT`'s departure in `smith/docs/design/protocol/hosts.status.md` was the `debt-27` cut's own, and the request here asks to pay the tags, so it converts and that entry goes.

Acceptance:

- `rg -n '<< 6[0-3]' smith/crates -g '*.rs'` finds one line: `smith-local-protocol/src/git_child.rs`'s test `IO_OWNER`. That's a test's own number: exempted, not changed. The worlds' own numbers under `smith/tests/` stay as well.
- `hosts.status.md`'s **Departs** entry on `PROGRAM_ROOT` is gone. Its file keeps its **Not yet**.
- No test changes what it asserts. `program_check/tests/fixture.rs` only follows `next_owner`.
- The net is at most +10 lines.

## Inputs

- `temper/docs/plan/debt-29/inputs/debt27-review.md`, finding 3.
- `skein/docs/design/lib.md`, 4, and `skein/crates/skein-lib/src/token.rs`.
- `smith/docs/design/protocol/hosts.status.md`.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the files in the header. `skein-lib` isn't touched: `Token::tagged` isn't a `const fn`, so `PROGRAM_ROOT` becomes `fn program_root() -> Token` beside where it stands, or a field set once in `HostIo`'s construction; the choice is yours.

- `service.rs`'s `ACCOUNT_TAG` and `EFFECT_TAG` already use `Token::tagged` on kernel operations: they stay. `namespace_token`'s assert stays.
- **If a test reads `next_owner` or a raw owner value** beyond `fixture.rs`, tell it by the token, not by a rewritten number. Its assertions keep their values.
- **`smith/docs/design/**` is read by tests** (`.config/gate/read-by-tests.txt`). The cut found no test reading `hosts.status.md`: `rg -n 'hosts\.status' smith -g '*.rs'` shows none; check again, and run what you find.

## Steps

Make each change in the table. Name the tags beside the site that uses them, as `credentials.rs`'s `ROOT_TAG` and `lock.rs`'s `LOCK_TAG` do. `HostIo`'s comment on `PROGRAM_ROOT` stays true.

No new test: every value stays, and `smith-host-protocol`'s and `smith-local-service`'s tests already compare what these name. Check the values hold: the `io_owner` of `Token::new(7)` is `0x8000_0000_0000_0007` as before, by reading it, no new assertion.

`heavy cargo nextest run -p smith-host-protocol -p smith-local-service -p smith-hosts-world`. `cargo fmt`; `(cd smith && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p smith-host-protocol -p smith-local-service --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`. Commit, push.

## Report

As your text says, with the search's reading, and the net lines.
