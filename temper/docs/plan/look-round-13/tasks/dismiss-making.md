+++
id = "dismiss-making"
part = "dismiss"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-protocol-wire/**", "temper/crates/temper-engine-domain-forge/src/making.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/tests/making.rs", "temper/crates/temper-engine-domain/src/route/applications.rs", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-protocol-store/**", "temper/crates/temper-client-domain/src/forge/family.rs", "temper/tests/engine/domain/tests/suite/making.rs", "temper/tests/codecs/tests/suite/fuzzy/client_wire.rs"]
effort = "high"
minutes = 60
+++

## Goal

A failed repository making can be dismissed: the wire's and the forge connector's request `DismissMaking`, which erases a failed row, frees its place in `forge.makings`, and is sent by nothing in the view yet (`sheet-rows` does). The client's `forge::dismiss_making` builds it.

Acceptance:
- a failed row dismissed is erased, and the forge's watch shows it gone; its place in `forge.makings` is free;
- no row at the name answers `Kept` (a request sent again joins); a row waiting, making, made or adopting answers `Refused(NotFailed)` and goes on unchanged;
- admitted as `MakeRepository` is, from the request: an owner of the project it names, or with none a deployment owner; others refused by role; a row whose `into` is not the project the request names (a row with no `into` against a request naming one, and the other way) refused as `NotOwner` and unchanged;
- the same key sent again, after the row is gone and after a restart, replays the first answer;
- the wire round-trips it in the fuzzy codec test and its goldens are regenerated;
- `temper-engine-domain-forge`'s, `temper-client-protocol-wire`'s and `temper-service`'s focused tests pass.

## Inputs

- `temper/docs/design/domain/forge.md`, 4.5, *A failed row is dismissed*; `temper/docs/design/client/README.md`, 4.6, `DismissMaking`; `temper/docs/design/draft/repository-copies.md`, 5.2.
- `temper/docs/plan/look-round-13/notes/common.md`, and `notes/contracts.md`, *`DismissMaking`*.

## Owns and may change

- Owns the `files` above. Follow `MakeRepository`'s path through each: the wire's and the store's schemas and generated code (regenerate with each crate's generator, never by hand; `MakeRefusal` is kept in the store's answer), `temper-service/src/family.rs`'s `w::Request` arm, `domain.rs`'s `Ask` arm, `making.rs`.
- May change, for the compiler alone: every exhaustive `match` on `w::Request`, `Ask` or `MakeRefusal`, which include `temper-client-tools/src/forges.rs`, `forges/lifecycle.rs`, `forges/connection.rs`, `temper-client-domain/src/tests/form.rs`, `temper-engine-domain-forge/src/make.rs`, `connect.rs`. `MakeRefusal::NotFailed`'s words where a `match` needs them, in `temper-client-domain`'s and `temper-client-tools`' existing refusal words: "It is not failed.".
- `CLIENT_VERSION` and the wire's version change in place, with no older side kept (`repository-copies.md`, section 1). A wire change makes journey recordings stale: re-record with `again` in this task (`notes/common.md`, *Journeys*).
- The scope comes from the request's `project` alone, in `temper-engine-domain/src/route/applications.rs`, as `MakeRepository`'s `into` gives its own; the root reads no row and no person's roles for it. The connector's `making::dismiss` compares the request's `project` with the row's `into`.

## Steps

1. Schema, generated, version; `Ask`, its route and `service/family.rs`; `making::dismiss` beside `making::admit`, in `temper-engine-domain-forge/src/making.rs`, erasing the row by the same path removal of a forge uses for its rows.
2. Test sketch in `temper-engine-domain-forge/src/tests/making.rs`: a failed row dismissed is erased and frees `forge.makings` (a request past full then is kept); a dismissal of no row is `Kept`; of a running row `Refused(NotFailed)`, the row unchanged; by a stranger refused; naming another project than the row's `into`, `Refused(NotOwner)`, the row unchanged. A world story in `temper/tests/engine/domain/tests/suite/making.rs`: copy fails, dismiss, the row gone on the watch, restart keeps it gone.
3. `forge::dismiss_making`, with a test beside `make_repository`'s.
4. Goldens; focused tests of the packages and their direct dependents; `cargo fmt`, old forms, clippy; commit with the trailer; push.

## Report

As the implementer's text has it.

## Answers

2026-11-06 (temper/ui). Neither: keep the scope stable without a lookup. temper/ui answers. Carry the failed row's project in the request, as MakeRepository's into does: the client has it, from the row it's dismissing. Derive the scope from the request, not from the row, so a repeat of the same key has the same scope and jig replays it, after a restart too. Admission: an owner of that project, or with none a deployment owner. A row that exists with a different into is refused, and no row answers Kept. So no change to jig, and no Own-scoped admission with its own role checks in temper. Your world test's same-key replay then passes as it stands: keep it. Note this in plan.md as a departure from the spec's line reading into from the row. Below this, run focused and fuzzy alone, with no render check: the user's rule, and render checks follow landing.

2026-11-06 (the answer call). Jig's retained scope lookup, or Own-scoped admission? Neither, as temper/ui answered above; the spec above is amended to it, and the checkpoint `4c33c881fd` is kept and changed:
- `DismissMaking` and `Ask::DismissMaking` gain `project`, an optional u32 (`notes/contracts.md`); `forge::dismiss_making` takes it. Wire and goldens regenerate again, in the same version bump.
- `route/applications.rs`: `Some(project)` is `ApplicationScope::Project(project)`, `None` is `Deployment`; the arm's row read and search of the person's roles go.
- `making::dismiss`, in this order: the role is not owner, `NotOwner`; no row, `Kept`; the row's `into` is not the request's project, `NotOwner`; failed, erased and `Kept`; otherwise `NotFailed`. No new refusal.
- World tests: `dismissal_authority_comes_from_the_rows_destination` sends the row's `into` as the project and adds the mismatch; `dismissal_without_a_row_admits_a_deployment_or_project_owner` names the project for its project owner. The same-key replay assertion stays as it is.
- The design and `plan.md` already say this; this task's report need not repeat it. Jig is not touched.
