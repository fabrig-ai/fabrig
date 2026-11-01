# Where the code stands, against the draft's section 2

The run's status (the brief's step 1), 2026-10-31, at `c30cf6a03` (the
inputs' commit `d372bfcf6`). The facts are the surveys beside it, in
`notes/surveys/` (read-only Codex runs over this checkout); each claim
below names its survey and section. The draft is
`temper/docs/design/draft/playground-reset.md`.

## 1. Forges in the document (draft 2, 3)

- **Forges are `application.forges[]`, `identities[]` and `setup[]`,**
  parsed by `temper-service`'s `startup/application.rs` (`forge`, `identity`,
  `resolve`) into `settings::Forge` and `settings::Identity`, and by
  `temper-forge-setup`'s `Setup::read` into `Entry { forge, directory,
  bot, orgs, mode, asked }` (s1, 1). Nothing allocates a forge number;
  no forge record exists in the store (s1, 6).
- **The engine is given them at construction:** `temper-engine-domain`'s
  `Config.forges: Box<[forge::Configured]>`, then
  `temper_engine_domain_forge::Domain::new(.., forges, under_box)`, whose
  `configured` map is "never restored from records" (s1, 3). Every
  per-forge table (`connections`, `first`, `ended`, `apps`, `writers`,
  `identities`) is keyed by the forge's number.
- **skein's forge layers are fixed at construction too:**
  `skein-forge-connection`'s `Component::new(settings, ..)` holds
  `Settings.forges`, and `skein-forge-setup`'s host its `host::Settings`;
  no add or remove exists (s1, 3; `notes/surveys/s6-skein-forges.md`).
- **Limits:** no `forge.forges`; the connection component's
  `forge_component.forges` (default 4), and several per-forge tables
  sized by `forge.client.repositories` (s1, 5). `memory_bytes` is a
  runtime budget since 272c945cc.
- **The first repository's intent** is in memory twice: the domain's
  `first: Map<u16, First>` and the service's `setup/lifecycle.rs`
  `Intent`; a test pins it volatile
  (`qa_first_repository.rs`, `first_repository_row_is_volatile_across_restart`) (s1, 4).
- **`temper init`** writes forge 1 (the demo's fake), 2 (GitHub) and 3
  (owned Forgejo, asked), their setup entries, `application.demo` and
  the demo provider (s1, 2).

## 2. The playground today (draft 2, 6)

- **The demo** is `Ask::StartDemo`, the `Demo` row (family 17, `{forge,
  state: Running | Ended}`, no project field), `DemoConfig`, and the end's
  kept acts, `DemoAct` (family 19: `Close { project, task }`,
  `RemoveAccount { account }`), sent again at each start
  (`temper-engine-domain-forge/src/demo.rs`,
  `temper-engine-domain/src/route/demo.rs`) (s2, 1-2). Another
  configured forge's connect ends it for good.
- **The playground's project is the seeded project 1**
  (`application.demo.project` is a number in the code, though the
  design says `{name, policy}`); runtime projects exist for the first
  repository (`MakeProject`), not for the playground (s5).
- **The client** judges the model's and the forge's halves apart
  (`temper-client-domain/src/forge/demo.rs`, `Halves`); the welcome,
  banner and setup strip are in `temper-client-view` (s2, 4).
- **The box** builds its groups from the document at its start
  (`temper-fake::steps`, `temper-forge-setup::steps`); its pipe knows
  `start` and `stop` (`temper-box/src/pipe.rs`), its record
  `box/started/N`; there is no `box/groups/N` and no `remove` (s4, 1).

## 3. jig and deleting a project (draft 5)

- **Not there:** an act that deletes a project or a task; horizon
  retention and a task's erasure whole; a project's adoptions released
  to its connectors; settings by project scope (jig's settings keys
  write scope 0 alone); a durable multi-step deletion (s3, 1-6).
- **There:** `MakeProject` (party request and deployment act,
  `jig-domain-core/src/projects.rs`); the deployment's acts with their
  grants (`ActKind`, `engine.acts`); `Close`'s compound cancel
  (`routing/deployment_acts.rs`); forgotten-once-read erasure (s3, 2-5);
  jig's conformance world and referee (s3, 7).

## 4. Earlier work

- `playground-apart.md` was never on main; the draft says what it held.
- On main since 10-20: the demo's start and end, the dormant owned
  forge, credentials in records, the welcome's fake backends, runtime
  projects for the first repository (from the surveys' git history).
  None of it builds forges as records, removal or project deletion.
- No branch or note before the replayed run holds this draft's work.

## 5. What the run decides (binding: `notes/contracts.md`)

The cut, its parts and its order are in `plan.toml`; the shapes that
cross parts, the store's new families, the document's machine members,
the box's group files and pipe lines and jig's new acts, are in
`notes/contracts.md`.
