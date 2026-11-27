# A model's trial

Provisional, 2026-11-07. One short real task, once a model's account is usable, through the path a chat
takes: the worker, smith and the model. Written from `../draft/model-check.md`. This document holds what
runs, who pays, what ends it and what crosses to the client. What the person reads, Model's line and its
words, is `../client/pages/README.md`, *The try*.

## 1. In one page

- **What runs:** one task of charter 10, in no project, paid for by the person. A person's first usable account's
  names no `llm`; a later account's names that account, so that it is the one tried (2).
- **Who keeps it:** `temper-engine-domain-trials`, a child of the root, in memory: a row per account (3).
- **What starts it:** the client, when an account becomes usable under the welcome's Model step or Ready, or
  `ConnectModel`, and "Try again" (6). Nothing else does: a restart starts nothing.
- **What ends it:** its first committed turn, as answered; its account's usage limit, as limited, with the reset
  where the run gives one; a run ending otherwise, or a minute, as failed (3.2). Each ends the task at once, so no
  second turn is spent, and nothing tries again at a reset.
- **What it takes from no one:** a trial's end is no fact of the account, and keeps no one out (7).

## 2. The task

- **Its charter:** charter 10, `TRIAL_CHARTER` (`../protocol/configuration/stock.md`): no tools, one working
  turn, charter 1's models and hosts.
- **What it asks:** the words `Say one word.`, its contract a report of at most 64 bytes, no job, parameters or
  inputs.
- **The first usable account's names no `llm`,** so that it selects as a new chat does (jig's `domain/accounts-uses.md`,
  1.3): a trial that named its model could pass while chats still hung. It is first when the person's offer in project
  0 (jig's `domain/engine/views.md`, 1), within the deployment ceiling's models, holds no account but the one tried.
  Those are the models a person's seed allows (jig's `domain/authority.md`, 6.1); the offer is not narrowed to
  the trial's charter for this decision. Then what a new chat takes is that account.
- **A later account's names it.** Where that offer holds any other account, the person's own or the deployment's, the
  trial's `llm` is jig's `Core::account_llm` for the account tried: that account, and the charter's first model offered
  with it, else the first it serves, at its default effort (jig's `domain/accounts-uses.md`, 1.3). Its run takes that
  account alone, and waits for it, saying why, rather than moving to another. So whichever provider comes first, in
  either order, and for a second account of one provider, the account just connected is the one tried, and its row's
  `took` is its own (3.1). An account that serves none of the charter's models can't be tried: `Refused` (3.2).
- **Each `Make` judges it again,** a "Try again" included: a first account tried again once another is usable names
  itself.
- **In no project:** the welcome's Model step has none yet, so its task is jig's person's task (jig's
  `domain/authority.md`, 6.1), its payer the person. So no list holds it: it is not in Chats, Needs you or Home's
  counts, and it writes nothing to a project's document.
- **No workspace:** a run with no repository, as the status agent's (`forge/attention.md`, 3).
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
| `state` | `Running`, `Answered { seconds }`, `Limited { reset }` or `Failed { why }`; the last three are ended |

- **A row is the account's, not a page's:** leaving a page stops nothing. It lives until its account is removed,
  another trial replaces it, or the engine restarts.
- **Nothing of it is stored,** its task's row aside (5). Formats change nothing.
- **Its room:** `Limits::rows`, the least of the deployment's account limit, the wire's 64 rows and the number of
  rows that fit one delivery's room (`temper-engine-domain`'s `room::trial_rows`); a row past it is refused, the
  oldest ended going first.

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
| `RunLimited { task, reset }` | `Running` | `Limited { reset }`, `step` `Answers`; asks `Cancel` |
| `RunFailed { task, class }` | `Running` | `Failed { why: Ended { class } }`; asks `Cancel` |
| `Due` | `Running`, a minute past `began` | `Failed { why: Waited }`; asks `Cancel` |
| `AccountGone { account }` | any | the row goes; asks `Cancel` if `Running` with a task, and likewise for rows that `took` it |

Every change asks `Changed { person }`. An event for a task no row holds is nothing.

- **A minute** is the draft's, `WAIT`, from `began`, whatever the task is waiting for. The child's
  `next_due() -> Option<Wall>` is the root's timer.
- **A committed turn is the first committed text:** a trial's run has no tools to call. The root reads turns, not
  their bytes.
- **A failed run ends it.** jig would try again, past 1 to 30 s, and each try may spend: the trial's first ending
  is its end (jig's `domain/engine/executors.md`).
- **A limit is its own ending,** neither a refusal nor a failed start: the model answered, with its account's usage
  limit, so the run was past starting and its step is `Answers` whatever was observed. `reset` is the wall time the
  limit lifts, as the run gave it to jig, and none where it gave none: never a reset the run did not give.
- **Nothing tries at the reset.** jig lifts the account's hold at the reset, and the row stays `Limited`: only
  `Try { again: true }`, a person's "Try again", runs another, since a try spends and nothing spends unasked (the
  user, 2026-11-27).

## 4. The root

`temper-engine-domain`'s `route/trials.rs` joins the child to the core, and holds nothing:

- **`Application::Try { account, again }`,** admitted with its person, whose answer is `Tried` or, where the person
  doesn't hold that account, `NotTheirs`. `route/applications.rs` keeps that answer in memory for its key's retention, as
  it keeps a return's, so a replayed key spends nothing.
- **`Make`:** the trial's choice (2), before allocating a seed: `Core::offer` for `Payer::Party(person)` in project 0
  with the deployment ceiling's models; where it holds an account other than the row's, `Core::account_llm` for
  charter 10 and the row's account, and none is `Refused`; otherwise no choice. An unserved later account is refused
  before `Core::person_seed`, so no unused seed opens funding. Then `Core::person_seed` and `Event::PersonTask` with
  `Executor::Agent { charter: TRIAL_CHARTER }` and that `llm`, and `Made` with the seed's number; no seed, `Refused`.
- **Lines:** what `route_core_requests` sees saved for a task a row holds: `Line::Turn` is `Turn`, `Line::RunEnded`
  with `Finished` is `Finished`, with `Refused` while the account is at its limit (*A limit*) is `RunLimited`,
  otherwise with `Failed { class }` or `Refused` is `RunFailed`. An account's erase is `AccountGone`.
- **A limit:** jig ends a run that meets its account's usage limit as `EndKind::Refused`, holds the account's source
  `Limited { until }` with the reset the run gave, and sends the task back to wait for it; the line carries no
  cause. So the root reads the cause from jig's accounts, at the line: the account the task kept (its
  `kept_account`), else the row's, whose deciding source (jig's `account_state`) is held `Limited { until }`, makes
  `RunLimited` with `reset` that `until`'s wall, none where it has none. A running row whose task waits on its
  source held `Limited` (*Its wait*) is at its limit too, and is `RunLimited` with that wait's `until`, at once: the
  trial waits for no reset.
- **`Observed`:** while a row runs, each step reads `Core::run_past_starting` and the task's `kept_account`, and
  tells the child only what changed.
- **Its wait:** while a row runs with a task, each step reads that task's run wait, as jig's Task document gives it
  (its `run_wait`, made by jig's core from the task's row), and where it differs from the one the root last read,
  keeps it and marks `TRIALS_SUBJECT`; the trials' document carries it on the row (6). The root holds it in memory
  alone, a row's at a time, and a restart reads it again.
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
enum TrialClass { transient, permanent, run, agent, lost, invalid, refused, infrastructure }
enum TrialWhy { ended: TrialEnded, refused, waited }  # record TrialEnded { class: TrialClass }
record TrialLimited { reset: option u64 }            # the reset's wall, in nanoseconds
enum TrialState { running, answered: TrialAnswered, failed: TrialFailed, limited: TrialLimited }
record TrialRow { account: u32 task: option u64 took: option u32 step: TrialStep state: TrialState run_wait: option bytes 1024 }
versioned record Trials { rows: list 64 TrialRow }    # Watch::trials
```

- **`limited`** is appended to `TrialState`, so every earlier variant keeps its tag. The family's version stays, and
  its layout, its fingerprint, moves (`../protocol/wires.md`, 9): a client of the previous build finds temper's
  owner not served at its hello and reloads once to the engine's bundle, as at any change to this family. No stored
  form changes: a row is never stored (3.1).

- **`run_wait`,** from the forge family's version 29 (`../draft/page-round-trips.md`, section 4): jig-client-protocol-wire's
  `TaskRunWait`, encoded at jig's `VERSION`, as `CoreRefused` carries jig's refusal (`../client/forge/records.md`): the
  running row's task's wait (4, *Its wait*); none on a row not running, with no task, or whose task waits on nothing.
  The client follows a running trial by its row alone and opens no watch of its task.

- **The client starts one** when the `accounts` document shows an account of the person's usable that its last
  delivery in this session did not, while the page is the welcome's Model step or Ready, or `ConnectModel`. Two
  tabs ask twice, and the row stands for both. A reload in the gap asks nothing, and the line stays ticked.
- **From the ask to its row** the line is `Running`: nothing goes ticked in between.
- **Why it waits** is its row's `run_wait`, the Task document's, which the person who pays may see (jig's
  `domain/authority.md`, 6.1), in the words a chat's wait has (`../client/pages/accounts-words.md`, 1). A wait
  that a moment settles, a source `Asking`, `Refreshing`, `Granted` or `Moving`, isn't a stop; any other says so at
  once, on a row still running, and goes if the wait ends. Its row's `Waited` keeps those words where the client saw
  them, and otherwise says only that it had no answer in a minute.

### 6.1 Model's line

`temper-client-view`'s `pages/model_check.rs`, `trial` and `line`, is the one place a row becomes words
(`../client/pages/README.md`, *The try*). Which state it draws, in this order:

1. **The host's check out:** Running at the first step. **Settled not usable:** Stopped at it, in the host's check's
   words; "Try again" checks the host again.
2. **A trial asked, or its row `Running`, with a wait that's no moment's:** Stopped, its why the wait's words, its step
   the row's. Its "Try again" asks nothing more while the row runs, so it isn't offered until the row ends.
3. **Its row `Running`, or asked:** Running. The first step ticks as a usable account's does; for a subscription `Starts`
   is the second step under way and `Answers` the third; for a plan or a key the second is `Answers`, whatever the row's
   step.
4. **`Answered { seconds }`:** Answered, "answered in N s".
5. **`Limited { reset }`:** Limited, at the last step, every step before it ticked, its why by 6.4: before the reset,
   the limit; once the page's clock reaches it, that the limit has passed. The page turns at the reset by its own
   clock, with no delivery.
6. **`Failed { why }`:** Stopped at the row's step, its why by 6.4.
7. **A usable account and no row:** Usable, ticked, with no try's fact.

ACCOUNT and WHO come from the row's `took`, or, until then, its `account`. On the welcome the line's row is the newest
of the person's: the document's order is newest first.

- **One row, one state:** every page that draws a row, the Model step, Ready and the account's row, draws the same
  state and the same step marks for it, from the row and its account alone; only 1's host's check, while it is
  under way or settled not signed in, stands in for the row, on the pages that ran it.
- **A source held is signed in.** The host's check, and a sign-in's end, judge whether the source is signed in, not
  whether it is usable: a source held by a limit, or by any hold, finished its sign-in. Its line is its row's, never 1's
  stop, and a sign-in's banner never stands for it (`../client/pages/welcome-model.md`, 1, *Not finished*).

### 6.2 The account's row

The accounts page's row of an account, under its head and before its sources, draws `line` for its row: the row whose
`took` is that account, or with no `took` whose `account` is. "Sign in again" goes to the way's page on `ConnectModel`,
not the welcome's; a limit's "All ways" goes to `ConnectModel` with no way chosen, where another plan or a key is
connected. No row, or a row answered for another account: nothing, and the row reads as it does today.

### 6.3 Home

While a row of the person's is `Failed` or `Limited`, every project's Home holds one line under its intro, whatever
else that intro says: for `Failed`, "WHO didn’t answer when tried"; for `Limited`, its why by 6.4, the limit until the
reset and that it has passed after; each in `HOME_BYLINE`'s look, and "Try again", which runs another and takes the line
away until that one ends. Answered says nothing more; a running row says nothing. No new look: Home's line is drawn in
what Home has.

### 6.4 Why, in the person's terms

| Why | The words |
|---|---|
| `Ended { class: Transient }`, `Infrastructure` | "WHO was busy or out of reach just now." |
| `Ended { class: Permanent }` | "WHO refused the request. Its account may need attention." |
| `Ended { class: Run }`, `Agent`, `Invalid` | "The run stopped before WHO answered." |
| `Ended { class: Lost }` | "The machine running it went away." |
| `Ended { class: Refused }`, `Refused` | "The try couldn't start." |
| `Waited` | the wait's words, if the client saw one; otherwise "WHO didn’t answer within a minute." |
| `Limited { reset }`, before it | "ACCOUNT is at its LIMIT until TIME.", as "Claude subscription is at its 5-hour limit until 09:30." |
| `Limited { reset }`, once it has passed | "ACCOUNT's limit has passed." |
| `Limited` with no reset | "ACCOUNT reached its usage limit." |

- **LIMIT** names the account's window where the `accounts` document holds exactly one of its usage windows limited
  (`../client/pages/accounts-words.md`, 1, *Usage*): `five_hour` "5-hour limit", `seven_day` "weekly limit"; any other,
  several or none, "usage limit". The window is jig's fact; the run says no window.
- **TIME** is the reset in the person's local time where the page knows its offset, the browser's: "09:30" on the
  page's day, "Nov 28, 09:30" on another. The terminal knows no offset, and says it in UTC: "09:30 UTC".
- **"Passed"** is the page's clock at or past the reset. With no reset nothing says it has passed: the account's
  state is the accounts page's to say.

A wait's words are those a chat's wait says, with no command after them: never a command for temper's own tools. Each
offers "Try again". The answering act, quiet, is "Sign in again" only where the cause is the source's sign-in: a
wait on a source that needs a sign-in, is expiring or is signed in on no host, or a `Permanent` ending whose account is a
sign-in or a program's. A limit's ending, or a wait on a source held by its limit, needs no sign-in: its act is "All
ways", on the welcome and on an account's row (6.2), to connect another plan or a key. Otherwise "All ways" on the
welcome and nothing on an account's row.

## 7. What it stops

Nothing of the account: a source's own state is the host's check's (jig's `domain/accounts-sources.md`, 1). Ready's
heading and act count an account just declared as usable until its host's check says otherwise
(`../client/pages/README.md`, *Ready*); the trial changes neither. A failed or limited trial is a line in
the pane, on the account's row and on Home, and "Try again". A limit's hold on the account is jig's, not the trial's:
the trial ends as limited and holds nothing.

## 8. Later

- **Real-account verification** (2, 6.2): after landing, connect Claude then ChatGPT, ChatGPT then Claude, and
  two accounts of one provider on a real box. Check that the later account's own row shows its trial and that
  "Try again" retries it; the first usable account's trial still names no `llm`.
- **A deployment's accounts** are the owner's, on a document only the accounts page watches, and aren't tried.
- **Settings:** whether the accounts page offers "Try it" on an account (the draft's 5).
- **A key whose `uses` name a project alone** isn't covered in no project, so its trial waits saying there's no
  account where it is the first, and can't start where it is a later one (2), where a chat in that project
  wouldn't.
- **Surviving a restart:** a failed trial's line is lost at a restart. Storing it would hold a line, not a fact,
  and nothing has asked.
- **A limit already past at the run's end:** where the reset the run gives has passed when jig reads it, jig holds
  nothing and refuses the run with a backoff, so the account reads no limit at the line and the trial reads as a
  refusal (4, *A limit*). Nothing records that limit for the root to read.
- **The limit's window from the run:** the run says no window; LIMIT is the `accounts` document's (6.4). A program's
  limit event naming its window would let the words name it where no usage read is held.
