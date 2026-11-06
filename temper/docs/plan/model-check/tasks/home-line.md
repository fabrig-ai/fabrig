+++
id = "home-line"
part = "home-line"
kind = "feature"
needs = ["pane-feed"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
]
effort = "medium"
minutes = 55
+++

# Home says a try that stopped

## Goal

A person who has already left when a trial fails finds it on Home: `temper/docs/design/domain/trial.md`, 6.3. "Try again" runs
another from there.

Acceptance, tests in `src/tests/home.rs`, failing first, browser and terminal:

- `home_says_a_try_that_stopped`: with a `Failed` row, whatever `to_begin` is, "WHO didn’t answer when tried" and "Try again"
  stand under the intro, in that order.
- `answered_says_nothing_more` and `running_says_nothing`.
- `try_again_runs_another_and_the_line_goes`: its act is `Action::Forge(forge::Act::TryAccount { account, again: true })`;
  with the ask out or the row running, no line.
- `a_home_with_no_trial_reads_as_today`: no node changes.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`, `notes/client.md`, `notes/contracts.md`, *The view*.
- `temper/docs/design/domain/trial.md`, 6.3.
- `temper/docs/design/client/README.md`, 3.5, *Home*.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/forge/bindings.rs`: a `Meaning` for the act, as its neighbours; its arm.
- `temper/crates/temper-client-view/src/words.rs`: `HomeTryStopped`, "{name} didn’t answer when tried"; `limits.rs` and each
  shell's: derived bounds.
- `temper/tools/gallery/src/samples.rs`, `names.txt`, `tests.rs`: one sample, `home-try-stopped`, sorted, regenerated on a merge.
- `temper/crates/temper-client-shell-terminal/src/tests/**`: a print that reads Home, its expectation only.

Options, each pre-approved:

- **`frame::Bindings` can't carry the act:** take `view.accounts` in `home::main` as the welcome does, and change its three
  callers (`root.rs`, `pages/configure.rs`, `pages/new_chat.rs`) by the one argument.
- **`HOME_BYLINE` reads wrong beside "Try again"** at a phone's width: the words are the line and the button its own, under it. No
  new CSS rule: the look's are the look request's.

## Steps

In `intro` (`pages/home.rs`), after the byline and before its `b.close()`, and in its terminal branch likewise:

```rust
if let Some((who, account)) = super::model_check::stopped(root, w) {
    text(b, Role::Paragraph, &w.say(Word::HomeTryStopped, &[Hole::Name(who)]), Some(look::HOME_BYLINE));
    // "Try again", the button its neighbours use, `WelcomeTryAgain`'s word.
}
```

The checks: fmt, `old_forms.py` for `temper`, clippy on `temper-client-view`, `temper-client-gallery`,
`heavy cargo nextest run -p temper-client-view --lib`, `-p temper-client-gallery`, `-p temper-client-shell-terminal --lib`. Citations. Push.

## Report

As the implementer's text asks: each test's bite; whether you took `notes/client.md`'s merge; which option.

## Rebase (2026-11-06, step merge, foreman model-check/home-line/foreman-1)

reconcile HomeTryStopped with the landing’s Copy vocabulary and combined word bounds
Rebase `3d7107d4c9` onto the updated landing target.
The merge conflicts in `temper/crates/temper-client-view/src/words.rs`: both sides change vocabulary totals and `ALL`’s length.
Upstream merge `98c62ae899` brings in main’s Copy vocabulary, including merge `b1230cbb3e`.
Keep `WelcomeCopy`, `WelcomeCopied`, and `WelcomeNotCopied`, alongside this task’s `HomeTryStopped` enum variant, `ALL` entry, ID `3876`, and English phrase.
Preserve the trial-ending vocabulary retained by the integration branch.
Against integration tip `6c11e278cd`, the combined values are `PHRASES = 1869`, `SEGMENTS = 3272`, `ENGLISH_BYTES = 45678`, and `ALL: [Word; 1269]`; recount if the target advances.
Keep both independently added node allowances in the automatically merged `limits.rs`: account trial nodes and Home’s stopped-trial line.
Review the automatic merges in `root.rs` and `tests/home.rs` for preservation of both sides.
Run the spec’s checks through the required tools, including the view, gallery, and terminal tests; regenerate any combined-code expectations that become stale.
