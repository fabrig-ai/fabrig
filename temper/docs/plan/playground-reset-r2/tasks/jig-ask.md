+++
id = "jig-ask"
part = "project-delete"
kind = "interface"
needs = ["jig-acts"]
files = [
  "jig/crates/jig-domain-people/src/**",
  "jig/crates/jig-domain-core/src/acts/**",
  "jig/crates/jig-domain-core/src/acts.rs",
  "jig/crates/jig-domain-core/src/documents/offers.rs",
  "jig/crates/jig-protocol-store/src/codec/people/asks.rs",
  "jig/crates/jig-protocol-engine-clients/src/**",
  "jig/crates/jig-client-domain/src/**",
  "jig/crates/jig-client-protocol-wire/**",
  "jig/crates/jig-client-protocol-link/src/**",
]
effort = "medium"
minutes = 80
+++

## Goal

Add the project owners' request `Ask::RemoveProject { project }` through
every layer a party's request crosses: people's request vocabulary, the
core's owner judgment and offered acts, the store's asks codec, the
engine's client protocol, and the client contract and wire, so that
`jp-delete` builds the deletion behind one request and the client part
shows Delete. The request is admitted for the project's owners and
refused *not offered* until `jp-delete`; it is offered (`ActKind::
RemoveProject` in `OwnerProject.acts`) to owners.

Acceptance:

- An owner's `RemoveProject` on a live project passes the owner judgment
  and is refused *not offered*, unsaved; a non-owner's is refused as
  lacking the role, as `Rename`'s is (`jig-core-world`'s owner-act cases,
  beside `Rename`'s).
- `OwnerProject.acts` lists `RemoveProject` for an owner and not for
  another party (the core's offers tests).
- The ask round-trips the store's asks codec, the engine-clients
  translation and the client wire (each crate's codec tests, wire schema
  changed in place with its goldens).
- Every package of jig and temper builds; tests pass unchanged in
  meaning.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 1.
- `jig/docs/design/domain/people.md`, 5.1 and 5.2 (a party's requests;
  the project itself).
- `jig/docs/design/client/domain/contract.md`, the asks and the owner's
  project document (find them by rg `OwnerProject`).
- `temper/docs/design/draft/playground-reset.md`, section 5.

## Owns and may change

Owns the files in the header for the request's vocabulary. The survey
of `Rename`'s path gives every reader; follow `Ask::Rename`'s arms
(paths relative to `jig/crates/`):

```text
jig-domain-people/src/domain.rs:770,823,951,1175,1335,1430,1554,1877,2153,2269,2310,2395
jig-domain-people/src/domain/start.rs:279
jig-domain-people/src/domain/start/fit.rs:192
jig-domain-people/src/models.rs:122,208,278
jig-domain-people/src/capacity.rs:72,108
jig-domain-people/src/batch.rs:227
jig-domain-people/src/words.rs:48
jig-domain-core/src/routing/people.rs:313
jig-domain-core/src/routing/party_controls.rs:211,286
jig-domain-core/src/roles.rs:138
jig-domain-core/src/person_task.rs:28,177,217
jig-domain-core/src/acts/requests.rs:120,169
jig-domain-core/src/acts/standing.rs:14
jig-domain-core/src/acts/objects.rs:45,89
jig-domain-core/src/acts/objects/task.rs:59,113,251
jig-domain-core/src/acts/objects/decisions.rs:79,118,183,245
jig-domain-core/src/documents/offers.rs:134,426
jig-protocol-store/src/codec/people/asks.rs:468,643
jig-protocol-engine-clients/src/request.rs:184,324,375,448
jig-protocol-engine-clients/src/translate/asks.rs:131
jig-protocol-engine-clients/src/translate/task.rs:77
jig-protocol-engine-clients/src/translate/llm.rs:217
jig-client-domain/src/requests.rs:274
jig-client-protocol-link/src/translate/asks.rs:698
jig-client-protocol-link/src/translate/store.rs:183
jig-client-protocol-link/src/translate/compound.rs:64,118
jig-client-protocol-link/src/translate/documents_part2.rs:69
jig-client/src/facts.rs:174
```

May change, match arms only, each doing for `RemoveProject` what it does
for `Rename` (an owner's project act) and drawing nothing new:

- `jig-client-view/src/{acts.rs,confirm.rs,confirmation.rs,cards/common.rs,activity/sending.rs}`,
  `jig-client-view/src/page/project/owner_form.rs`,
  `jig-client-view/src/sections/project/owner.rs` (the reference UI's
  Delete is `jp-`'s, later: here `RemoveProject` is unsupported by the
  owner form, as `Rename` is);
- the client test kits (`jig/testing/jig-client-test-kit/src/check.rs`,
  `jig/testing/jig-client-scripted-engine/src/{requests.rs,requests/projects.rs,payload.rs}`);
- temper's readers: `temper/crates/temper-engine-domain/src/store.rs:720`,
  `temper/crates/temper-client-tools/src/jig/render/{acts.rs,outcome.rs,document/admissions.rs}`,
  `temper/tools/gallery/src/worlds.rs` (an `ActKind` list), each an arm
  in the style of its `Rename` arm ("Delete project" for an offered
  act's words).
- `jig/crates/jig-domain-people/src/boundary.rs` and `jig-protocol-store`'s
  people schema: the `Ask` variant only (`jig-acts` owned them before;
  it has merged).

## Steps

1. `people::Ask::RemoveProject { project: u32 }` (boundary.rs), its
   project (`fn project(ask)`), its words, models, capacity and batch
   arms as `Rename`'s. The core: `acts/requests.rs` maps it to an owner's
   project act (`Act` in `jig-domain-core/src/acts.rs` gains
   `RemoveProject`), `standing.rs` requires `Role::Owner` on a live
   project, and routing refuses it *not offered*, unsaved, with a comment
   naming `jp-delete`.
2. Offered acts: `documents/offers.rs` lists `RemoveProject` among an
   owner's project acts. Client contract
   (`jig-client-domain/src/contract/`): `Ask::RemoveProject { project:
   u32 }` and `ActKind::RemoveProject`; the client wire's current schema
   (`jig-client-protocol-wire/schema/client-v19*.schema`) in place, its
   generated code and goldens regenerated; the link's translations both
   ways.
3. Tests: `jig-core-world` cases
   `an_owner_may_ask_to_remove_a_project_which_is_not_offered_yet`,
   `a_non_owner_asking_to_remove_a_project_lacks_the_role`,
   `an_owners_project_document_offers_remove_project`; the codec cases in
   `jig-protocol-store`, `jig-protocol-engine-clients` and
   `jig-client-protocol-wire`. Then, one at a time:
   `heavy cargo nextest run -p jig-domain-people -p jig-domain-core -p
   jig-protocol-store -p jig-protocol-engine-clients -p jig-client-domain
   -p jig-client-protocol-wire -p jig-client-protocol-link`;
   `heavy cargo nextest run -p jig-core-world -p jig-people-world -p
   jig-client-wire-world -p jig-engine-clients-world -p jig-client-world`;
   `heavy cargo check --tests -p jig-client-view -p temper-engine-domain
   -p temper-client-tools -p temper-client-view -p temper-gallery` (use
   the gallery's package name from its `Cargo.toml`) and the focused
   tests of each you edited; `cargo fmt`; clippy over every package you
   edited; old forms from `jig/` and `temper/`.
4. Commit and push.

## Report

The standard report, plus every match arm outside the owned files, and
whether a wire or store version moved.
