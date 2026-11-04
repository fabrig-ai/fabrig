+++
id = "source-name-refused"
part = "source-name"
kind = "feature"
needs = []
files = ["jig/testing/jig-test-service/src/program_sources.rs", "jig/testing/jig-test-service/src/tests/program_sources.rs"]
effort = "medium"
minutes = 20
+++

# The test service answers a source name smith refuses

## Goal

`jig-test-service`'s `program_sources::Component::start` rebuilds smith's `SourceName` from
jig's with `.expect("same source name grammar")`. The grammars match only for names `new`
admits: `jig_domain_host::SourceName::from_stored` keeps stored bytes without judging their
spelling, so a stored odd name reaches `start` and aborts the test service.

Acceptance:

- A name smith's `SourceName::new` refuses answers the command `SourceFailure::Unconfigured`
  with the detail `invalid source name`, as the same function's `Error::Name` arm does, and the
  component stays settled.
- The new test below fails on the old code by that panic, and passes.
- Every test of `jig-test-service` stays green.

## Inputs

- `jig/docs/plan/bugs-jig/notes/common.md`.
- `jig/docs/design/domain/hosts.md`, 5.1 and 6.8.
- The code: `jig/testing/jig-test-service/src/program_sources.rs` (`Component::start`, `failed`);
  `SourceName` in `jig/crates/jig-domain-link/src/lib.rs` (`new`, `from_stored`) and
  `smith/crates/smith-host-domain/src/source_name.rs`.

## Owns and may change

- `jig/testing/jig-test-service/src/program_sources.rs`: the one line that builds `source` in
  `start`.
- `jig/testing/jig-test-service/src/tests/program_sources.rs`: one new test, and any imports it
  needs.
- `temper/crates/temper-service/src/program_sources.rs` has the same line: not this plan's.
  Leave it; the lead's report carries it.
- The neighbouring `.expect("same printable method grammar")` in `answer` stays.

## Steps

1. The test, `a_stored_name_smith_refuses_answers_unconfigured`, beside the existing tests:
   - `Component::new(64)`, `configure(resources())`;
   - `jig_domain_host::SourceName::from_stored(b"Upper-Case")`, which `new` would refuse;
   - `start` with `sources::Operation::Check`, `skein_io::kernel::Fd::new(9)` for the root, and
     empty `Queue`s for `above` and `below`;
   - assert `above` holds one `SourceAnswered` whose answer is `Checked(LoginState::Failed)` with
     `SourceFailure::Unconfigured` and the detail `invalid source name`, `below` is empty, and
     `settled()`;
   - also `Operation::SignOut`, its `SignedOut(SignOutResult::Failed)`.
   See it fail on the `expect`.
2. In `start`:

   ```rust
   let Some(source) = smith::SourceName::new(name.as_bytes()) else {
       above.push(failed(owner, operation, jig::SourceFailure::Unconfigured, b"invalid source name", self.detail));
       return;
   };
   ```

3. `heavy cargo nextest run -p jig-test-service --lib -E 'test(=tests::program_sources::a_stored_name_smith_refuses_answers_unconfigured)'`,
   then `heavy cargo nextest run -p jig-test-service`.
4. `cargo fmt`; old forms; `heavy cargo clippy -p jig-test-service --all-targets -- -D warnings`;
   commit and push.

## Report

As the implementer's text asks, with how the test failed on the old code.
