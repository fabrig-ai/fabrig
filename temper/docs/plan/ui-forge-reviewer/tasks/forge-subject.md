+++
id = "forge-subject"
part = "reviewer"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/src/forge_user.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/forge_user.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs",
  "temper/crates/temper-engine-domain/src/route/kinds.rs",
  "temper/crates/temper-engine-domain/src/route/host_route.rs",
  "temper/docs/design/domain/forge.status.md",
  "temper/docs/design/protocol/forge.status.md",
  "temper/docs/design/protocol/store.status.md",
]
effort = "high"
minutes = 90
+++

# A review counts only from the forge it was made on

## Goal

Provider 0's subject doesn't name a forge, and `landing_reviewers` looks up `{ provider: 0, subject: review.author }` whichever forge the review is on. So in a project on two forges, GitHub's user 3 is taken for the party adoption seeded from Forgejo's user 3, role and all, and a stranger's approval can count toward a landing. Reproduce that first, failing; then make the subject name the forge, and put every reader behind one lookup by forge and user.

This task departs from the implementer's text in one way: three status entries are yours to remove (acceptance 7), because the design already says what this task builds.

Acceptance:

1. The branch's first commit holds the table test below, with the party at today's 8-byte key, and it fails on its stranger's row: the report quotes that failure.
2. `a_review_counts_only_from_the_forge_it_was_made_on` passes, with the party at `user_key(1, 7)`.
3. `human_forge_identity`'s rows pass (Steps, 3).
4. `user_key` and `key_user`'s rows pass, the exact 10 bytes among them.
5. Adoption seeds `user_key(the repository's forge, the collaborator)`: `stock_adoption_seeds_safe_human_approval_roles` passes, with its helper `collaborator_roles` reading the seeds through `key_user` and asserting forge 1.
6. In temper's production code, only `forge_user.rs` writes or reads the subject's form: no `to_be_bytes` subject and no `<[u8; 8]>::try_from` of a subject is left in `temper-engine-domain/src/route/`.
7. These entries are gone: `temper/docs/design/domain/forge.status.md`, **Not yet**, 4, 8.7, 8.9; `temper/docs/design/protocol/forge.status.md`, **Not yet**, 2.1, *Its users among jig's parties*; `temper/docs/design/protocol/store.status.md`, **Not yet**, 2, *Provider 0's subject*.
8. Focused tests pass in `temper-engine-domain-forge`, `temper-engine-domain`, `temper-engine-domain-world`, `temper-engine-smith`, `temper-client-world`, `temper-system-world`, `temper-conformance-world` and `temper-service`. This is a change of meaning, so these are its readers' tests, past what your own edits reach.

## Inputs

- `temper/docs/plan/ui-forge-reviewer/notes/contracts.md`, *The subject* and *The lookup*: they bind.
- `temper/docs/plan/ui-forge-reviewer/notes/common.md`.
- `temper/docs/design/domain/forge.md`, section 4 (*People the forge knows*, *Who a forge's user is*), 8.7 (*An approval*), 8.9 (*The root checks approval eligibility*).
- `temper/docs/design/protocol/store.md`, section 2 (*Provider 0's subject*).
- `temper/docs/design/draft/forge-identity.md`, section 3, for the reason only: don't build its link.

## Owns and may change

**Owned:** the `files` above, with `forge_user.rs` and its test new. The `mod` and `pub use` lines in the forge crate's `lib.rs` and `tests.rs` are yours.

**Readers, each edited only in the expression named.** Every site below builds or compares a provider-0 key from a `u64`; each becomes `temper_engine_domain_forge::user_key(1, N)` (or the file's alias of that crate), a comparison becoming `identity.key == user_key(1, N)`, and a decode going through `key_user`. Forge 1 is these worlds' forge (`fixtures::adoption(1)`, `forge_world::REPO`): where a story adopts on another number, use that number.

| Package | Files |
|---|---|
| `temper-engine-domain-world`, `src/` | `forge_routes.rs`, `direct.rs`, `walking.rs`, `walking_referee.rs`, `roles.rs`, `escalation.rs`, `escalation_referee.rs` |
| `temper-engine-domain-world`, `tests/suite/` | `root_routes.rs`, `settings.rs`, `settings/kinds.rs`, `watch_room.rs`, `trials.rs`, `panels.rs`, `forge_routes/paths.rs`, `forge_routes/approval_identity.rs`, `configuring.rs`, `configuring/fixture.rs`, `escalation.rs`, `applications.rs`, `making.rs` |
| `temper-client-world` | `src/front.rs`, `src/fixtures.rs`, `src/front/dismissed.rs`, `tests/suite/settings.rs`, `tests/suite/forges.rs`, `tests/suite/configuring_chat.rs` |
| `temper-system-world`, `src/` | `goals.rs`, `forge.rs`, `composed.rs`, `composed/chat.rs` (`parties::Act::SignIn`'s `subject` takes `user_key(1, 7).subject`) |
| `temper-conformance-world` | `src/lib.rs` |
| `temper-service` | `src/tests/fixtures/worker_peers.rs` |

Why they all change: in these worlds a person signed in on provider 0 is also a collaborator of the fake forge. Adoption's seed has to find that same party, or it makes a second one.

**Options, each with its default:**

- A fixture in `temper/crates/temper-engine-domain/src/tests.rs` or `src/tests/**`, whether its subject is opaque (`b"owner"`, `b"member"`) or a number (`tests.rs`, `restore_fit.rs`, `resume_fixture.rs`): leave it unless a test of yours fails on it; then change it, and say so.
- A world's `identity_bytes` too small for the 2 more bytes: raise that fixture's own figure by what's needed, and say so. It's a fixture's bound, not a budget.
- An inventory pin in a story whose project has no repository moves to `forge identity unknown`, which is now true: change the pin, and say so. With a repository, `forge user N` stays as it is until `inventory-forges`.
- A slow-tier story found reading the subject: change it, compile it, don't run it. The survey found none: `temper/tests/client-browser`, `temper/tests/real-loop`, `temper/tests/one-process` and `temper/crates/temper/tests` hold no provider-0 key and pin no inventory line.
- If something else in `temper/crates/**` proves to read the subject, it's a fourth reader: put it behind the lookup, and say so.

**Not yours:** `jig/**`, `temper/crates/temper-engine-smith/src/settings.rs`, the shape of `ProjectPerson` (`inventory-forges` changes it), and `temper/crates/temper-service/src/stock/configuring.txt`.

## Steps

1. **Reproduce, failing.** Add `forge_user.rs` (contracts) with its export. In `temper/crates/temper-engine-domain/src/route/forge_route/tests.rs`, add a fixture on the existing `two_forges()` (forges 1 and 3, each with repository 1, `adopted(number).project` being 5) that restores one party with a role in project 5, as `temper/crates/temper-engine-domain/src/tests/kinds.rs` restores its reader: `people::Domain::new`, `start_restore`, then `people::Event::Restore` of `Stored::Person`, `SignIn` and `Roles`. Add one table test; no table reaches `landing_reviewers` today, and the domain world's fake holds one forge, so this is where two forges can stand.

   ```rust
   #[test]
   fn a_review_counts_only_from_the_forge_it_was_made_on() {
       // The pull's forge, the review's author, and who it should be: the party is user 7 on forge 1.
       for (forge, author, counted) in [
           (1, 7, true),  // the person, on their own forge
           (3, 7, false), // a stranger with the same number
           (1, 8, false), // nobody's number
       ] {
           // landing_reviewers(&domain, &env, &row, &evidence): `row` is `change_row(None)` with
           // `repository: { forge, repository: 1 }`; `evidence.reviews` one `Approve` by `author`.
           // Counted: one reviewer, the party, and no unmapped. Else: no reviewer, and `unmapped`
           // names `account: author`.
       }
   }
   ```

   Give the party today's key, `{ provider: 0, subject: 7_u64.to_be_bytes() }`, run the test, and commit it failing on `(3, 7, false)`. Quote the failure in the report.

2. **The fix.** In `forge_route.rs`: `forge_person`, `person_forge_user` and `human_forge_identity` (contracts); `landing_reviewers` looks up `forge_person(domain, project, row.repository.forge, review.author)`, its writer skip first as now; adoption (`forge::Request::Adopted`) seeds `forge::user_key(adopted.repository.provider.forge, collaborator.user).subject`, under `forge::FORGE_PROVIDER`. In `route/kinds.rs`, `eligible_people` passes `member.person`, with no read of the identity. In `route/host_route.rs`, `settings_people` keeps its fields' shape and fills them through the lookup:

   ```rust
   let mut forge_user = None;
   let mut writer = false;
   for repository in &repositories {
       let forge = repository.provider.forge;
       if let Some(user) = super::forge_route::person_forge_user(domain, context.project, forge, member.person) {
           forge_user = Some(user);
           writer = domain.forge.is_writer(forge, user);
       }
   }
   ```

   Correct `ProjectPerson::forge_user`'s comment, and the one on `human_forge_identity`. Give the test's party `user_key(1, 7)`.

3. **The rest of the tests.**
   - `human_forge_identity`, a second table beside the first, in `forge_route/tests.rs`. The party's key, each forge's writer (restored as `forge::Stored::Writer`), and whether they count: `user_key(1, 7)`, with forge 3's writer being user 7: true, as a number on another forge is another account; `user_key(1, 7)`, with forge 1's writer being user 7: false; `user_key(2, 7)`, a forge the project isn't on: false; a key of provider 1: false.
   - `temper/crates/temper-engine-domain-forge/src/tests/forge_user.rs`, one table: `user_key(1, 7).subject` is `[0, 1, 0, 0, 0, 0, 0, 0, 0, 7]`; `(1, 7)`, `(3, 7)` and `(u16::MAX, u64::MAX)` come back through `key_user`; an 8-byte subject, an 11-byte one and a provider-1 key of the right length answer `None`.
   - The sweep in the table above. In `approval_identity.rs`, `writer_identity_is_ineligible` and `give_writer_legacy_membership` take `user_key(1, 1)`. `collaborator_roles` asserts forge 1 from `key_user` (acceptance 5).
   - New cases are these rows and no others: no new story in the domain world.

4. **The status entries** of acceptance 7.

5. Narrow tests after each change; then `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `python3 -I -B tools/citations/citations.py --base origin/main`, `heavy cargo clippy` on every package you changed, `--all-targets -- -D warnings`, and the focused tests of acceptance 8. Commit and push.

## Report

As the implementer's text says, and: the reproduction's failure as seen; each rule broken by hand for a new row and how it failed (for `forge_person`, have `landing_reviewers` look up forge 1 whichever the row's forge is); every option above you took; and each sweep site you left, with why.
