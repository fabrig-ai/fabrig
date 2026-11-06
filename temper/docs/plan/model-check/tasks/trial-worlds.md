+++
id = "trial-worlds"
part = "trial-worlds"
kind = "feature"
needs = ["trial-start", "account-row", "home-line"]
files = [
  "temper/tests/client/tests/suite/model_check.rs",
  "temper/crates/temper/tests/fake_backends/walk/model_check.rs",
]
effort = "high"
minutes = 110
+++

# The trial's stories, joined

## Goal

The request's tests, at their lowest tier, past what each piece's own test shows. `the_success_story` stands, from `trial-start`.
A walk on a fake box proves the whole path a chat takes.

Acceptance, in `temper/tests/client/tests/suite/model_check.rs`, each failing first where its code is yours to fix:

- `the_first_step_stops_on_a_login_not_signed_in`: the host's check settles not usable: "Almost ready", Create disabled, the
  line Stopped at its first step, "Try again" and "Sign in again".
- `the_second_step_stops_on_a_run_that_cant_start`: a run ending failed before past "Starting": "Ready", Create works, the line
  Stopped at "Claude Code starts on this machine".
- `the_third_step_stops_on_a_run_that_ends_without_answering`: past "Starting" then failed: Stopped at "Claude answers".
- `a_minute_stops_it`: the world's clock.
- `a_wait_says_so_at_once`: for a wait of `first-usable-model`'s, a lapsed login once the trial is out: the wait's words at
  the next delivery, not at the minute.
- `no_model_is_named`: the claim's assignment is the first usable's, and the task's document's `llm` is the choice `next_llm` made,
  beside a second model's account that a named trial would have taken.
- `a_person_whos_left_sees_it_on_home`: they press Create before it fails. Home says "Claude didn’t answer when tried", and its
  "Try again" answers.
- `a_restart_spends_no_second_turn`: the engine restarts with a trial out: its attempt ends, no other is claimed, no line stands;
  a reload on a usable account after it asks nothing.
- `removing_the_account_cancels_it`: through the accounts page.
- `the_welcomes_layout_doesnt_change_for_the_check`: the tree's nodes other than Model's line, compared at the press, while the
  host's check is out, at its answer, while the trial runs and once it answers: identical. This is the flash the request names: 305 ms
  and a different layout.

In `temper/crates/temper/tests/fake_backends/walk/model_check.rs`, on the fake box, the fake model answering by its
`otherwise` and the fake `claude`: `a_trial_runs_through_the_worker_and_the_fake_model`: the welcome's Claude way, Ready at once, the
line reaching "answered in N s"; no chat in Chats; Home's counts none.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`, `notes/client.md`.
- `temper/docs/design/domain/trial.md`, whole.
- `temper/docs/design/domain/testing.md`, 2.1, 2.6, 4.1.
- `temper/docs/plan/model-check/inputs/request.md`, *What the plan must hold*.
- `temper/tests/client/tests/suite/agent_recovery.rs`: a failed attempt's precedent.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/tests/client/src/**`: helpers; `temper/crates/temper/tests/fake_backends/walk.rs`, `support/**`: a `mod` line and a
  helper.
- **Code a story shows wrong,** in `temper-client-view/src/pages/model_check.rs`, `temper-client-domain/src/forge/trials.rs`,
  `temper-engine-domain/src/route/trials.rs` and `temper-engine-domain-trials`: a fix, a test first, named in your report.
- `smith/tests/fake-claude/**` isn't yours: a story needing it to hang or fail uses the scripted worker in the client world instead.

Options, each pre-approved:

- **A walk's fake `claude` default doesn't answer a trial:** set its story in the walk's support, as the real loop's
  `temper/tests/real-loop/src/claude.rs` does. If the fake box gives no way to do that without changing `temper-fake`, use the
  plan's way or a key's on the fake model for the walk, and say so.
- **A walk is over its tier's budget:** by `AGENTS.md`, 5: first whether it belongs here; a measured question otherwise. The client
  world's stories stand on their own without it.

## Steps

Each story names the step that stopped, by its words in the tree, not by a state's name. No sleeps: the world's clock.

The checks: fmt, `old_forms.py` for `temper`, clippy on what you changed, `heavy cargo nextest run -p temper-client-world --test suite -E 'test(model_check)'`, the walk's own, and the focused tests
of any code you fixed. Citations. Push.

## Report

As the implementer's text asks: each story's bite and the walk's time.
