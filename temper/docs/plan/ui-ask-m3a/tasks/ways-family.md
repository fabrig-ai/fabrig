+++
id = "ways-family"
part = "read"
kind = "feature"
needs = ["ways-surface", "ways-wire", "stock-ways"]
files = ["temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/family/**", "temper/crates/temper-service/src/route.rs", "temper/crates/temper-service/src/tests/family.rs", "temper/docs/design/client/README.status.md", "temper/crates/temper-engine-domain-ways/src/**", "temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema", "temper/crates/temper-client-protocol-wire/src/generated/v1.rs", "temper/crates/temper-client-protocol-wire/golden/**", "temper/crates/temper-client-protocol-wire/src/lib.rs", "temper/crates/temper-client-protocol-wire/src/tests.rs"]
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
  answer encodes and decodes; the worst document the root makes fits one
  decision's delivery under Small, as the changes panel's test does
  (`:3062`), by the answer below.
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

## Answers

**2026-11-08.** *The worst `WaysPanel` (2,600,069 bytes; own path alone
67,321) passes Small's 65,536: revise the document's bounds, or raise the
allowance?* Revise the bounds; raise nothing; keep the shape. The wire's
ceilings were the wrong measure: the derivation fills a path from fixed
words and the stock's items are fixed data, as the changes test measures
its document by the forge's limits.

1. **Bounds** (`notes/contracts.md`, sections 1 and 3, already amended):
   `STOPS` 32 → 24 and `VALUE` 384 → 128 in
   `temper-engine-domain-ways` (`path.rs`); in the schema, `BranchPath`'s
   `stops: list 24 Stop` and `Fact`'s `value: text 128`. Regenerate the
   generated code and goldens with the codec's generator, as `ways-wire`
   did (never by hand), and fix any of the ways crate's or the wire's tests
   that name the old figures. `temper/docs/design/client/README.md`, 4.6,
   is already amended. `ways-wire` foresaw 24 as the floor; go no lower.
2. **The test** `the_worst_small_ways_document_fits_its_decision`: the
   worst document the root makes under Small, not the wire's ceilings:
   - `ways`: the stock's three, `crate::stock::ways()`, with their names,
     lines and `ways::items(way, project, Some(kind 7))`, kind 7 at its
     largest;
   - each path, the three and `own` (`current` none): `repository`
     `OWNER/NAME`, each part at Small's `limits.forge.name_bytes`;
     `branch` at that bound; `STOPS` stops, each the largest agent stop
     `derive` makes: actor `Agent`, step `ProduceMany`, words
     "produce ×n", opens `Charter(u32::MAX)`, the card of contracts
     section 2's repair (`who` "an agent · repairs what a check or a
     review found, then goes back through them", `name` "the coding
     agent") with "Model" at `VALUE` bytes, "Can" all four families'
     words (110 bytes), "Asked to" at `VALUE` bytes. An approval's or a
     tier's stop is smaller (words and name 64, one fact of `VALUE`);
     say so in a comment.
   - Assert the encoded panel within `result_bytes.max(transcript_bytes)`
     and report its bytes. Estimated about 56,000. If it still fails,
     report the figures as a `Question:`; don't lower past 24 and 128.
3. `Decision::deliver` admitting `Delivery::Ways` with no byte check is
   `ways-surface`'s file: leave it, under `## Left`.

Then the steps' runs, plus `-p temper-engine-domain-ways` and
`-p temper-client-protocol-wire` (`--lib` and its tests) and `temper/tests/codecs`'s
drift test, by name.
