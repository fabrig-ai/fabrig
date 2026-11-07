+++
id = "trial-account"
part = "trial"
kind = "feature"
needs = ["account-choice"]
files = ["temper/crates/temper-engine-domain/src/route/trials.rs", "temper/tests/engine/domain/tests/suite/trials.rs", "temper/tests/client/tests/suite/model_check.rs", "temper/docs/design/domain/trial.status.md"]
effort = "high"
minutes = 150
+++

# The account just connected is the one tried

## Goal

A trial's `Make` names its account where the person has another usable one, so that a later account's trial runs on that account, whichever provider comes first and for a second account of one provider; the first usable account's trial still names no `llm`. Its row's `took` is its own, its own row on the accounts page holds the line, and "Try again" retries that account.

Acceptance:

- Section 4 of `temper/docs/plan/ui-trial-account/notes/contracts.md`, exactly, in `route/trials.rs`.
- `temper/tests/engine/domain/tests/suite/trials.rs`: `took_is_the_account_the_run_keeps` becomes the table test `a_later_account_is_the_one_tried`, its rows (the first account's provider, the later one's, the model the later trial runs):
  - `(1, 2, second)`: Claude, then ChatGPT;
  - `(2, 1, scripted)`: ChatGPT, then Claude;
  - `(1, 1, scripted)`: a second account of one provider.

  For each: the first account's trial runs with `task.llm` `None` and `took` the first; once it ends, the later account's trial runs that model, with `task.llm`'s `account` `Some(later)`, `task.kept_account` `Some(later)` and its row's `took` `Some(later)`; once that one ends, `again: true` on the later account runs on it again. In one row, `again: true` on the first account, once the later is usable, names the first.
- `a_try_runs_one_task_of_charter_ten_naming_no_llm` passes unchanged on the widened `config()`: the first usable account's trial names no `llm`.
- `removing_the_account_cancels_it`: its `took` row now has the later account's own trial, and removing the later account takes its row and cancels its run while the first's ended row stays. The rule for rows that `took` another account stays in `temper-engine-domain-trials`' unit tests (`tests.rs`, the `AccountGone` cases), which this task doesn't touch.
- `temper/tests/client/tests/suite/model_check.rs`: `no_model_is_named` becomes `a_later_account_is_the_one_tried`, through the joined client and engine. The key of `h::PROVIDER` first, then the program sign-in of `h::PROVIDER + 1`, as it is today; then:
  - the later trial's row has `took` `Some(later)`, its assignment runs `second-scripted` on `later`, and its Task document's `llm` names `account: Some(later)`, where today they name the first;
  - the later trial fails (`fail`); the line drawn for the later account says it stopped and offers "Try again", and the first account's line still says it answered;
  - "Try again" there sends `Try { account: later, again: true }`, and the next assignment runs on `later`.
- The focused and fuzzy tests of `temper-engine-domain`, `temper-engine-domain-world`, `temper-client-world`, `temper-real-loop-world` and `temper-playground-world` pass, and `heavy cargo check --tests` for `temper-service` and `temper`.
- The journeys: if the focused replay checks of `temper-client-journeys` fail, run `heavy cargo run -p temper-client-journeys` and, for each journey it names stale, `heavy cargo run -p temper-client-journeys -- again NAME`, as `common.md` says. Read each print's diff: only a later account's trial may move (its account, model and line), beyond times, ports and generated ids; anything else is a red.
- `temper/docs/design/domain/trial.status.md`: its **Not yet** entry on 2 and 4 goes, both its paragraphs.

## Inputs

- `temper/docs/design/domain/trial.md`, 2, 3.1, 4, 6.1 and 6.2.
- `jig/docs/design/domain/accounts.md`, 4.3, *Holding a new task to one account*.
- `temper/docs/plan/ui-trial-account/notes/contracts.md`, 4.
- `temper/docs/plan/ui-trial-account/notes/common.md`.

## Owns and may change

Owns the header's files. Beyond them, each only as named:

- **Stories a later account's trial trips** in `temper/tests/client/tests/suite/*.rs` (`connect_model.rs`, `accounts.rs`, `second_sign_in.rs`, `accounts_page.rs` and any other): an assertion that a later account's trial ran on, took, or drew its line on an earlier account changes to the account itself, and a helper that answers or fails "the trial" takes the later one; nothing else in them changes. Name each in the report.
- `temper/tests/client/src/front/account_acts.rs`: only if a helper must name an account or a provider it hard-codes; `model-catalogue-3` changes `temper/tests/client/src/**`, so keep to those lines and say so.
- `temper/tests/playground/tests/fixtures/story-a.log`: if its replay goes stale, regenerate it with `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-playground-world --test suite -E 'test(=walk::story_a_walks_from_the_welcome_to_the_csv_change_landed)'`, then run `walk::story_a_kept_log_replays_natively` alone.
- `temper/docs/design/client/journeys/*/recording/**`: regenerated as the acceptance says, never by hand.

Options, each pre-approved:

- **The engine world's second provider.** `trials.rs`'s `config()` gains a provider 2, offering keys as provider 1 does, and a model `second` of provider 2 on endpoint 1, in `settings.models` and as charter 10's alternative; charter 1 stays as it is. If the in-process worker must declare `second` to place its run and `hello` doesn't, declare it in this file's setup; if that needs `temper/tests/engine/domain/src/**`, change only the line that names the models and say so.
- **A first account of provider 2.** `configured()` adds the first key as provider 1: give it the provider as a parameter (`driver()` keeps 1), and give `add_account` one likewise, keeping its callers.
- **An account serving no charter model.** If this world can add a later key whose `uses` name project 1 alone in one change, add a row that its trial is `Failed { why: Refused }` and makes no task; otherwise leave it to `account-choice`'s `None` row and say so.
- **Where "Try again" is drawn.** On the way's page after the sign-in, or on the accounts page's row of the later account (`trial.md`, 6.2), whichever this story's page draws; read the words through the drawn view, as the file's `stopped` does.
- **`choice` returns three cases.** A private enum in `route/trials.rs` (first, held with its choice, unserved) or a nested `Option`, as reads best.

## Steps

```rust
// route/trials.rs, at trials::Request::Make { account, person }, after the seed
/// The trial's choice (temper/docs/design/domain/trial.md, 2): none for the person's first usable
/// account; for a later one, `Core::account_llm` for it, or refused where it serves no model.
fn choice(core: &core::Core, seed: &core::PersonSeed, person: u64, account: u32) -> Choice;
```

```rust
// temper/tests/engine/domain/tests/suite/trials.rs
#[test]
fn a_later_account_is_the_one_tried() {
    // The first account's provider, the later one's, and the model the later trial runs.
    for (first, later, model) in [
        (1, 2, b"second".as_slice()),  // Claude, then ChatGPT
        (2, 1, b"scripted".as_slice()), // ChatGPT, then Claude
        (1, 1, b"scripted".as_slice()), // a second account of one provider
    ] {
        // first trial: llm None, took first; end it
        // add the later account; its trial: model, llm.account, kept_account, took; end it
        // again: true on the later account runs on it
    }
}
```

The client story keeps its fixture, `h::trial_selection_fixture`, and its order; its new steps are the failure and "Try again".

Each new or changed test breaks its rule once by hand: make the first trial's case name its account too (`a_try_runs_one_task_of_charter_ten_naming_no_llm` fails), and make every trial name none (the later rows fail on `took`). `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-world -p temper-client-world --all-targets -- -D warnings`; the focused and fuzzy runs and checks the acceptance names, one command at a time. Commit and push.

## Report

As the implementer's text says, with each test's wall time before and after, each story outside the header's files you repaired and why, and the journeys recorded again with what each print's diff held.
