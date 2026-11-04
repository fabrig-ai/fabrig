+++
id = "dismissed-read"
part = "engine"
kind = "feature"
needs = ["dismiss-request"]
files = ["temper/crates/temper-engine-domain-forge/src/views.rs", "temper/crates/temper-engine-domain-forge/src/dismissed.rs", "temper/crates/temper-engine-domain-forge/src/tests/views.rs", "temper/crates/temper-engine-domain/src/route/panels.rs", "temper/crates/temper-engine-domain/src/route/asks.rs", "temper/crates/temper-engine-domain/src/tests/documents.rs", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/tests/family.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/tests/root_checks.rs"]
effort = "high"
minutes = 35
+++

# The subject `dismissed`, served

## Goal

A party's client watches `dismissed`, of its own scope, and receives its own rows, again whenever a
dismissal commits (contracts K5). temper's first application subject of the party's own scope; jig
serves it as it stands (`Ask::Document` names the party).

Acceptance:

- K5's `Document::Dismissals`, `document` with `party`, `panels::document` with it.
- `to_root`'s `w::Watch::Dismissed` is K5's `Watched::Application`, replacing the forced arm.
- `family::document` encodes `Document::Dismissals` as `wire::dismissals`.
- A start whose `forge.judge_projects` is past `wire::DISMISSED` is a problem naming both, in
  `startup/checks.rs`, on the shape of `sign_in.members`' check against `forge.bindings`.
- In `temper-engine-domain-forge/src/tests/views.rs`: `dismissed_is_each_watchers_own_rows`,
  `the_own_scope_answers_no_other_subject`.
- In `temper-engine-domain/src/tests/documents.rs`:
  `a_dismissal_makes_the_watchers_document_again` (an open watch of the subject, a `Dismiss`
  committed, a second `Delivery::ForgeDocument` holding its row);
  `two_parties_watching_dismissed_get_their_own_documents`.
- In `temper-service/src/tests/family.rs`: the watch's subject and the document's bytes; in
  `tests/root_checks.rs`: `judge_projects_past_the_wires_dismissals_name_both`.
- `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain` pass and `-p
  temper-service` narrowed by `-E` to `family` and `root_checks`; `heavy cargo check --tests -p
  temper-client-world -p temper-one-process-world` pass.

## Inputs

- `temper/docs/plan/to-begin-store/notes/common.md`; `notes/contracts.md`, K3 and K5.
- `temper/docs/design/domain/forge.md`, 5.5, "Its document".
- `temper/docs/design/client/README.md`, 4.6 ("A person's dismissals, the subject `dismissed`").
- `temper/docs/design/protocol/configuration.md`, 7 ("the dismissals'").
- jig's `jig/docs/design/domain/connectors.md`, 5.5 ("the scope each names").

## Owns and may change

- The files of the header. `views.rs`: `Document`, its `owned_bytes`, `document`. `lib.rs` of
  `temper-engine-domain-forge`: an export line.
- `document`'s callers: `route/panels.rs` and `tests/views.rs`, the new argument alone.
- `temper-engine-domain/src/decision.rs`, if `owned_bytes` is matched there: the forced arm.
- Status: remove "5.5: a person's dismissals..." from
  `temper/docs/design/domain/forge.status.md` and "7: the check of `forge.judge_projects`..." from
  `temper/docs/design/protocol/configuration.status.md`.
- Pre-approved:
  - One `DISMISSED_SUBJECT` for every party: a mark makes each watcher's document again
    (`temper/docs/design/domain/forge.md`, 5.5). Keep it; no token per party.
  - If jig's views refuse, drop or merge an own-scope subject between two parties, stop and ask:
    that is a change of jig's, which this plan names none of.

## Steps

1. `Document::Dismissals`, `owned_bytes`, `document` and its callers; `panels.rs`.
2. `family.rs`: the watch and the document; `checks.rs`.
3. The tests, each biting: answer every party's rows, and remove the mark in the decision, each once.
4. The two status lines; the implementer's checks; commit and push.

## Report

As the implementer's text asks; say how many deliveries a dismissal made with two parties watching.
