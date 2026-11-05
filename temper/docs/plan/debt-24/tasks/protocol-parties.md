+++
id = "protocol-parties"
part = "front-identities"
kind = "feature"
needs = []
files = ["jig/tests/client-protocol/src/front.rs", "jig/tests/client-protocol/src/front/**", "jig/tests/client-protocol/src/engine.rs", "jig/tests/client-protocol/src/fixtures.rs", "jig/tests/client-protocol/tests/suite/shared/a.rs", "jig/tests/client/src/joint/role_fixture.rs", "jig/tests/client/src/joint/mod.rs", "jig/tests/client/src/stories/shared/a_roles.rs"]
effort = "high"
minutes = 60
+++

# The protocol front signs in several parties, and six stories run

## Goal

`jig/docs/development/debt/D029-protocol-front-one-identity.md`'s first half. Six shared role stories are ignored on the protocol front: its `Seed::Roles` is a no-op, and `front.rs`'s `bind` asserts the party is the first owner. They run after this, over real HTTP. **No production file changes:** the engine has one printed link, and `people::Event::Mint` already signs in any party, as temper and the Joint front do.

Acceptance:

- `rg -o '#\[ignore' jig/tests/client-protocol | wc -l` reads 0, and the six `roles_*` tests in `tests/suite/shared/a.rs` pass, with the bodies in `jig/tests/client/src/stories/shared/a_roles.rs` unchanged but `confirm_two_parties` (see Answers).
- The fixture's shape, inside which the choices are yours:

  ```rust
  // fixtures.rs: providers holds [0]; a party's identity is provider 0 and its subject
  pub fn identity(party: u64) -> people::Identity;

  // engine.rs: `Minted` no longer panics
  pub minted: Vec<(Token, Secret)>;
  pub fn mint(&mut self, exchange: Token, identity: people::Identity);

  // front.rs
  // Side gains the party it bound; `bind` records it, and its assert goes.
  // sign_in: party 1 signs in over the printed link, as today; any other party mints.
  // advance: a minted secret goes wire.restore(Some(secret)), then SignIn { With::Held(secret) }.
  // seed: Seed::Roles { story } rebuilds the front as the Joint front does.
  ```

- `joint/role_fixture.rs` gives both fronts one setup, so a role story's limits, peers and first rows aren't copied:

  ```rust
  pub fn setup(story: Story) -> (Config, Limits, Peers);
  pub fn install(driver: &mut Driver, story: Story, now: Time, wall: Wall);
  ```

- The role stories' receiving limits (`a_roles::limits()`) reach the front's `link::Limits` and its wire decoder (`wire.rs`'s default `link_limits`).
- One fixture test holds the seam: two parties signed in on one front have different party numbers in their `me`, each over HTTP. **It must bite:** make `identity` answer one subject for every party, watch it fail, and restore.
- `heavy cargo nextest run -p jig-client-protocol-world` and `-p jig-client-world` pass, and no other story's assertion changes. The report gives the six tests' times and the binary's before and after.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `jig/docs/development/debt/D029-protocol-front-one-identity.md`
- `jig/crates/jig-protocol-engine-clients/src/mint.rs`, and `boundary.rs`, `Event::Mint`
- `jig/tests/client/src/joint/component.rs`, its mint path
- `jig/crates/jig-client/src/client.rs`, `With::Held`

## Owns and may change

- **Owns:** the files in the header. `jig/tests/client-protocol/src/client.rs` and `client/accounts.rs` stay.
- **`settings.providers` becomes `[0]` in this fixture:** only `mint.rs` and `component.rs` read it, so nothing else moves. If another test's outcome changes, stop and ask.
- **A party other than 1 skips `/sign-in/link`,** and its later calls go over HTTP. That's the design of `Mint`, and stays.
- **The role documents may not fit this fixture's bounds.** A limit in `fixtures.rs` may rise to what the role fixture's own limits say. No limit outside this world moves; if one must, stop and ask.
- **`a_delegate_s_escalation_record_names_the_person_who_asked` isn't asked for.** Add it to `a.rs` if it passes as it is, one test; otherwise leave it and say so.

## Steps

1. Run one ignored test with `--run-ignored only` and read how it fails.
2. `fixtures.rs`, `engine.rs`, then `front.rs`; then `role_fixture.rs`'s two functions, with the Joint front on them.
3. The ignores go, one story at a time; the fixture test and its break.
4. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-client-protocol-world -p jig-client-world --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the first failure; each limit raised; the times. These six stories are newly held, so the lines they cost are named as such.

## Answers

### 2026-11-05: May the shared two-party story accept either winner, or must the protocol fixture complete the owner first? (debt-24/protocol-parties-1, foreman debt-24/protocol-parties/foreman-1)

Either winner, with both clients naming the same one: fabrig/debt, the plan's owner, settled it. That's the rule the story's own doc states ("see the same winning decision"); "party 1 wins" was the Joint front's schedule, and `joint/role_stories.rs`'s `two_parties_decide_one_proposal_and_the_second_is_told_who` keeps pinning it there, unchanged, through its stored `ProposalDecided` rows.

- **One body changes:** `confirm_two_parties` in `a_roles.rs`, added to the header. Each client presses Confirm and sees `Accepted` and a `Decided by person: ` text, as `decide` does with the number left out. Then, after `finish`, exactly one of persons 1 and 2 is named, and it's the same on both trees (`world.tree`, as `b_proposals.rs` reads it). `decide` and its other callers stay.
- **Nothing else moves:** no other assertion, and the fixture isn't retimed or ordered.
- **It must bite:** break the rule for real, not the comparison: make the second, repeated answer name the party asking instead of the recorded winner, watch this story go red on the protocol front, restore, and report it.
- **If another shared story holds a front's schedule,** that's the same case: stop and ask fabrig/debt.

Finish the rest on the pushed branch (`21d128213a`): the four other ignores, the seam test and its break, the optional delegate story, the checks and times above. `seed-projects`' run has found the other four stories pass on this front. `seed-projects` depends on this repair and won't make it.
