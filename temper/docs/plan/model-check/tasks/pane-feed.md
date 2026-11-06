+++
id = "pane-feed"
part = "pane-feed"
kind = "feature"
needs = ["trial-facts"]
files = [
  "temper/crates/temper-client-view/src/pages/model_check.rs",
  "temper/crates/temper-client-view/src/tests/model_check.rs",
]
effort = "high"
minutes = 110
+++

# Model's line reads the trial

## Goal

`model_check::trial`, the one function `look-round-12` left for it, is fed: a row's steps, its answer, its stop and why, and
"Try again" runs another. The page's heading and act don't read it: only the host's check keeps a person out.

Acceptance, tests in `src/tests/model_check.rs`, failing first, browser and terminal:

- **Each row of `temper/docs/design/domain/trial.md`, 6.1,** in its order. In particular a subscription's second and third
  step, and a plan's and a key's two.
- **From the ask to its row** the line is Running, never ticked: `trial_asked` with no row.
- **Each `Why` of 6.4** says its words; `Permanent` offers "Sign in again"; `Waited` says the last wait's words, then with
  none "WHO didn’t answer within a minute."
- **A wait that's no moment's says so at once,** on a row still running: `NeedsSignIn`, `Limited`, `Refused`,
  `Misconfigured`, `Unloaded`, `Host`, `Empty`, `Credential`, `TaskRunWait::Host`, `Refused(Hello)`, each in the words a chat's
  wait has. A wait that settles (`Asking`, `Refreshing`, `Granted`, `Moving`) leaves it Running. No command is printed.
- **`took`:** where a row took another account, ACCOUNT and WHO are that account's.
- **Only a failed trial:** with the account usable and a `Failed` row, the page stays "Ready" and its act works.
  With the host's check settled not usable, "Almost ready" and disabled.
- **"Try again"** on a `Failed` row sends `Action::Forge(forge::Act::TryAccount { account, again: true })`; on a host's stop
  it checks the host, as today. Running offers neither.
- `dead_code` goes: `First::{Plan, Key}` and `State::Answered` are reached outside tests, and the two `expect`s are gone.
- The gallery's `welcome-ready-*` samples still hold, with a trial's row where the sample needs one.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`, `notes/client.md`, `notes/contracts.md`, *The client's facts*, *The view*.
- `temper/docs/design/domain/trial.md`, 6.1, 6.4.
- `temper/docs/design/client/README.md`, 3.5, *The try*, *Ready*.
- `temper/crates/temper-client-view/src/choice.rs`, its `waiting` and `account_wait`: the wait's words.

## Owns and may change

Owns, as `files` lists them.

May also change, each by a named hunk:

- `temper/crates/temper-client-view/src/choice.rs`: `wait_why`, `wait_settles`, no change to what `waiting` draws.
- `temper/crates/temper-client-view/src/accounts.rs`: `Meaning::Try` and its act.
- `temper/crates/temper-client-view/src/words.rs`: the words of 6.4; `limits.rs` and each shell's `limits.rs`: bounds derived
  from the word list.
- `temper/crates/temper-client-view/src/pages/welcome.rs`: the call's new argument.
- `temper/tools/gallery/src/samples.rs`, `tests.rs`, and `temper/crates/temper-client-view/src/tests/{welcome,fixtures}.rs`:
  expectations that read the line.

**The test `only_the_welcomes_host_check_supplies_a_trial_today` is rewritten to the new rule.** `temper-client-domain`'s
`pages/welcome.rs` `model_counts` doesn't change.

Options, each pre-approved:

- **ACCOUNT's words for an account by number:** `first_usable_kind` reads the first usable one; draw a helper from it in
  `pages/accounts.rs`, taking a line, and have it call the helper, one hunk.
- **A word's bound past `limits.rs`' derivation:** the derivation follows; a bound elsewhere is left, and reported.

## Steps

The note's surface. `line`'s link for "Sign in again" follows `Place`: `Place::Row` is `pane-feed`'s to take as an argument,
`account-row`'s to use. `Place::Row`, here, goes to `Address::ConnectModel` at the way's page as the accounts page's own link
does.

In `trial`, `First` comes from the account's first usable source's `kind`: `Program` is `Subscription`, `SignIn` is `Plan`,
`Key` is `Key`. The way for "Sign in again" comes from the same source.

The checks: fmt, `old_forms.py` for `temper`, clippy on `temper-client-view`, `temper-client-gallery`,
`heavy cargo nextest run -p temper-client-view --lib`, `-p temper-client-gallery`, and `-p temper-client-shell-terminal --lib`,
whose prints read the line. Citations. Push.

## Report

As the implementer's text asks: each test's bite; whether you took `notes/client.md`'s merge.
