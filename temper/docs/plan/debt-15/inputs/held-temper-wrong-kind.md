+++
id = "temper-wrong-kind"
part = "temper-wrong-kind"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/accounts.rs", "temper/crates/temper-service/src/engine.rs", "temper/crates/temper-service/src/tests/accounts_host.rs", "temper/docs/design/protocol/credentials.md"]
effort = "medium"
minutes = 40
+++

# temper: a restored secret of the wrong kind

## Goal

The request's part 2, first follow-up. jig stops its engine on a restored secret of the wrong kind
(`a_restored_sources_wrong_kind_stops_the_engine`). In temper, by reading, `Output::InvalidSecret` becomes
`Corrupt { owner: Token::new(0) }`, which finds no flight in `store::route::up`; the accounts child has aborted
itself and the engine runs on.

The design does not say which holds: `jig/docs/design/protocol/service.md`, 11.1 leaves it to the owner ("what
that means is the owner's"), and temper's says nothing of this case. So the request's own rule applies, and
**this spec changes behaviour only on the user's answer**, the question `wrong-kind-meaning` in
`temper/docs/plan/debt-9/plan.toml`.

Acceptance:

- The reproduction below exists, and its result is in your report.
- With an answer, the code does what the answer says, and the test fails on your base and passes after.
- With none, no production file changed, and your report's first line is `Question:`.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11.1; `jig/docs/design/protocol/store.md`, 7.1 and 9;
  `temper/docs/design/protocol/credentials.md`, 5; `skein/docs/design/oauth.md`, 6.4.
- `jig/testing/jig-test-service/src/tests/accounts.rs`, `a_restored_sources_wrong_kind_stops_the_engine`.

## Owns and may change

- Owns: `temper-service/src/accounts.rs` (the one `InvalidSecret` arm and a const), `engine.rs` (one arm of
  `answer`), `tests/accounts_host.rs` (one test).
- `temper/docs/design/protocol/credentials.md`, 5: on an answer that stops the engine, one sentence after
  "A failure of jig's secrets' store itself stops the engine, ...": "A restored secret of the wrong kind stops
  it too (jig's `protocol/service.md`, 11.1): the store holds what the account's row says it should not."
  This departs from the implementer's text for that sentence alone.
- `a_restored_sources_other_kind_fails_the_read` stays as it is: it pops `above` itself, so `answer` never
  runs.
- No jig file changes: `service.md`, 11.1 makes the meaning temper's.

## Steps

1. **Reproduce,** in `temper/crates/temper-service/src/tests/accounts_host.rs`, the lowest world that composes
   temper's host. Beside `a_restored_sources_other_kind_fails_the_read`, on its table of kinds, and after its
   passes:

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
   (take the path from `--list`). It should fail with `engine.end` `None`. If it doesn't, stop: report `Question:`
   with what you saw, for the request's reading is then wrong.
2. **Read the question** `wrong-kind-meaning` in `temper/docs/plan/debt-9/plan.toml` as it stands in your slot.
   With no `answer`: commit the test, change nothing else, push, and report `Question:` with the plan's question
   and the failure you saw. Your next run starts from that branch. With an answer that says the engine doesn't
   stop: do what it says, turning the test to pin that.
3. **On an answer that says the engine stops:**

   ```rust
   // accounts.rs: the token of a secret of the wrong kind, under the accounts' prefix.
   pub(crate) const WRONG_KIND: Token = Token::new(PREFIX);
   Some(Output::InvalidSecret) => store.host.above.push(store::Event::Corrupt { owner: WRONG_KIND }),

   // engine.rs, in `answer`, beside the arm for `forge::gone::owns`:
   jig_protocol_store::Event::Corrupt { owner } if owner == crate::accounts::WRONG_KIND => {
       stop::unavailable(engine, b"accounts secrets");
   }
   ```

   Compare with the one token, never `accounts::owns`: a real `Corrupt` on an accounts write is another
   question, left as it is. Check that `Token::new(PREFIX)` is an exchange the accounts' store never issues
   (exchanges start at 1, or say so and choose another); if `Token::new` isn't `const`, use a `fn`.
4. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-service --lib`, and the worlds that read this event's route:
   `heavy cargo nextest run -p temper-one-process-world` (take the name from `temper/tests/one-process/Cargo.toml`).
5. Commit and push.

## Report

The common note's items. The reproduction's output on your base. Break: remove the new arm and name the
failure. Say whether any `secret()` caller other than restore now stops the engine, and what test shows it:
`jig-protocol-accounts`' `Component::secret` is also reached from adopted, moved and held values.
