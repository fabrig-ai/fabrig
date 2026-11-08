+++
id = "ways-family"
part = "read"
kind = "feature"
needs = ["ways-surface", "ways-wire", "stock-ways"]
files = ["temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/family/**", "temper/crates/temper-service/src/route.rs", "temper/crates/temper-service/src/tests/family.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 100
+++

## Goal

Carry the root's `ways` document and its `View` answer to the client: the
service maps the wire's `Watch::Ways` to the root's subject and
`Read::View` to `ForgeRead::View`, and encodes `Delivery::Ways` as a
`WaysPanel` and `ForgeReadAnswer::View` as a `ViewAnswer`
(`temper/docs/design/client/README.md`, 4.6;
`temper/docs/design/client/forge.md`, section 9).

Acceptance:

- `family.rs`: `Watch::Ways(subject)` becomes the forge's application
  watch of the project's scope, subject
  `temper_engine_domain::ways::subject(project)`, as `Watch::Changes` does;
  `Read::View(read)` becomes `ForgeRead::View { project, address }` in the
  project's scope, refused where the read's scope is not that project's;
  `ForgeReadAnswer::View` is encoded as `ViewAnswer`.
- `route.rs`: `Delivery::Ways` is delivered as the forge family's record
  `WaysPanel`, as `Delivery::Approvals` is.
- One translation of a path, `ways::Path` to the wire's `BranchPath`, shared by
  both: each stop's actor, step, words, opens and card; a way's `items`
  as `ProposalItem`s, `Set` encoded by `temper_engine_smith::kinds::encode`
  as `forge_read`'s `ProposalItems` arm encodes its values, `Remove` as
  `removed`. A value past a wire bound is refused as the family refuses
  any record it can't make (the document not sent, the read answered
  none), never cut.
- Tests in `src/tests/family.rs`, rows beside the `changes` ones: a
  `WaysPanel` encoded from a hand-built document (three ways, a current
  index, a set and a removed item, a card with four facts) decodes back
  to its values; `Watching { ways }` maps to the project's subject; a
  `Reading { view }` maps to `ForgeRead::View` and a `View::Proposal`
  answer encodes and decodes; the worst document of the subject at the
  wire's ceilings fits one decision's delivery, as the changes panel's
  test does (`:3062`).
- `temper/docs/design/client/README.status.md`: the sentence "4.6: the
  records `BranchPath`, `WaysPanel` and `ViewAnswer`, the subject `ways` and the
  read `View`." leaves its **Not yet**, the rest staying.

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`; `notes/contracts.md`,
  sections 1, 3 and 4 (binding).
- `temper/docs/design/client/README.md`, 4.6 (*A path*, *A project's
  ways*, *A view*); `temper/docs/design/client/forge.md`, section 9.
- The pattern: `family.rs`'s `Watch::Changes` arm, `changes_panel`,
  `approvals`, `forge_read` and `forge_settings_query`; `route.rs`'s
  `Delivery::Approvals`.

## Owns and may change

- **Owns:** the `ways` and `view` arms and translations in
  `temper-service/src/family.rs` (or a new `family/ways.rs` it declares);
  `route.rs`'s `Delivery::Ways` arm; the tests above.
- **Options, pre-approved:** if the decision's delivery bound refuses the
  worst `WaysPanel`, do not raise a limit: report the measured figures as
  a `Question:` with the wire's ceilings. If `temper-service` lacks the
  `temper-engine-domain-ways` dependency, add it (one line).

## Steps

1. The path translation; the panel and the view answer.
2. The watch and read mappings; the delivery arm.
3. The tests; break the translation once (swap two actors) to see the
   round trip fail, and restore.
4. The status entry.
5. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-service --lib -E
   'test(/family/)'`, then `-p temper-service --lib`. Commit and push.

## Report

As the implementer's text asks; give the worst panel's encoded bytes.
