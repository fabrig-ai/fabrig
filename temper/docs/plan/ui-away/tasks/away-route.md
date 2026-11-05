+++
id = "away-route"
part = "away-route"
kind = "feature"
needs = ["away-wire", "away-forge"]
files = [
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/admission.rs",
  "temper/crates/temper-engine-domain/src/route/applications.rs",
  "temper/crates/temper-engine-domain/src/route/playground.rs",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/tests/engine/domain/tests/suite/panels.rs",
]
effort = "high"
minutes = 60
+++

# The root and the service carry the switch, Overturn and their documents

## Goal

A client's `SetAway` and `Overturn` reach the forge, their outcomes come back in the wire's words, and `away` and `decided`
deliver: `temper/docs/design/client/README.md`, 4.6 and section 8.

Acceptance:

- `notes/contracts.md`, *The root and the tool*, for the two applications.
- **Scope:** `SetAway` and `Overturn` are of the project they name, as `Dismiss`; a party with no role there is refused at
  admission, in jig's words.
- **The subjects:** `away` is of the party's own scope and `decided` of the project's, each document built for the asking party;
  a party without a role in the project gets no `decided`.
- **Outcomes:** `away`; `forge refused, too large`; `overturned`; `overturn_refused` with `already` or `unknown`; a request's
  outcome no longer held is *unknown*.
- **A party's rows:** two parties' rows and documents are each their own, in a world's test.
- `temper/docs/design/client/README.status.md` and `domain/forge.status.md` lose what this closes.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 4.6, section 8; `temper/docs/design/domain/forge.md`, 5.8 and 5.9.
- The models: `Application::Dismiss` and `Looked` in `temper-engine-domain/src/boundary.rs` and `route/applications.rs`;
  `temper-service/src/family.rs`' gate, requests, subjects, answers and documents; commit `bdf141ef2a`, which routed `Looked`.

## Owns and may change

Owns, as `files` lists them. Status lines as above. May also change: compiler-forced arms in `temper/tests/engine/domain/src/**`
and `temper-fake`; `temper/crates/temper-engine-domain/src/tests/**` for an arm. Replace `away-wire`'s and `away-forge`'s placeholder
arms in `family.rs` and `panels.rs`.

Options, each pre-approved:

- **`until`'s conversion:** the wire's u64 to `Wall` where `family.rs` turns a request, as `ChangesLanded.at` goes the other way.
- **A client's `kinds` above 15:** the wire's validation refuses it; the root assumes none.

## Steps

1. The applications, the admission scope, the answers.
2. `family.rs`: gate, requests, subjects, answers, documents.
3. Tests: `temper-service/src/tests/family.rs`: `set_away_reads_as_the_forges_event`, `overturn_reads_as_the_forges_event`,
   `away_and_decided_subjects_are_scoped`, `their_documents_encode`. `temper/tests/engine/domain/tests/suite/panels.rs`:
   `a_partys_away_is_its_own`, `decided_is_the_asking_partys_own`, `a_party_without_a_role_is_refused_at_admission`.
4. The check of the implementer's text, with `temper-engine-domain`, `temper-service` and `temper-engine-domain-world`
   (`temper/tests/engine/domain`'s package, by its `Cargo.toml`), and push.

## Report

As `notes/common.md`, Your report.

## Rebase (2026-11-05, step merge, foreman ui-away/away-route/foreman-1)

combine Away’s narrowed status with the landing target’s removal of the completed timezone gap
Rebase `31db682713` onto the landing target.
Resolve the conflict in `temper/docs/design/client/README.status.md` by preserving both changes:
- Keep this task’s replacement: `3.5, *Away*: its client domain and view.`
- Keep `0a4db1a83b` (“Read client calendar times in the browser's zone”): remove `5.12: the zone, which every form reads as universal time.`

The combined line should begin:
`- **Not yet:** 3.5, *Away*: its client domain and view. 3.5, *Home on any project*:`

Preserve the target’s other status entries, including its Goals and Settings updates. Keep the task’s root/service implementation and `domain/forge.status.md` reduction. The refusal reports only this documentation conflict; no code repair is indicated.
