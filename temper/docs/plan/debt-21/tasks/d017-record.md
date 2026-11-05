+++
id = "d017-record"
part = "narrowing"
kind = "docs"
needs = ["narrow-settings", "narrow-application", "narrow-effects", "narrow-core-local", "narrow-authority", "narrow-tasks", "narrow-engine-clients", "narrow-notes-reads"]
files = ["jig/docs/development/debt/D017-unreachable-re-narrowing.md"]
effort = "medium"
minutes = 15
+++

# jig D017: the record says what's left

## Goal

`jig/docs/development/debt/D017-unreachable-re-narrowing.md` stays open: its **Paid when** can't hold in this plan. It says what this plan took, what's left and why, so that the next round starts from it.

Acceptance:

- The item isn't deleted, and **Status** and **Paid when** stay.
- **Measure** keeps its command, and gives the count you measured, at your base's commit, beside "382 at debt-21's base". The cut expects about 274; merges from other plans move `reads*` by a site or two. A count over 280 is a `Question:`, naming each `narrow-*` task's file that's over its count.
- **Where** and **Checked** are brought up to date, naming what `## What` names. **Seen** lines stay.
- `## What` is as below, with its numbers measured. Check each named site still exists; drop a line that has gone.

  ```
  <count> `unreachable!` in production code. Most guard a state machine (`jig-domain-host/src/hosted.rs`, nearly all of
  `jig-domain-fleet/src/attempt.rs`) or another step's promise on a shared enum, about 30 of them, and those stay.
  What still re-narrows:

  - **A public type would change:** hub-link's one payload map of five families (`jig-domain-hub-link/src/lib.rs`,
    `payload.rs`; `Payload`, `retain`, `payload` and `take_payload` are read by `temper-engine-domain`, `jig-ops-domain` and
    `jig-test-domain`); notes' `Rows` (`read.rs`, `domain.rs`, `drop_project.rs`; read by `jig-protocol-store`); people's
    `Update` and `Stored` (`domain/inbox.rs`, `start.rs`); `ApplicationScope` (`scope.rs`; ops' `client_documents.rs`);
    accounts' `party_account`, `restore` and `Component::new`; `acts::Creating`; `EffectOrigin::Accept`
    (`jig-domain-core/src/effects.rs`); `reads::Scope` and `Range`.
  - **A stored form would change:** `Held { why, source }` (`jig-domain-tasks/src/compound/staged.rs`).
  - **Others are changing these:** `jig-client-view` (`task.rs`, `not_the_work.rs`, `ui.rs`, `english.rs`); the core's
    `reads.rs`, `reads/rows.rs` and `reads/settings.rs`; `routing/effect/note.rs`.
  - **Small and free, left for size:** `jig-protocol-store` (`reader.rs`, `settings.rs`, `indexed.rs`, `directory.rs`, `phase.rs`,
    `walking.rs`, `read.rs`: take first and pass the fields); `jig-client-protocol-link` (`component.rs`, `sign_in.rs`, `frame.rs`);
    `jig-domain-fleet/src/attempt.rs`'s three "as matched above"; `jig-domain-host` (`jobs.rs`, `start_refused.rs`); `jig-client`
    (`thread.rs`, `facts.rs`, `watches.rs`); the core's job admission (`settings/charters.rs`, `routing/run/jobs.rs`,
    `routing/run/preparation.rs`: one assertion in `tests/jobs.rs` names its return) and its permanent refusal
    (`acts/bounds.rs`, `admission/roles.rs`, `runs/account_preference.rs`).
  - **Not worth their code at this size:** the core's double matches on a recorded action (`routing/run/calls.rs`,
    `routing/party/decisions.rs`, `acts/objects/decisions.rs`), which would take a 28-arm enum; `jig-domain-tasks/src/run.rs`; the
    tasks' `judge`; `close_project.rs`.
  ```

- `python3 -I -B tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/narrowing.md`
- `jig/docs/development/debt/README.md`, 2: how an item is kept

## Owns and may change

- **Owns:** the one file. Nothing is built or tested. It's the note's "What this plan doesn't pay", written down, and no site changes here.

## Steps

1. The item's own **Measure**; each named file by `rg -c 'unreachable!'`.
2. The edits. Citations after you commit. Commit and push.

## Report

The common note's items; the count; any line you dropped from the list, and why.

## Answers

2026-11-05. Resume after `narrow-tasks` pushes its implementation, or record 299 above the 280 limit? Resume; don't record 299. `narrow-tasks` has since pushed its implementation (`origin/debt-21/narrow-tasks`, at `c0666f1e35`, still running): all ten files you named read their targets there, and that branch merged with the trunk measures 274 by the item's **Measure**. As your brief says, `git fetch origin` and merge `origin/debt-21/narrow-tasks` again before you measure, and again before you push; record the count measured at your merged commit. The 280 limit stands: a count still over it is a `Question:` as the acceptance says.
