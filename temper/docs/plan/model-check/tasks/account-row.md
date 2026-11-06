+++
id = "account-row"
part = "account-row"
kind = "feature"
needs = ["pane-feed"]
files = [
  "temper/crates/temper-client-view/src/pages/accounts/rows.rs",
  "temper/crates/temper-client-view/src/tests/accounts.rs",
]
effort = "medium"
minutes = 50
+++

# An account's row holds the same lines

## Goal

Off the welcome, the accounts page's row of an account holds what its trial says, so a person on `ConnectModel` who adds an
account finds it there: `temper/docs/design/domain/trial.md`, 6.2.

Acceptance, tests in `src/tests/accounts.rs`, failing first:

- `a_row_holds_its_trials_line`: each state, under the head and before the sources, in a browser and in the terminal.
- `a_row_answered_for_another_account_says_nothing`: `took` is that row's alone.
- `sign_in_again_on_a_row_goes_to_the_ways_page`, not the welcome's; with no answering act there's no "All ways".
- `try_again_on_a_row_runs_another`.
- `a_row_with_no_trial_reads_as_today`: no node changes.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`, `notes/client.md`, `notes/contracts.md`, *The view*.
- `temper/docs/design/domain/trial.md`, 6.1, 6.2.
- `temper/docs/design/client/README.md`, 3.5, its accounts page.

## Owns and may change

Owns, as `files` lists them.

May also change: `temper/crates/temper-client-view/src/limits.rs`, each shell's `limits.rs`: node room per row, derived;
`temper/tools/gallery/src/samples.rs`, `names.txt`, `tests.rs`: one sample, `accounts-try-stopped`, sorted, and regenerated
on a merge.

Options, each pre-approved: **`draw`'s `expect(clippy::too_many_lines)`** may be added or kept; the row's line is one call.

## Steps

In `rows.rs`'s `draw`, after `notice` and before "Sources":

```rust
if let Some(trial) = crate::pages::model_check::of_account(root, w, row.number) {
    crate::pages::model_check::line(b, root, w, bindings, &trial, crate::pages::model_check::Place::Row);
}
```

The checks: fmt, `old_forms.py` for `temper`, clippy on `temper-client-view`, `temper-client-gallery`,
`heavy cargo nextest run -p temper-client-view --lib`, `-p temper-client-gallery`. Citations. Push.

## Report

As the implementer's text asks: each test's bite; whether you took `notes/client.md`'s merge.
