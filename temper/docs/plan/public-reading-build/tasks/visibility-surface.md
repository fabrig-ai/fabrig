+++
id = "visibility-surface"
part = "jig-public"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-people/src/boundary.rs", "jig/crates/jig-domain-people/src/words.rs", "jig/crates/jig-domain-people/src/capacity.rs", "jig/crates/jig-domain-people/src/models.rs", "jig/crates/jig-domain-people/src/domain/**", "jig/crates/jig-protocol-store/schema/people-v14.schema", "jig/crates/jig-protocol-store/schema/secrets-v1.schema", "jig/crates/jig-protocol-store/src/generated/people_v14.rs", "jig/crates/jig-protocol-store/src/generated/secrets_v1.rs", "jig/crates/jig-protocol-store/src/generated/layouts.rs", "jig/crates/jig-protocol-store/src/codec/people.rs", "jig/crates/jig-protocol-store/src/codec/people/**", "jig/crates/jig-protocol-store/src/codec/secrets*", "jig/crates/jig-protocol-store/src/keys/**", "jig/crates/jig-protocol-store/src/boundary.rs", "jig/crates/jig-protocol-store/golden/people/**", "jig/crates/jig-protocol-store/golden/secrets/**", "jig/crates/jig-protocol-store/src/tests/**"]
effort = "high"
minutes = 150
+++

# A project's visibility as a people request and a stored row; the public names' secret

## Goal

The people child's vocabulary and the store gain what a project's visibility is made of, so
the core's task can decide it and the wire's can carry it: `SetVisibility` as a people
request, the row that says a project is public, and the secrets' row that will hold the
public identifiers' key (`jig/docs/design/domain/people.md`, 13.2 and 13.8;
`jig/docs/design/protocol/store.md`, 4.4 and section 9). Nothing decides them yet: the people
child refuses `SetVisibility` until `visibility-core`.

Acceptance:

- `jig_domain_people::Ask::SetVisibility { project: u32, public: bool }`, appended last to
  `Ask` (`boundary.rs`), with its doc citing people.md 13.2; `Stored::Public { project: u32 }`
  appended last to `Stored`, doc: present while the project is public.
- The people child answers `SetVisibility` with the refusal it gives a request it does not
  take for that party, before any other check; every other match on `Ask` and `Stored` in
  `jig-domain-people` gains its arm, `SetVisibility` grouped with `RemoveProject` wherever the
  arm sorts requests by kind (a project's own act, by its owner) and `Stored::Public` with
  `Stored::Deletion` wherever rows are sorted by project.
- The store's people family, `schema/people-v14.schema` in place (as its recent changes are,
  e.g. commit `95f342a4aa`): `record AskSetVisibility { project: u32 public: bool }` and the
  `Ask` enum's `set_visibility` appended last; a row record for `Stored::Public` under a key of
  its own beside the deletion's (`keys/`), its key the project; the codec both ways; the
  generated code and goldens regenerated through the drift test (`jig/tests/codec-families.rs`,
  `SKEIN_CODEGEN_WRITE=1`). A store written before holds no such row and opens as before.
- The secrets' store: `jig_protocol_store::SecretOwner::PublicNames` (`boundary.rs`) with a
  key tag of its own, number 0, holding 32 bytes, kept, read and erased as the other owners'
  rows are; `secrets-v1.schema` in place only if the row needs a record the schema lacks.
- `jig/docs/design/protocol/store.md` is read by `src/tests/words.rs` (sections 2, 5.3, 5.4,
  6, 10 and 10.1): those words are unchanged by this plan; the test still passes.
- Step tests in `jig-protocol-store`'s `src/tests/` (where the people codec's and the
  secrets' are):
  - `set_visibility_round_trips_as_a_kept_ask`: both values of `public`, the largest project;
  - `a_public_row_round_trips_under_its_project_key`: written, read, erased; its key sorts
    with the project's other rows as the deletion's does;
  - `a_store_without_public_rows_reads_every_project_private`: a store written with the
    family's rows and no public row restores none;
  - `the_public_names_secret_is_kept_read_and_erased`: 32 bytes under `PublicNames`; a value
    of any other length refused as the other secrets' are.
- A step test in `jig-domain-people` (where its request tests are):
  `set_visibility_is_refused_until_it_is_decided`, by an owner, for project 0 and another.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, section 2.
- `jig/docs/design/domain/people.md`, 5.1's table (`SetVisibility`'s row), 13.2 and 13.8;
  `jig/docs/design/protocol/store.md`, 4.4 (the people family's row) and section 9 (the
  public identifiers' key).
- As they stand: `jig-domain-people/src/boundary.rs` (`Ask`, `Stored`), `domain/requests.rs`
  (`refused`), `domain/asks.rs`, `domain/applications.rs`, `domain/deletion.rs`;
  `jig-protocol-store`'s `schema/people-v14.schema`, `codec/people/asks.rs`, `keys/`,
  `boundary.rs` (`SecretOwner`), its secrets' codec and `src/tests/`.

## Owns and may change

- Owns the files in the header.
- **Compiler-forced arms** for the new `Ask` and `Stored` variants, each edited only for the
  arm (a refusal, the arm grouped with `RemoveProject`'s or `Deletion`'s, or nothing done for
  a `Public` row at start, which `visibility-core` loads):
  - jig: `jig-domain-core/src/acts/requests.rs`, `routing/party/people.rs`, the core's start
    files that sort restored rows, `jig-protocol-engine-clients/src/request.rs`,
    `translate/asks.rs` (a people ask with no wire form yet: the arm maps nothing and the
    report says where), `jig/testing/jig-conformance/src/core/naming.rs`,
    `jig/tests/people/src/world.rs`, `jig/tests/core/src/referee/offers.rs`,
    `jig/tests/core/tests/suite/deletion.rs`, and any other the compiler names in jig;
  - temper (a required arm, edited only for it): `temper/crates/temper-engine-domain/src/store.rs`
    (`record_bytes`), and any other the compiler names in temper.
- The `SecretOwner` of `jig-protocol-engine-clients` is not this task's (`public-component`
  adds its own); a service's map from the component's owners to the store's, if the compiler
  asks for an arm, maps nothing to `PublicNames` yet.
- Pre-approved: if `set_visibility` cannot be appended last because the schema's enums are
  ordered otherwise, follow the schema's order and say so. If a `Public` row needs a family
  tag rather than a key under the projects' tag, take the next free tag of the people family
  and say which.

## Steps

1. `Ask::SetVisibility` and `Stored::Public` in the people child, its refusal and arms;
   `heavy cargo check -p jig-domain-people --tests`.
2. The schema, codec, key and secrets' owner; regenerate; the forced arms in jig and temper
   until `heavy cargo check -p jig-domain-core -p jig-protocol-engine-clients -p temper-engine-domain --tests`
   passes.
3. The step tests; run them alone, then `heavy cargo nextest run -p jig-protocol-store --lib`
   and `-p jig-domain-people --lib`, and the drift test
   (`heavy cargo nextest run -p jig-store-codecs-world -E 'test(/drift::/)'`).
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; clippy on every package you changed; the
   focused tests of `jig-domain-people`, `jig-protocol-store`, `jig-domain-core`,
   `jig-protocol-engine-clients` and `temper-engine-domain`; the citations check.
5. Commit (why: a project's visibility as a request and a stored row, and the key's place,
   which the core and the component build on); push your branch.

## Report

As the implementer's text says, and also: the refusal `SetVisibility` gets until it is
decided, the row's key and tag, the secret's tag, and every forced arm by file.

## Answers

2026-11-03. **Q:** may the task change `jig-protocol-store`'s `src/intervals.rs`,
`src/patching/sets.rs`, `src/secrets.rs` and `src/secrets/**` for the public row's restore and
the public names' secret, which are not compiler-forced? **A:** Yes: the task owns those four
files, since its acceptance (a public row that restores, a secret kept, read and erased as
the other owners' are) cannot hold without them. The header's `src/codec/secrets*` names the
secrets' codec at `src/secrets.rs` and `src/secrets/**`. Edit them for this alone:
- `intervals.rs`: `RestoreRange::People`'s second range ends at `[JIG, 2, 11]`, so tag 10's
  `Public` rows restore; add a step test (or extend
  `a_store_without_public_rows_reads_every_project_private`'s neighbour) showing a written
  public row is restored.
- `patching/sets.rs`: the `jig` secrets' kept keys gain `[4, 0]` to `[4, 1]`, its root the
  record the row is encoded with (`Source`'s envelope as reported, holding the key value; no
  secrets schema change, as the spec allows).
- `secrets.rs`, `secrets/codec.rs`: `PublicNames` kept, read and erased at prefix `[4, 0]`,
  number 0 only, the value exactly 32 bytes, any other length or number refused as the other
  owners' invalid values are; replace the checkpoint's refusing arms.
Nothing else widens; the rest of the acceptance and steps stand.
