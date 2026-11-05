+++
id = "temper-wrong-kind"
part = "temper-wrong-kind"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/accounts.rs", "temper/crates/temper-service/src/engine.rs", "temper/crates/temper-service/src/tests/accounts_host.rs", "temper/docs/design/protocol/credentials.status.md"]
effort = "medium"
minutes = 30
+++

# temper: a restored secret of the wrong kind stops the engine

## Goal

jig's reference stops its engine on a restored secret of the wrong kind
(`a_restored_sources_wrong_kind_stops_the_engine`). In temper `Output::InvalidSecret` becomes
`Corrupt { owner: Token::new(0) }`, which finds no flight in `store::route::up`: the accounts child has aborted itself
and the engine runs on. The user has decided (the request's part 1): the engine stops. This is the one behaviour change
of this task.

Acceptance:

- `a_restored_sources_other_kind_stops_the_engine` exists, fails on your base with `engine.end` `None`, and passes after.
  Its failure on the base is in your report.
- `a_restored_sources_other_kind_fails_the_read` passes as it stands: it pops `above` itself, so `answer` never runs.
- A real `Corrupt` on an accounts write routes as it does today: the new arm compares the one token, never
  `accounts::owns`.
- `credentials.status.md`'s **Not yet** entry for 5 is gone.
- `heavy cargo nextest run -p temper-service --lib` and `-p temper-one-process-world` pass.

## Inputs

- `temper/docs/plan/debt-15/notes/common.md`
- `temper/docs/design/protocol/credentials.md`, 5: the sentence on a restored secret of the wrong kind, written for this
  task.
- `jig/docs/design/protocol/service.md`, 11.1 (`InvalidSecret`: "what that means is the owner's").
- `jig/testing/jig-test-service/src/tests/accounts.rs`, `a_restored_sources_wrong_kind_stops_the_engine`.
- For reading, on an older main: `git show origin/debt-9/temper-wrong-kind:temper/crates/temper-service/src/tests/accounts_host.rs`
  holds the test as it was written. Don't merge that branch.

## Owns and may change

- **Owns:** `temper-service/src/accounts.rs` (the one `InvalidSecret` arm, a const), `engine.rs` (one arm of `answer`),
  `tests/accounts_host.rs` (one test).
- **May change:** `temper/docs/design/protocol/credentials.status.md`: the sentence beginning "5: a restored secret of
  the wrong kind" goes, nothing else but the date. This departs from the implementer's text for that sentence alone.
- **Not to touch:** any jig file: `service.md`, 11.1 makes the meaning temper's.
- **Options:** every `secret()` caller in `jig-protocol-accounts`' `Component` (a restored, adopted, moved or held value)
  reaches the same `Output::InvalidSecret`, and the owner can't tell them apart, so each of them now stops the engine, as
  each does in jig's reference. That's the expected reading: change nothing for it, and say in the report which test shows
  it, or that none does.

## Steps

1. **Reproduce,** in `temper/crates/temper-service/src/tests/accounts_host.rs`, beside
   `a_restored_sources_other_kind_fails_the_read`, on its table of kinds:

   ```rust
   #[test]
   fn a_restored_sources_other_kind_stops_the_engine() {
       // As the read's test, for both kinds; then, instead of popping `above`:
       crate::engine::answer(engine);
       let mut expected = config::Problems::empty();
       expected.push(config::Path::root().member(b"accounts secrets"), config::What::Refers);
       assert_eq!(engine.end, Some(crate::End::Stopped(crate::Stop::Start(expected))), "{kind:?}");
   }
   ```

   Run it: `heavy cargo nextest run -p temper-service --lib -E 'test(=tests::accounts_host::a_restored_sources_other_kind_stops_the_engine)'`
   (take the path from `--list`). It should fail with `engine.end` `None`. If it doesn't, stop: report `Question:` with
   what you saw, for the request's reading is then wrong.
2. **The fix:**

   ```rust
   // accounts.rs: the owner of a secret of the wrong kind, under the accounts' prefix and above every exchange
   // the accounts' host issues.
   pub(crate) const WRONG_KIND: Token = Token::new(PREFIX | (1_u64 << 59));
   Some(Output::InvalidSecret) => store.host.above.push(store::Event::Corrupt { owner: WRONG_KIND }),

   // engine.rs, in `answer`, beside the arm for `forge::gone::owns`:
   jig_protocol_store::Event::Corrupt { owner } if owner == crate::accounts::WRONG_KIND => {
       stop::unavailable(engine, b"accounts secrets");
   }
   ```

   Bit 59 is the choice because a source's own exchange is its number, which may be 0, so `Token::new(PREFIX)` could be
   a real read's owner; the highest bit `jig-protocol-accounts/src/component.rs` uses is 39.
3. The status sentence goes. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets -- -D warnings`; the two
   packages' tests; the citations check. Commit and push.

## Report

The common note's items. The reproduction's output on your base. Break: remove the new arm and name the failure. Which
test shows a wrong kind on a non-restore read stopping the engine, or that none does.
