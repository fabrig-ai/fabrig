+++
id = "pg-jig-uses"
part = "playground"
kind = "interface"
needs = ["jp-acts"]
files = ["jig/crates/jig-domain-people/src/boundary.rs", "jig/crates/jig-domain-people/src/domain/acts.rs", "jig/crates/jig-domain-people/src/domain/start/fit.rs", "jig/crates/jig-domain-core/src/routing/deployment_acts.rs", "jig/crates/jig-protocol-config/src/acts.rs", "jig/crates/jig-protocol-config/src/tests/acts.rs", "jig/crates/jig-protocol-store/schema/people-v*.schema", "jig/crates/jig-protocol-store/src/codec/people/acts.rs", "jig/crates/jig-protocol-store/src/generated/people_v*.rs", "jig/tests/core/tests/suite/deployment_acts.rs"]
effort = "high"
minutes = 60
+++

## Goal

Give jig a deployment act that sets an account's uses, so temper's
playground can make its scripted account the deployment's with its uses
naming the playground's project alone (draft
`temper/docs/design/draft/playground-reset.md`, 6.1; jig's
`domain/accounts.md`, 4.2). jig has `AccountChange::SetUses` for a
holder's own ask, but no deployment act reaches it: `Act::AddKey` makes a
key account with `uses: None` (`jig-domain-accounts/src/keys.rs`, about
line 139) and no act changes them.

Acceptance:

- `people::Act::SetUses { account: u32, uses: Option<Box<[people::Use]>> }`
  and `ActKind::SetUses` exist, `SetUses` in `ActKind::ALL`; `engine.acts`
  reads the grant word `"set_uses"`.
- It is admitted as every deployment act is (grant, first owner, keyed
  and retained answer, saved answers before grant checks) and routed in
  `jig-domain-core/src/routing/deployment_acts.rs` through the existing
  `account(...)` helper as `people::AccountChange::SetUses { account, uses }`
  with `Holder::Deployment`: an account the deployment does not hold is
  refused unknown, as a person's `SetUses` on another's account is; its
  answer is what the holder's `SetUses` answers today.
- Its bounds: the uses' count and uniqueness as a holder's `SetUses`
  takes them (refused at admission with the same refusal the holder's ask
  gets; `fit.rs` bounds a restored act's uses the same way).
- It round-trips through the people codec as a retained act (a new
  people schema version, unless the trunk's newest people schema was made
  by this run's `jp-acts` and no store has been released since: then add
  `ActSetUses` to that version and say so).
- Tests in `jig-core-world`'s `suite` binary, `deployment_acts.rs`:
  `a_connector_sets_a_deployment_accounts_uses_to_one_project` (AddKey,
  then SetUses naming one project; the accounts document shows the uses),
  `set_uses_refuses_an_account_the_deployment_does_not_hold` (a person's
  account: unknown), `set_uses_without_its_grant_is_refused_not_offered`,
  and a replay answering its retained answer once; the codec round trip in
  `jig-protocol-store`'s library tests (the act case list, as `jp-acts`
  registers its acts); the grant word in `jig-protocol-config`'s
  `tests/acts.rs`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1),
  `temper/docs/plan/playground-reset-r5/notes/playground.md` (section 2,
  the step that uses this act).
- `jig/docs/design/domain/people.md`, 5.3 (the deployment's acts);
  `jig/docs/design/domain/accounts.md`, 4.2 (uses).
- `temper/docs/plan/playground-reset-r5/tasks/jp-acts.md`: the act added
  just before this one, the same path file by file; follow its shape.
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s3-playground-today.md`,
  section 4 (the act's path: `boundary.rs`, `domain/acts.rs`,
  `start/fit.rs`, `deployment_acts.rs`, `acts.rs` grants, the people
  codec).

## Owns and may change

- Owns, for the `SetUses` act alone (other tasks of this run edit the
  same files for other acts: touch only `SetUses`'s lines and the arms it
  forces): `jig-domain-people/src/boundary.rs` (`Act`, `ActKind`,
  `ActKind::ALL`), `domain/acts.rs` (`valid`, `kind`, subject checks),
  `domain/start/fit.rs` (`act`); `jig-domain-core/src/routing/deployment_acts.rs`;
  `jig-protocol-config/src/acts.rs` and its tests; `jig-protocol-store`'s
  people schema, generated code and `codec/people/acts.rs`;
  `jig/tests/core/tests/suite/deployment_acts.rs`.
- May change, compiler-forced arms only: every exhaustive match over
  `people::Act` and `people::ActKind` in jig's crates, `jig/testing/*`,
  `jig/tests/*`, `jig/examples/*`, and temper's (`temper-engine-domain`'s
  `store.rs` charges a deployment act's payload: charge `SetUses` as its
  uses' bytes, as `Adopt` charges its own).
- Options: if a holder's `SetUses` validation lives in a function the
  people domain can't reach (it is the core's, `account_requests::change`),
  bound the count at admission by the uses bound the people domain
  already holds for a holder's ask, and leave party/project existence to
  the core's path; if no such bound is in the people domain, add the
  accounts' `USES` constant's value as a people limit beside `words` and
  say so.

## Steps

1. Vocabulary and admission in `jig-domain-people`; the grant word.
2. Route in `deployment_acts::route`: `people::Act::SetUses { account,
   uses }` → `account(core, work, request, people::AccountChange::SetUses
   { account, uses })`.
3. Codec: `ActSetUses { account: u32 uses: option list N Use }` reusing the
   schema's existing `Use` form from the holder's `AccountChangeSetUses`;
   translations; round-trip case.
4. Tests above. Narrow runs: `heavy cargo nextest run -p jig-core-world
   --test suite -E 'test(/deployment_acts/)'`, `heavy cargo nextest run -p
   jig-protocol-store --lib`, `heavy cargo nextest run -p
   jig-protocol-config --lib`.
5. `cargo fmt`; `heavy cargo clippy -p jig-domain-people -p
   jig-domain-core -p jig-protocol-config -p jig-protocol-store
   --all-targets -- -D warnings`; `heavy cargo check -p
   temper-engine-domain --tests` and each other package whose arm you
   touched; the focused tests of `jig-domain-people`, `jig-people-world`,
   `jig-core-world`. Old-forms from `jig/`. Commit and push.

## Report

As `notes/common.md` says; name the schema version you used and every
file outside the owned list you edited for forced arms.
