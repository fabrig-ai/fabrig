# D017: `unreachable!` re-narrows values the types could carry

- **Kind:** complexity
- **Where:**
  - `jig-domain-host/src/hosted.rs`, `jig-domain-host/src/jobs.rs`, `jig-domain-host/src/hosted/start_refused.rs`;
  - `jig-domain-fleet/src/attempt.rs`;
  - `jig-domain-hub-link/src/lib.rs`, `jig-domain-hub-link/src/payload.rs`;
  - `jig-domain-notes/src/read.rs`, `jig-domain-notes/src/domain.rs`, `jig-domain-notes/src/drop_project.rs`;
  - `jig-domain-people/src/domain/inbox.rs`, `jig-domain-people/src/domain/start.rs`, `jig-domain-people/src/scope.rs`;
  - `examples/ops/domain/src/client_documents.rs`;
  - `jig-protocol-accounts/src/party.rs`, `jig-protocol-accounts/src/component.rs`;
  - `jig-domain-core/src/acts.rs`, `jig-domain-core/src/effects.rs`, `jig-domain-core/src/reads/vocabulary.rs`,
    `jig-domain-core/src/reads.rs`, `jig-domain-core/src/reads/rows.rs`, `jig-domain-core/src/reads/settings.rs`,
    `jig-domain-core/src/routing/effect/note.rs`, `jig-domain-core/src/settings/charters.rs`,
    `jig-domain-core/src/routing/run/jobs.rs`, `jig-domain-core/src/routing/run/preparation.rs`,
    `jig-domain-core/src/tests/jobs.rs`, `jig-domain-core/src/acts/bounds.rs`, `jig-domain-core/src/admission/roles.rs`,
    `jig-domain-core/src/runs/account_preference.rs`, `jig-domain-core/src/routing/run/calls.rs`,
    `jig-domain-core/src/routing/party/decisions.rs`, `jig-domain-core/src/acts/objects/decisions.rs`;
  - `jig-domain-tasks/src/compound/staged.rs`, `jig-domain-tasks/src/run.rs`, `jig-domain-tasks/src/waits.rs`,
    `jig-domain-tasks/src/close_project.rs`;
  - `jig-client-view/src/tree/sections/task.rs`, `jig-client-view/src/tree/pages/not_the_work.rs`,
    `jig-client-view/src/state/ui.rs`, `jig-client-view/src/words/phrases/english.rs`;
  - `jig-protocol-store/src/restore/reader.rs`, `jig-protocol-store/src/codec/settings.rs`,
    `jig-protocol-store/src/reads/indexed.rs`, `jig-protocol-store/src/directory.rs`,
    `jig-protocol-store/src/patching/phase.rs`, `jig-protocol-store/src/patching/phase/walking.rs`,
    `jig-protocol-store/src/transcripts/read.rs`;
  - `jig-client-protocol-link/src/component.rs`, `jig-client-protocol-link/src/sign_in.rs`,
    `jig-client-protocol-link/src/translate/documents/frame.rs`;
  - `jig-client/src/thread.rs`, `jig-client/src/facts.rs`, `jig-client-domain/src/watches.rs`.
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Signals in the step code": `unreachable!` arms)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T7 "the same smell elsewhere", A1 F12, A5 F10, A7 F6)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 9: about 372 outside tests; T7: notes `ReadPending`, engine-clients' `Waiting`)
- **Checked:** 2026-11-05 at `dd9dbc92de` (each remaining site below, after debt-21's narrowing changes)
- **Value:** medium (a pairing the types encode cannot be broken by a later change)
- **Size:** M
- **Status:** open
- **Paid when:** each `unreachable!` left in production guards a state machine's invariant; none re-matches a value whose variant an earlier match already fixed.
- **Measure:** `rg -o 'unreachable!' jig/crates jig/examples -g '!**/tests/**' -g '!**/tests.rs' -g '!**/tests_*.rs' -g '!**/generated/**' | wc -l` (274 at `dd9dbc92de`; 382 at debt-21's base)

## What

274 `unreachable!` in production code. Most guard a state machine (`jig-domain-host/src/hosted.rs`, nearly all of
`jig-domain-fleet/src/attempt.rs`) or another step's promise on a shared enum, about 30 of them, and those stay.
What still re-narrows:

- **A public type would change:** hub-link's one payload map of five families (`jig-domain-hub-link/src/lib.rs`,
  `payload.rs`; `Payload`, `retain`, `payload` and `take_payload` are read by `temper-engine-domain`, `jig-ops-domain` and
  `jig-test-domain`); notes' `Rows` (`read.rs`, `domain.rs`, `drop_project.rs`; read by `jig-protocol-store`); people's
  `Update` and `Stored` (`domain/inbox.rs`, `domain/start.rs`); `ApplicationScope` (`scope.rs`; ops' `client_documents.rs`);
  accounts' `party_account`, `restore` and `Component::new`; `acts::Creating`; `EffectOrigin::Accept`
  (`jig-domain-core/src/effects.rs`); `reads::Scope` and `Range`.
- **A stored form would change:** `Held { why, source }` (`jig-domain-tasks/src/compound/staged.rs`).
- **Others are changing these:** `jig-client-view` (`task.rs`, `not_the_work.rs`, `ui.rs`, `english.rs`); the core's
  `reads.rs`, `reads/rows.rs` and `reads/settings.rs`; `routing/effect/note.rs`.
- **Small and free, left for size:** `jig-protocol-store` (`reader.rs`, `settings.rs`, `indexed.rs`, `directory.rs`, `phase.rs`,
  `walking.rs`, `read.rs`: take first and pass the fields); `jig-client-protocol-link` (`component.rs`, `sign_in.rs`, `frame.rs`);
  `jig-domain-fleet/src/attempt.rs`'s three "as matched above"; `jig-domain-host` (`jobs.rs`, `hosted/start_refused.rs`);
  `jig-client` (`thread.rs`, `facts.rs`); `jig-client-domain/src/watches.rs`; the core's job admission
  (`settings/charters.rs`, `routing/run/jobs.rs`, `routing/run/preparation.rs`: one assertion in `tests/jobs.rs` names its
  return) and its permanent refusal (`acts/bounds.rs`, `admission/roles.rs`, `runs/account_preference.rs`).
- **Not worth their code at this size:** the core's double matches on a recorded action (`routing/run/calls.rs`,
  `routing/party/decisions.rs`, `acts/objects/decisions.rs`), which would take a 28-arm enum; `jig-domain-tasks/src/run.rs`; the
  tasks' `judge` (`waits.rs`); `close_project.rs`.

## Why it matters

The invariant lives in the programmer's head, not in a type; a refactor that breaks the
pairing compiles and aborts at run time.

## What pays it back

Maps keyed by family, so a lookup returns the narrow type; types that carry the pairing
(a read and the document answering it). Follows from D002 and D015.
