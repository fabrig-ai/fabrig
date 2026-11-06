+++
id = "trial-start"
part = "trial-start"
kind = "feature"
needs = ["trial-root", "trial-facts", "pane-feed"]
files = [
  "temper/crates/temper-client-domain/src/forge/trials.rs",
  "temper/crates/temper-client-domain/src/tests/trials.rs",
  "temper/tests/client/tests/suite/model_check.rs",
]
effort = "high"
minutes = 120
+++

# An account becoming usable starts its trial

## Goal

The meeting point: the client asks, the real engine runs it, the pane says so. An account newly usable on the welcome or
`ConnectModel` starts a trial, and every world that walks there now runs one. This task carries that fallout.

Acceptance:

- `temper-client-domain`'s tests, failing first:
  - `an_account_newly_usable_on_the_welcome_is_tried`: on `Welcome { Model { .. } | Ready }` and `ConnectModel`, a delivery that
    shows an account usable that the last didn't sends one `try_account { again: false }`, and `trial_asked` holds it.
  - `the_first_delivery_tries_nothing`: a reload, or a session opened on a usable account, spends nothing.
  - `elsewhere_tries_nothing`: on any other page.
  - `the_playground_is_never_tried`: its provider's account, whatever the page.
  - `an_account_with_a_row_is_not_asked_again`.
- `temper/tests/client/tests/suite/model_check.rs`, the client and the real engine joined, its worker scripted: `the_success_story`,
  from "I’ve signed in" through Running at each step to Answered, the page on Ready from the press on.
- The worlds and walks that reach a usable account hold, each fixed in place: no cue of theirs is spent on a trial.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`, `notes/client.md`, `notes/contracts.md`, *The client's facts*.
- `temper/docs/design/domain/trial.md`, 1, 6.
- `temper/docs/design/domain/testing.md`, 2.1, 4.1, 6.3.
- `temper/tests/client/src/front.rs`: `hosted_attempt`, `reply_in_pieces`, `release_reply`, `finish_reply`;
  `front/account_acts.rs`: `program_checked`.

## Owns and may change

Owns, as `files` lists them.

May also change, naming each:

- `temper/crates/temper-client-domain/src/route.rs`: `told`'s one line, beside `pages::home::look`.
- `temper/tests/client/src/**`: a helper that settles a trial's attempt, `answer_trial(front, root)`, and fixtures' options;
  `temper/tests/client/tests/suite/{connect_model,connect_model_edges,forges_edges,sign_in,sign_in_paste,welcome_m3,welcome_m4,reloads,main}.rs`,
  and any other of that suite where a trial's attempt appears: expectations and the helper's call only.
- `temper/crates/temper/tests/support/{person,sign_in_welcome}.rs`, `temper/crates/temper/tests/fake_backends/walk/**`,
  `temper/crates/temper-client-shell-web/tests/suite/host/**`, `temper/tests/client-browser/**`, `temper/tests/real-loop/src/case/**`:
  an expectation that a trial's attempt or line moves, and `sign_in_welcome.rs`'s comment, which says no model call. Its
  bound of 20 s is the user's: if a walk passes it, that's a measurement for your report, and a question.
- `temper/crates/temper-fake/src/**`: only if a walk needs `otherwise` set where it isn't.

Options, each pre-approved:

- **A world's scripted worker takes the trial's attempt for its own:** answer the trial first in that world's setup, by the
  helper. If a world can't, fix its fixture to start past the welcome.
- **A fake model with no `otherwise`:** the trial fails and nothing stops; the world holds with its line Stopped, if the
  line isn't what it tests.
- **A world that counts tasks** by the deployment's attempts: count charter 1's.

**This changes what a shared path does: every walk through the welcome now spends a fake turn.** Run the tests of every package
above, and `temper`'s `--test suite` and `--test fake_backends`.

## Steps

```rust
// forge/trials.rs
/// Told after the pages (domain/trial.md, 6). Reads the accounts document; the page holds no state of it.
pub(crate) fn look(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>);
```

The `Trials` state gains `seen: Option<List<u32>>`: none until a first delivery, then the accounts with a usable source.
Each `Told` on the root, `look` reads `pages::accounts::document`, and sends for each account in the new set and not in `seen`
whose provider isn't `playground_provider()`, with no row and none asked, while the address is one of the three. `seen`
moves on, whatever the page.

The checks: fmt, `old_forms.py` for `temper`, clippy on what you changed, the tests above. Citations. Push.

## Report

As the implementer's text asks: each test's bite; every world you touched and why; each walk's time before and after.
