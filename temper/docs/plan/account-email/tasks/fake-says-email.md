+++
id = "fake-says-email"
part = "look"
kind = "feature"
needs = ["keep-facts", "program-report", "names-by-email"]
files = ["temper/crates/temper-fake/src/oauth.rs", "temper/crates/temper-fake/src/oauth/tests.rs", "temper/tests/real-loop/tests/suite/sign_in.rs", "temper/tests/real-loop/tests/suite/model_check.rs", "temper/docs/design/client/journeys/**"]
effort = "high"
minutes = 80
+++

# The fake provider says an email, and the whole way holds

## Goal

The meeting point: a sign-in's email and a program's, through the real listener, the real engine and the real client, to the words a person reads. `temper fake`'s issuer says one too, so that a fake box's Ready and the journeys recorded on it read as a real box does.

Acceptance:

- `temper-fake/src/oauth.rs`: the issuer signs in as `fake-person` with a verified `sam@example.com` and the name `Sam`, through `sign_in_as`. `oauth/tests.rs` holds it.
- `temper/tests/real-loop`, `sign_in.rs`, in `a_terminal_sign_in_returns_to_the_real_listener`: its issuer signs in with a verified `owner@example.com`; the story sees `Signed in to ChatGPT as owner@example.com` where it saw `Signed in to ChatGPT`; and `account["email"]` is `owner@example.com`. No new test.
- `temper/tests/real-loop`, `model_check.rs`, in `a_trial_runs_through_the_worker_and_the_fake_model`: the fake program's `you@example.com` shows, by one more `see` in a step it already has. No new test.
- Run `heavy cargo run -p temper-client-journeys`. For each journey it names stale, run `heavy cargo run -p temper-client-journeys -- again NAME`, as `common.md` says, then the focused replay checks. Read each print's diff: the email on Ready's and the Model step's lines, with times and ports, is what you'd expect. Anything else is a red.
- A browser story whose words this changes is compiled, not run: `heavy cargo check -p temper-client-browser-world --tests`, by whichever name its manifest has. Say so in the report; after-landing runs it.
- Each commit carries `Look-request: ready-account-email`, directly above the other trailers (`temper/docs/design/draft/look-requests.md`, 3).
- The focused and fuzzy tests of `temper-fake`, `temper-real-loop-world` and `temper-client-journeys` pass. These add no process and no wait, so `temper-real-loop-world` stays within its budget; if it doesn't, its figures go in the report, and the repair follows `AGENTS.md`, 5.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Ready*'s model line.
- `temper/docs/design/client/journeys.md`, 8.1.
- `jig/docs/design/domain/accounts.md`, 3.7.
- `temper/docs/plan/account-email/notes/common.md`.

## Owns and may change

Owns the header's files. Readers: a test that pins a fake box's Ready or Model step without an email takes the email, in any test the focused runs name, each edited only for that: on the trunk `temper/crates/temper/tests/**` and `temper/tests/client-browser/**` are the likely ones. `temper/tests/client-browser/src/issuer.rs` signs in as it does today, unless its story needs the email.

Options, each pre-approved:

- `temper/tests/real-loop/src/issuer.rs`'s `OpenId` already takes an `Account` through `sign_in_as`: use it where the story builds its peer, in `sign_in.rs`.
- If a real-loop story trips on the sign-in's words past the one above, the email in the words is correct: repair that assertion.
- `temper/crates/temper/tests/fake_backends/walk/program_here.rs` is in the process tier: compile it, and don't run it. Its words change if it names the program's line, as `after-landing` will say.

## Steps

```rust
// temper-fake/src/oauth.rs, after Peer::new
peer.sign_in_as(issuer::Account {
    subject: b"fake-person".as_slice().into(),
    email: Some(issuer::Email { address: b"sam@example.com".as_slice().into(), verified: true }),
    name: Some(b"Sam".as_slice().into()),
})
```

Each change breaks its rule once by hand. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them, one command at a time. Commit and push.

## Report

As the implementer's text says, with the journeys recorded again and what each print's diff held. No walk, and no browser story run.
