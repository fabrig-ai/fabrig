# A check's judgement

How temper judges what a check ended with, at a change's CI, a landing's gate and a goal's close: one function reads what the check showed of a verdict and its run's facts, and gives one of four classes. Only a refusal is the change's: the runner's fault and a check that broke never ask a repair. A delivery's check is smith's to judge, by the same rule (3). Whether a project's checks can flake at all is the project's to say (4). Written from `../../draft/whose-red.md`, 3, and 8's receipts bullet.

## 1. What a check shows

- **A record,** where its site promises one: a gate's configured report (`../landing-proof.md`, 2.3; `../landing-settings.md`, 1.1, kind 5's `report`) and a close's result (`../../protocol/tooling.md`, 4.2). It passes, names the change's own failures, or names nothing: missing, there but not read as its format, or naming no failure. **A record's promise binds:** a record that names nothing is no verdict, whatever the run's steps say.
- **What the run shows,** where its site promises none: a change's CI, and a gate that names no report. A failure in a workflow's own step, a failed check that names no job, and one whose run can't be read are the change's, as they were.
- **The runner's facts,** read from the run's and its jobs' states, never from a log's words (`../landing-proof.md`, 2.4; `change/README.md`, 2, step 3): a job whose set-up step failed, or whose completion step alone failed, or that failed with no step failed; a job or a run cancelled by no one of temper's; a run never found, found at another commit or lost; a dispatch refused for a missing workflow or runner; a close's job that jig ended lost. A job or run cancelled by no one of temper's is the run **interrupted**: the forge's states give no other sign of a run stopped at its wall, since skein's job conclusions are success, failure, cancelled and skipped.
- **The check's own error,** at a close: an exit other than 0 or an asked-for 3, a signal, the job's deadline, or a result missing, past its bound or not its attempt's (`../../protocol/tooling.md`, 4.2).
- **Starved tests, inherited failures and flakes** are the check's to name, and a check passes with them (`tools/pre-pr/design.md`, 6; `skein/docs/foundation/testing-strategy.md`, 8.1): temper judges the verdict the check gives.

## 2. The classes

One function, the same at every site, gives each outcome one class, the first that holds:

1. **Refused:** the check showed the change's own failures, a record naming them or, where no record is promised, a failure the run shows with no fact of the runner's. **A shown red stands:** a runner's fault or a broken check in another job of the same check never hides it.
2. **The runner's:** nothing the change caused is shown, and a job of the run holds a fact of the runner's, the first of them naming it. So a promised record that names nothing is the runner's when its run was interrupted.
3. **Broke:** nothing the change caused is shown and no fact is the runner's: the check's own error, or a promised record that names nothing from a run that wasn't interrupted.
4. **Passed:** a passed verdict, with whatever warnings the check named.

What follows:

- **Only a refusal is the change's.** It asks the site's action (3). The runner's and broke never ask a repair, never spend one and never reach a repair's brief as its red. Each is tried again within its allowance and then held, its words saying the change wasn't judged and why: the runner's naming the job and its fact, broke naming the check and what it showed.
- **Advisory gates** are judged the same and hold nothing (`../landing-proof.md`, 2.2).
- **An agent's verdict on its repair** (a flake, no failing test) is no class: `change/repairs.md`, 1, reads it.
- **The function owns no allowance.** Each site keeps its own count of tries and its own hold, as 3 names them.

## 3. At each site

| Site | What it shows | Refused | The runner's or broke |
|---|---|---|---|
| A change's CI | the run (no record) | a counted repair (`change/repairs.md`, 1) | run again within `forge.change_policy.reruns`, then held (`change/README.md`, 2, step 3) |
| A landing's gate | its report where it names one, the run otherwise | a failed receipt, its unit repaired (`../landing-proof.md`, 2.3) | retried within `gate_retries`, then the landing held; no receipt (`../landing-proof.md`, 2.4) |
| A goal's close | its result | held at once (`change/repairs-releases.md`, 1, *Refused*) | tried again in its job's class, then held (*Broke*, there) |
| A delivery's checks | smith's: the check's exit and what it said | the run told `checks-failed` | the run told `checks-not-judged`, and may deliver again (smith's `domain/run-delivery.md`, 1.1) |

- **A delivery is smith's to judge,** since smith names no code of temper's: the same rule over what a check can show smith, its exit and its stderr. temper reads only a check's record of reds (`reds.md`, 5.5), which a check that broke doesn't write.

## 4. Whether a project's checks can flake

- **The project says,** in kind 4's `flakes` (`../landing-settings.md`, 1.1): `none`, or `rerun N`, N at most the deployment's `checkout.rerun_processes`. Unset, it is `rerun` at that figure, as before.
- **With `none`:** every checkout a worker prepares for the project's runs and jobs has `temper-rerun 1 0` (`reds.md`, 5.2); the landing gate runs no known-flake rerun (`../landing-regate.md`, 2); and no flake of a delivery's check is filed or counted (`reds.md`, 5.8). A test that fails is the change's red, or a repair shows it isn't. A test that passes alone after failing beside others is still red, its log naming the tests before it.
- **With `rerun N`:** a worker's marker is N, at most its own `checkout.rerun_processes`; the rest is as before.
- **A CI runner's checkout is the project's own:** temper prepares none, so the project's command writes the marker its setting states where it finds none, as fabrig's goal check does (`tools/pre-pr/design.md`, 2, *Delivery and CI entry points*).
- **fabrig's is `none`** (the user, 2026-11-25): its checks run only tests that replay, which own their clock, seeds and kernel (`tools/tiers/layers.md`, 6.1), so a test that fails and then passes is a determinism bug, fixed where it lies. The reference's settings carry it (`../../protocol/configuration/reference.md`, 2).

## 5. Across builds

- **A retained failed receipt that names nothing,** its report read with no failed test or not read at all, is judged again by 2 before it's reused: it is no verdict, so it is dropped and its key dispatched as a key with no receipt (`../landing-proof.md`, 1.2). A previous build made such receipts.
- **A previous build's worker** keeps no bound the engine sends, so a project stating `flakes` runs only on workers of this build (`reds.md`, 5.2). The reference deployment's hand-set figure of 0 covers its workers until then (`../../draft/whose-red.execution.md`, 3).

## 6. Tests

All in the focused suites, with the fuzzy sweeps' scenarios:

- **The function,** in its crate: each class from its facts, a shown red beside a runner's fault and beside a broken check, a record naming nothing from an interrupted run and from one that wasn't.
- **At each site:** a change's CI whose runner's fault outlasts `reruns` holds and asks no repair, and a cancelled job is the runner's; a gate whose report names no failure, or can't be read, is retried within `gate_retries` and holds with no receipt, and a gate with a red job beside a runner's fault makes its failed receipt; a close's refusal naming no failure is broke; a delivery's check that says it didn't judge is told so. P37's scenario (`a_starved_gate_is_the_runners`) passes in the forge world's sweeps.
- **`flakes`:** with `none`, a worker's marker reads 0, the gate runs no known-flake rerun and a delivery's flake is neither filed nor counted; with `rerun N`, the marker is the lesser of N and the worker's figure; unset, as before.
- **Across builds:** a retained failed receipt with an empty report dispatches its key again.

## 7. Open questions

None.
