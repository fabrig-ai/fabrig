+++
id = "protocol-parties"
part = "front-identities"
kind = "feature"
needs = []
files = ["jig/tests/client-protocol/src/front.rs", "jig/tests/client-protocol/src/front/**", "jig/tests/client-protocol/src/engine.rs", "jig/tests/client-protocol/src/fixtures.rs", "jig/tests/client-protocol/tests/suite/shared/a.rs", "jig/tests/client/src/joint/role_fixture.rs", "jig/tests/client/src/joint/mod.rs"]
effort = "high"
minutes = 60
+++

# The protocol front signs in several parties, and six stories run

## Goal

`jig/docs/development/debt/D029-protocol-front-one-identity.md`'s first half. Six shared role stories are ignored on the protocol front: its `Seed::Roles` is a no-op, and `front.rs`'s `bind` asserts the party is the first owner. They run after this, over real HTTP. **No production file changes:** the engine has one printed link, and `people::Event::Mint` already signs in any party, as temper and the Joint front do.

Acceptance:

- `rg -o '#\[ignore' jig/tests/client-protocol | wc -l` reads 0, and the six `roles_*` tests in `tests/suite/shared/a.rs` pass, with the bodies in `jig/tests/client/src/stories/shared/a_roles.rs` unchanged.
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
