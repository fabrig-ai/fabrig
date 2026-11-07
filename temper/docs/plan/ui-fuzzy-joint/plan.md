# ui-fuzzy-joint: the plan

A budget red's repair: jig's fuzzy suite read 152.1 to 158.0 CPU s against its 120 from `model-catalogue-1`'s landing, all of it in `jig-ops-world::suite fuzzy::joint` (`inputs/request.md`).

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| joint-cost | `joint-cost` | | compares a profile at `e7bb41d364` with one at its base, names the cause with figures, fixes it at its lowest place, and measures before and after on one builder |

## Critical path

`joint-cost`, alone.

## Decisions

- **One task, not two.** The request allows two, and measuring then fixing reads as a natural split. But the fix can't be specified until the profiles exist, a cut doesn't wait for a run, and a pushed spec never changes. A second spec could only say "fix what the first found", so measuring and fixing stay in one task, which holds the options the figures choose between.
- **The cause is carried as unproved.** Nothing was run at this cut: a call runs on a machine that builds nothing, so the profile the request asks for is the task's. The spec gives what reading the code found as the first thing to test, with the figures that would rule it in or out.
- **What the reading found.** `model-catalogue-1` made `Touch::Accounts` mark `Subject::Frame`, and `route_account` marks it on nearly every `accounts::Event`, with nothing comparing the new document to the last. Each delivery is costly in this world: the `Wire` moves 1 to 31 bytes a round, each round is a `Bytes::pump` iteration, and each document costs a view build. That would explain why the scripted engine's `fuzzy::client::*` didn't move and why both halves of the profile grew. If so, real clients were paying too, and the fix belongs in the code under test, as the request says.
- **The default fix compares around the step.** `route_account` compares each watched Frame before and after the account's step and marks only those that moved. `jig/docs/design/domain/engine.md`, 11.1 already says `me` is marked as any of it moves and that changes invalidate the affected offers, so this needs no design change. `jig/docs/design/client/domain/contract.md`, 7.2 keeps only a mark and a window per watch, which rules out keeping the last delivered document or its hash, so the snapshot lives no longer than the step. Every other `touch_accounts` caller keeps marking unconditionally: an offer that time moves still needs its alarm.
- **Figures read against 70, not 120.** The task runs `jig-ops-world` alone, as the request says. Of the 153, that binary read 103, so the other 50 plus 70 makes the 120. That assumes the other 50 stays put, and the trunk's end check reads the suite itself.
- **The `Wire` stays closed.** Its schedule is every tree's seeded world and it isn't what moved. Changing it would make every byte cheaper by giving each story a different test.
- **Harness speed-ups come last and are reported apart.** Repeated work in `Bytes::pump` and `Net` predates the landing, so it isn't the cause. It's allowed only if the fix to the cause doesn't reach the figure, with each seed's schedule and every record check kept.
- **No design sections, notes or questions.** One task, nothing crosses a part, and the design already says what the default fix does. The spec lets the task correct one design line its fix makes false, as the request allows.

## Overlaps

`model-catalogue-2` is changing `jig-domain-core`'s `documents/catalogue.rs`, `documents/small.rs`, `lib.rs` and `runs/llm.rs`, and `jig-protocol-engine-clients`' `translate/**`.

- **The default fix avoids all of them:** it stays in `routing/children.rs`, `routing/view/marks.rs`, `documents.rs` and two test files.
- **Options B and C can't avoid them,** if the figures pick those: whichever lands second catches up, and the task doesn't take on that plan's work.
