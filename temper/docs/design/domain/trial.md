# A model's trial

Provisional, 2026-11-06. One short real task, once a model's account is usable, through the path a chat
takes: the worker, smith and the model. Written from `../draft/model-check.md`. This document holds what
runs, who pays, what ends it and what crosses to the client. What the person reads, Model's line and its
words, is `../client/README.md`, 3.5, *The try*.

## 1. In one page

- **What runs:** one task of charter 10, naming no `llm`, in no project, paid for by the person (2).
- **Who keeps it:** `temper-engine-domain-trials`, a child of the root, in memory: a row per account (3).
- **What starts it:** the client, when an account becomes usable under the welcome's Model step or Ready, or
  `ConnectModel`, and "Try again" (6). Nothing else does: a restart starts nothing.
- **What ends it:** its first committed turn, as answered; a run ending otherwise, or a minute, as failed (3.2).
  Each ends the task at once, so no second turn is spent.
- **What it takes from no one:** a trial's end is no fact of the account, and keeps no one out (7).

## 2. The task

- **Its charter:** charter 10, `TRIAL_CHARTER` (`../protocol/configuration.md`, 13.4): no tools, one working
  turn, charter 1's models and hosts.
- **What it asks:** the words `Say one word.`, its contract a report of at most 64 bytes, no job, parameters or
  inputs.
- **It names no `llm`,** so that it selects as a new chat does (jig's `domain/accounts.md`, 4.3): a trial that
  named its model could pass while chats still hung. So it takes the first model with a usable account, and that
  may be another account than the one that started it: its row's `took` says which (3.1).
- **In no project:** the welcome's Model step has none yet, so its task is jig's person's task (jig's
  `domain/authority.md`, 6.1), its payer the person. So no list holds it: it is not in Chats, Needs you or Home's
  counts, and it writes nothing to a project's document.
- **No workspace:** a run with no repository, as the status agent's (`forge.md`, 5.7).
- **Who pays and what:** the person, as for a chat. On a plan that is its included usage; on a key one short turn
  at the key's price. It spends only when something starts it (1): once as an account first becomes usable on those
  pages, and at each "Try again".

## 3. The child

`temper-engine-domain-trials` is a step: the root routes to it and carries out what it asks. It knows the core by
no type: tasks are numbers, accounts are numbers, a person is a number.

### 3.1 Its rows

One row per account, the newest trial's:

| Field | What it holds |
|---|---|
| `account` | the account whose becoming usable started it |
| `person` | who pays, the account's holder |
| `task` | its task; none where the core refused to make it |
| `began` | the wall time it started |
| `took` | the account the run keeps, once it has claimed one |
| `step` | `Starts` until the run is past "Starting", then `Answers` |
| `state` | `Running`, `Answered { seconds }` or `Failed { why }` |

- **A row is the account's, not a page's:** leaving a page stops nothing. It lives until its account is removed,
  another trial replaces it, or the engine restarts.
- **Nothing of it is stored,** its task's row aside (5). Formats change nothing.
- **Its room:** the deployment's accounts (`Limits::rows`); a row past it is refused, the oldest ended going first.

### 3.2 What moves it

| Event | From | Becomes, and asks |
|---|---|---|
| `Try { account, person, again: false }` | no row | a new row, `Running`; asks `Make` |
| `Try { again: false }` | a row | nothing: an account is tried once |
| `Try { again: true }` | ended | a new row in its place; asks `Make` |
| `Try { again: true }` | `Running` | nothing |
| `Made { account, task }` | `Running` | `task` set |
| `Refused { account }` | `Running` | `Failed { why: Refused }` |
| `Observed { task, past_starting, took }` | `Running` | `step` and `took`, each only forward |
| `Turn { task }`, `Finished { task }` | `Running` | `Answered { seconds }`, at least 1; asks `Cancel` |
| `RunFailed { task, class }` | `Running` | `Failed { why: Ended { class } }`; asks `Cancel` |
| `Due` | `Running`, a minute past `began` | `Failed { why: Waited }`; asks `Cancel` |
| `AccountGone { account }` | any | the row goes; asks `Cancel` if `Running` with a task, and likewise for rows that `took` it |

Every change asks `Changed { person }`. An event for a task no row holds is nothing.

- **A minute** is the draft's, `WAIT`, from `began`, whatever the task is waiting for. The child's
  `next_due() -> Option<Wall>` is the root's timer.
- **A committed turn is the first committed text:** a trial's run has no tools to call. The root reads turns, not
  their bytes.
- **A failed run ends it.** jig would try again, past 1 to 30 s, and each try may spend: the trial's first ending
  is its end (jig's `domain/engine.md`, 7).

## 4. The root

`temper-engine-domain`'s `route/trials.rs` joins the child to the core, and holds nothing:

- **`Application::Try { account, again }`,** admitted with its person, whose answer is `Tried` or, where the person
  doesn't hold that account, `NotTheirs`.
- **`Make`:** `Core::person_seed`, then `Event::PersonTask` with `Executor::Agent { charter: TRIAL_CHARTER }`,
  and `Made` with the seed's number; no seed, `Refused`.
- **Lines:** what `route_core_requests` sees saved for a task a row holds: `Line::Turn` is `Turn`, `Line::RunEnded`
  with `Finished` is `Finished`, with `Failed { class }` or `Refused` is `RunFailed`. An account's erase is
  `AccountGone`.
- **`Observed`:** while a row runs, each step reads `Core::run_past_starting` and the task's `kept_account`, and
  tells the child only what changed.
- **`Cancel`:** `tasks::Event::Cancel`, by `Party::Deployment { project: 0 }`.
- **Period 0** is no project's: the calendar takes no period of project 0.
- **`run_workspace`** answers a trial's run as a status run's: none.

## 5. A restart

A live trial's task is restored as any live task, and would run again, so the root cancels every live task of
charter 10 before its start lets a run be claimed. Its row isn't restored: Model's line goes back to its ticked
state with no try's fact, and Home says nothing. Nothing starts another: a restart spends no turn. A person's
next way through a page may, having a cause of its own: they've signed in again.

## 6. What crosses to the client

The forge's family (`temper-client-protocol-wire`), its scope the person's own:

```
record TryAccount { account: u32 again: bool }        # Request::try_account, answered tried or not_theirs
enum TrialStep { starts, answers }
enum TrialClass { transient, permanent, run, agent, lost, invalid, refused }
enum TrialWhy { ended: TrialEnded, refused, waited }  # record TrialEnded { class: TrialClass }
enum TrialState { running, answered: TrialAnswered, failed: TrialFailed }
record TrialRow { account: u32 task: option u64 took: option u32 step: TrialStep state: TrialState }
versioned record Trials { rows: list 64 TrialRow }    # Watch::trials
```

- **The client starts one** when the `accounts` document shows an account of the person's usable that its last
  delivery in this session did not, while the page is the welcome's Model step or Ready, or `ConnectModel`. Two
  tabs ask twice, and the row stands for both. A reload in the gap asks nothing, and the line stays ticked.
- **From the ask to its row** the line is `Running`: nothing goes ticked in between.
- **Why it waits** is the Task document's `run_wait`, which the person who pays may watch (jig's
  `domain/authority.md`, 6.1), in the words a chat's wait has (`../client/README.md`, 5.5). A wait
  that a moment settles, a source `Asking`, `Refreshing`, `Granted` or `Moving`, isn't a stop; any other says so at
  once, on a row still running, and goes if the wait ends. Its row's `Waited` keeps those words where the client saw
  them, and otherwise says only that it had no answer in a minute.

## 7. What it stops

Nothing of the account: a source's own state is the host's check's (jig's `domain/accounts.md`, 3.4). Ready's
heading and act read the host's check alone (`../client/README.md`, 3.5, *Ready*). A failed trial is a line in
the pane, on the account's row and on Home, and "Try again".

## 8. Later

- **Trying a second account:** a trial takes what a new chat takes, so an account of a later model isn't what
  answers while an earlier one is usable. Naming an account means naming a model, which is what the trial doesn't
  do. Its row's `took` says so, and the account's own row shows no try's fact.
- **A deployment's accounts** are the owner's, on a document only the accounts page watches, and aren't tried.
- **Settings:** whether the accounts page offers "Try it" on an account (the draft's 5).
- **A key whose `uses` name a project alone** isn't covered in no project, so its trial waits saying there's no
  account, where a chat in that project wouldn't.
- **Surviving a restart:** a failed trial's line is lost at a restart. Storing it would hold a line, not a fact,
  and nothing has asked.
