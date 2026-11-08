# core-after-project: the plan

2026-11-08. From `inputs/request.md`: a deployment's core stopped answering right after `project make`, and again after a restart on the same store. Reproduce it, find what the core waits on, fix it so that a store already in that state opens and answers, and make a core that cannot answer say why. Three tasks, under the request's five, so nothing is dropped. Item 4's client half beyond the sign-in needs a wire change, and is a question.

## The design

- **`temper/docs/design/domain/root.md`, section 4,** gains *A stopped journal ends the engine, naming why*. A store answer the journal refuses (skein's `lib.md`, 11) stops it like the root's own stops do, and every stop emits `Stop` with its reason. A root whose journal has stopped never stays up undeciding.
- **`temper/docs/design/protocol/README.md`, 8.2,** *When the core does not answer*: the service's gate and its terms; the log line `temper serve: the core is not answering: REASON` once a party's input has waited 1 s, again on a change, and `the core answers again`; the reasons' words, the start's steps in 9.1's words among them; and the stopped journal's exit line.
- **`temper/docs/design/client/tools.md`, 8.1:** a sign-in answered not ready to its deadline says `temper is not ready: it is starting, or its core is not answering, and temper serve's log says why`, where it said the deadline's generic line.
- **Status:** `domain/root.status.md` holds *Not yet* entries for 4 (the silent journal stop) and for 5.7 and 9.1 (the hang itself); `protocol/README.status.md` holds one for 8.2; `client/tools.status.md` holds one for 8.1's line. Each task removes its own.

## Parts and tasks

| Part | Task | Needs | Crates | Minutes |
|---|---|---|---|---|
| `core-answers` | `hang-story` | none | the world that hangs (engine domain world, `temper-service`'s tests, or the one-process world); the fix in the lowest crate that holds the cause | 360 |
| `core-says-why` | `sign-in-line` | none | `temper-client-shell-cli` (the sign-in's deadline line) | 45 |
| | `why-log` | `hang-story` | `temper-engine-domain` (`JournalStop`, the `Committed` arm's `Stop`), `temper-service` (`core_wait`'s `Watch`, `Stop::Journal`, the notes), `temper` (`host.rs`'s lines) | 180 |

## Critical path

`hang-story` (360) → `why-log` (180): about 540 implementer-minutes. `sign-in-line` (45) runs beside `hang-story` from the start. 585 implementer-minutes in all.

## Decisions

- **What reading found** (three read-only surveys and the cut's own reading). The evidence fits one state alone: the service's gate shut because the root is no longer `ready()`.
  - A private project's public read reaching jig answers 404. A 503 at once is the gate's `not_ready` (`temper-service/src/clients/translation.rs`), which also answers every party's input not ready and keeps the clients' component `starting`, so no link comes up.
  - At runtime the gate's other terms belong to startup alone. `ready()` fails only with a start stop or a stopped journal, and a start stop or a failed commit ends the engine loudly.
  - The one silent way there is a store answer the journal refuses (`skein-lib/src/journal.rs`, `committed`), taken in `temper-engine-domain/src/route.rs`'s `Committed` arm, which emits no `Stop`. The root's own `committed` wrapper even documents duplicates as inert, which skein's journal does not do.
  - No producer of such an answer was found. skein-kv numbers and answers each commit once, and jig's store fails loudly on a mismatch. So `hang-story` reproduces before it fixes, and its spec pre-approves the fix by the family of cause.
- **The world is a ladder,** lowest first: the engine domain world (the root on its store, `Driver::reopen`), then `temper-service`'s tests (the gate, and jig's store over in-memory maps reopened by `restarting_from`), then the one-process world (skein-kv's log files on the simulator's disk). The story lives in the first that hangs; all three run in the focused suite. If none hangs, the whole sequence lands green in the one-process world as a guard, and the hang is reported as unreproduced.
- **Item 5, read, not run: which preconditions matter.** By reading:
  - The forget (act 2) and the refused declare (act 4) leave no wait for the root or the core. Forgetting erases the account's rows and keeps its issued name. The declare is refused `NotOffered` for that issued name, its flight retired and a keyed refusal kept (`jig-domain-accounts/src/changes.rs` and `programs.rs`; `jig-domain-core/src/routing/party/account_requests.rs`). Act 2 explains act 4, and neither is needed for the hang.
  - The earlier goal (act 1) leaves forge records (landings, the outbox) that only a start's `ReadAfresh` and `SettleOutbox` read (`temper-engine-domain-forge/src/domain.rs`). It can hold a restart, not the running engine. It also grows the store's commit count and log.
  - The settings revision (act 3) reaches `project make` only through `Setting::Policy`, the configured policy a project copies (`jig-domain-core/src/admission/projects.rs`). Which two items changed is not known.
  - `project make` (act 6) and its follow-ups wait on nothing. The period is opened in its decision, `ProjectMade` is released after its commit, the calendar's `Began` gives `Event::Period` with a next start, and the existing ledger takes `PeriodNext` (`jig-domain-core/src/routing/party/projects.rs`, `routing/run/delegation.rs`; `temper-engine-domain/src/domain.rs`, `step_released`). What it adds is commits.
  - So reading points at act 6's commits meeting a store answer the journal refuses, with acts 1 and 3 as the likeliest preconditions. `hang-story` runs the acts to settle it, and its report gives the smallest sequence.
- **`why-log` waits for `hang-story`:** both may write `temper-engine-domain`'s journal handling, and the log must be able to name the cause `hang-story` finds. `hang-story`'s story comment carries the cause into the repository for it.
- **The client's half of item 4** is split by the wire. The sign-in already hears `NotReady` (`jig_client::NotSignedIn::NotReady`), so its line says so now (`sign-in-line`). A read's or an act's link hears a watch refused `Busy`, and nothing says why, so saying more needs jig's client wire: the question `not-ready-on-the-wire`, which no task waits for.
- No stored format or protocol changes in this plan. A fix that would need one stops `hang-story` with a question, its story pushed red.

## Overlaps

- **`ui-ask-m1`** (integration, not yet landed) changed `temper-engine-domain`'s `route.rs`, `domain.rs`, `lib.rs` and `tests.rs`, and `temper-service`'s `route.rs`. `why-log` edits `temper-engine-domain`'s `route.rs` (the `Committed` arm), `decision.rs`, `domain.rs` and `lib.rs`, and `temper-service`'s `engine.rs`, `stop.rs` and `lib.rs`. `hang-story`'s fix may land in the same root files. This could not be avoided: the cause sits in the root's journal handling, or near it. Each task keeps its hunks to what it names.
- **`ui-account-again`** (cut in flight) repairs re-declaring a forgotten account, in jig's accounts and people code and its words. No task here touches them (`notes/common.md`).
- **`temper-fuzzy-gate`, `survey-answer`:** none of their files is this plan's.
