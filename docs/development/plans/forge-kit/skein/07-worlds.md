# Session 07: the client's and setup's worlds

Move the client's and reconcile's worlds into skein, preserving their
referees, allocator checks and sweeps; then add the missing client recovery.
The additional setup stories (07.4) move to the GitHub draft, section 17 (ruling 189). Each of the two destinations is a hand-listed workspace
member; the component's worlds belong to skein 08 and 09 (decisions 22, 23).

Needs at start: temper 02.1 (the client world uses skein's client) and
temper 05.1 (the worlds use skein's fakes), landed on origin/main.
07.1 also needs skein 02.2; 07.3 temper 04.1 and skein 08.1 (README's sessions table).

## Read first

- skein's `docs/design/forge-client.md` 7.5, 8, 10;
  `forge-setup.md` 4, 7; `fake-forge.md` 8.
- skein's `docs/foundation/programming-model.md` 3, 6.3, 9, 10;
  `testing-strategy.md` 2.2, 2.4, 2.5, 6, 7, 8, 10.
- This plan's README (sessions, Rules, Done when 5),
  `code-organization.md` 1, 3, 5–8, `decisions.md` 2, 3, 9, 11, 14,
  18, 19, 22, 23 and Design questions 14, 22, 23, 25.
- `findings/survey-06-tests.md`, `trace-client.md` 2, 4, 5,
  `trace-setup.md` 1–4, `base-runs.md`, and `base-skein-07.md`
  (commands, names and the closing review corrections).

## What the design asks

- forge-client 10: a durable scripted parent, independent referee,
  restart and held-write stories, and every table full against `worst_case`.
- forge-setup 7: reconcile through HTTP bytes and an item store, repairs,
  paging, independent owners, no secret in diagnostics, and a seeded sweep.
- fake-forge 8 and testing-strategy 8: keep names, coverage and cost;
  unoptimized builds, focused tests below 0.5 s and fuzzy tests about 5 s.

## Choices this plan makes

Copy from temper's origin/main at each move's start, naming that commit
in the landing body. Files and counts below describe the base, not the
future source commit. Preserve module names and test names. Count each
file pair by code-organization 6; Size includes moved fixture/test lines
for review, and adds no shipped production implementation here.

Execute 07.1 (client move), 07.2 (recovery), then 07.3 (setup move). Client work comes before the setup wait on skein
08.1's shared machine support. New world behavior lands after its move.
The copied 858-line simulation file is split by 07.2, not 07.1.

## Rules that bind it

- No rewrite of a moved part. Only import paths, owner values and
  downstream configuration types change in a move, as each Shape lists.
- No `temper` in skein's code, tests or comments, no alias or re-export
  of a moved type; no serde. Values are `kit`, `kit/0.0.0`, `b"kit"`,
  `kit-admin`, `kit:bot`, `kit-api`, `kit-git` (code-organization 5).
- A moved test keeps its name, budget and time. The allocator test stays
  in its own binary. No filtering or optimization to hide a slow test.

## Increments

### 07.1 Move the client's world and its memory proof
Scope: forge-client 10's existing domain world, including the allocator
  test; the recovery additions are 07.2.
Needs: the session's start capabilities and skein 02.2's byte measure;
  `client::worst_case` is unchanged (Design question 14, answered).
Files: at the base, temper `tests/engine/forge/client/{Cargo.toml,
  src/{lib,world,translate,referee}.rs,tests/{simulation,memory,locate,
  job_answer,fuzzy_simulation}.rs}` -> skein `tests/forge/client/`.
  Change skein's shared `Cargo.toml` (member and workspace dependency),
  `Cargo.lock`; package `skein-forge-client-world`.
Shape: set `license.workspace = true` in the destination Cargo.toml's `[package]`,
  beside its other workspace-inherited fields (README, Workspace license inheritance).
  Keep the public `World`, `Settings`, `Stats`, `translate`, `LIMITS`,
  `REPO` and `World::{new,restart,make}`, with their signatures.
  `world` and `referee` stay private modules; `referee::Referee` stays
  `pub(crate)`. The world types and constants remain at the crate root.
  Client vocabulary imports become `skein_forge::api`; client, fake and
  world imports use their skein crates. Fixture branch names `temper/7`
  become `kit/7`; cite forge-client instead of the downstream document.
  `translate::KEY` and its golden marker become `<!-- kit:key `;
  `world::configured` uses namespace `b"kit"`. The five `worldold` key
  fixtures become version 1, length 3, `kitold`; sizes follow the word.
  The same frame correction reaches `tests/simulation.rs::{issue,post}` and
  `tests/fuzzy_simulation.rs::seeded_faults_restarts_and_keyed_effects`: use
  version 1, length 3, `kit`, preserving each identity suffix and assertion.
  Keep the independent marker encoder/decoder, not the code under test's.
  No change to restart's sequence, outcome handling or referee here.
Test shape: 29 focused tests (25 simulation, 2 locate, 1 job_answer,
  1 memory) and 2 fuzzy, at the base; all retain their names.
  Reuse crate-root `skein_forge_client_world::LIMITS`,
  `Settings::{calm,random}` and `translate::commit`; the driver keeps its
  crate-private `referee::Referee`. The small world profile stays unchanged;
  it is the kit-world profile, not a new copy of an application's limits.
  Keep `full_live_caches_inbox_stamps_echoes_outbox_and_every_entry_point_fit`:
  `heap::Counting`, `Meter`, `Measured`, `restore_participants`, both limit
  rows, each entry point checked against `client::worst_case`. Its floor
  is zero overrun; parent-owned outputs are released before the check.
  No new test or new signature; skein 02.2 owns the fixed byte measure
  of forge.md 3.7 (decisions.md, Design questions 14, answered).
Proof:
  world; `cargo nextest run -p skein-forge-client-world` shows
  29 passing tests, including memory; baseline 0.075 s, repeat 0.020 s.
  world; `cargo nextest run -p skein-forge-client-world --profile fuzzy`
  shows 2 passing tests; baseline 0.258 s, repeat 0.169 s.
  These prove the copied harness itself; the GitHub draft, section 17 (ruling 189), plans to connect the client
  to the component in a separate joint world.
Don't: replace the fake with scripted client answers; inspect client state
  in the referee; split simulation on this move; alter any client behavior.
Size: about 2,800 lines moved, 80 new.
Removes: nothing in temper; its deletion and translation-path user are
  temper 07's. Nothing besides obsolete import paths in skein.

### 07.2 Complete the scripted parent's recovery
Scope: forge-client 8, 10 and 7.5; full restart and Held then Remake,
  after the world has moved. No client production change.
Needs: 07.1's client world and allocator proof.
Files: `tests/forge/client/src/world.rs`, new `src/world/recovery.rs`;
  `tests/simulation.rs`, new `tests/recovery.rs`, new
  `tests/fixtures/mod.rs`. Move the two existing uncertain-branch restart
  stories and their entry construction into recovery/fixtures, preserving
  test names; this takes simulation below 800 lines before editing it.
  The moved tests are `an_uncertain_branch_is_found_without_retry_after_restart`
  and `a_deleted_uncertain_branch_is_held_after_its_deadline` (2 at the base).
Shape: keep `World::restart`; retain the parent's writer claims and
  held entries separately from client execution copies. Construct the
  configured client, Restore rows, Restored, replay Writer, PauseOutbox,
  Make only entries not held, ReadAfresh; on its one completion send
  SettleOutbox, recording its one completion. Commit a whole decision
  before releasing calls, as `take_client` already does.
  Parent-only tables: `writers: List<Resource>` bounded by `LIMITS.resources`,
  `held: Map<u64, Entry>` bounded by `LIMITS.entries`; completion counters
  are `u32`. They are not client state or additional production API.
  New owner-side methods; compile with source types in a skein-base
  scratch worktree, then with the moved types at the ready check:
```rust
// tests/forge/client/src/world/recovery.rs; impl World
/// Record a durable branch claim and tell the client; ends on taken=false.
pub fn writer(&mut self, resource: Resource, taken: bool);
/// Hand a held entry back on the person's choice, preserving its identity.
pub fn remake(&mut self, entry: Entry);
/// The parent's observed `ReadAfreshDone` and `OutboxDone` totals.
#[must_use]
pub fn recovery_completions(&self) -> (u32, u32);
```
Test shape: reuse `Settings::calm`, `LIMITS`, `REPO`, `translate::commit`
  and extracted branch-entry fixtures; no client-state reads. Add exactly
  2 focused tests in `tests/recovery.rs`:
  `restart_reads_afresh_before_settling_the_outbox`: one live branch and
  attempted write, durable Writer claim, restart; the branch watch is
  `participating: false` (participation admits only Issue/Pull resources).
  Keep completions `(1, 1)`,
  the saved attempt unchanged, no write while paused, one creation total.
  `a_held_branch_is_written_again_only_after_remake`: seed 50, existing
  late-copy/delete scenario; hold beyond its deadline with writes `1`,
  then explicit Remake, writes `2`, branch `Some(head)` and one new Made.
  Reference for the new file (the fixture scripts the existing Held story):
```rust
#[test]
fn a_held_branch_is_written_again_only_after_remake() {
    let (mut world, entry, head) = fixtures::held_branch();
    world.run_for(30);
    assert_eq!(world.stats().writes, 1);
    world.remake(entry);
    world.run_for(30);
    assert_eq!(world.stats().writes, 2);
    assert_eq!(world.branch(b"kit/7"), Some(head));
}
```
  Both are below 0.5 s; no fuzzy seed or existing assertion is removed.
  The recovery module holds the moved restart body, so world.rs also
  remains below 800 lines. Memory's named test stays unchanged.
Proof:
  world; `cargo nextest run -p skein-forge-client-world` shows
  31 passing tests; base form shows 29, 0.020 s; aim below 1 s total.
  world; `cargo nextest run -p skein-forge-client-world --profile fuzzy`
  shows the same 2 tests, baseline 0.258 s; each below 5 s.
  Parent recovery is this domain harness's behavior under test; the
  added client/component joint is deferred to the GitHub draft, section 17 (ruling 189).
Don't: replay held entries through Make; refresh a saved attempt on a
  find; make Remake implicit; fix production behavior in this harness.
Size: about 180 lines moved, 180 new.
Removes: the partial restart path and duplicated recovery fixture setup.

### 07.3 Move reconcile's world over given values and items
Scope: forge-setup 7's existing HTTP world and fuzzy sweep.
Needs: temper 04.1 (the source world calls skein's reconcile), and
  skein 08.1's `skein-forge-machine-world`, exporting
  `skein_forge_machine_world::pipe::{Pipe,Cut}` and
  `skein_forge_machine_world::fixtures::fake_limits`.
Files: at the base, `tests/forge-setup/reconcile/{Cargo.toml,
  src/{lib,world,fixtures,exchange}.rs,tests/{reconcile,fuzzy_reconcile}.rs}`
  -> `tests/forge/setup/`; `tests/codecs/src/forgejo.rs`'s `Decoded`,
  `decode_answer`, `drive` -> new `tests/forge/setup/src/decode.rs`.
  Change skein `Cargo.toml` (hand-listed member), `Cargo.lock`;
  package `skein-forge-setup-world`. Its Cargo dependencies are skein-lib,
  skein-http, skein-json, skein-forge, skein-forge-setup,
  skein-fake-forge-protocol and skein-forge-machine-world; the last is
  08.1's workspace dependency at `tests/forge/machine`. Drop the source's
  jig-protocol-config and temper-codecs-world dependencies.
Shape: set `license.workspace = true` in the destination Cargo.toml's `[package]`,
  beside its other workspace-inherited fields (README, Workspace license inheritance).
  Keep `World::{managed,owned,reconcile,missing_org,listening,put,
  file,store,store_mut,tally,requests,trace,secrets_absent}`, `Ended`,
  `exchange::send`. Relative to the base, temper 04.1 adapts the world to
  skein 04.1's `given::Given`, `given::Refused` and `item::Item`; copy those
  landed adaptations, with no second Given type:
  replace `entry: Entry` and the separate listener field with `given: Given`;
  `World::listening` sets `given.listener`, `missing_org` sets `given.orgs`.
  Replace `Reconcile::new(&self.entry, self.listener)` with
  `Reconcile::new(&self.given)`, whose `Result<Self, Refused>` is checked
  with `expect` for valid fixture values before `Begin`.
  Store keys and `file`/`put` use `Item`; the driver takes `Need::Read(item)`
  and `Need::Write { item, bytes }` instead of `place::File` and `file`.
  `fixtures::seed_token` also takes `skein_forge_setup::item::Item` instead
  of `temper_forge_setup::place::File`, retaining its existing seeded rows.
  In skein, `fixtures::given` constructs Given directly, dropping
  Setup/JSON/mode parsing. Its `owned: bool` selects `org_owner`:
  `Some(b"ada".as_slice().into())` when owned, `None` when managed;
  no independent make flag (Design question 23, answered).
  In world.rs, fixtures.rs and the moved tests, rename the owner's values:
  `temper` -> `kit` for `word` and `bot_login`, `temper-admin` -> `kit-admin`
  for `admin_name`, `temper-api` -> `kit-api` for `api_name`, and
  `temper-git` -> `kit-git` for `git_name`; `bot_email` becomes
  `kit@example.invalid`. Seeded/expected `temper:bot` becomes `kit:bot`.
  Hand the names literally through 04.1's Given fields; do not derive them.
  exchange.rs imports 08.1's `pipe::{Pipe,Cut}`; world.rs and fixtures.rs
  call its `fixtures::fake_limits`. lib.rs declares private `decode`;
  exchange.rs calls `crate::decode::decode_answer`. decode.rs imports
  `skein_json::tokenizer`, skein-lib and `skein_forge::forgejo`.
  `decode_answer` and the tokenizer driver retain their signatures and
  bounded execution. The private `Decoded` copy keeps `rows` and `document`;
  omit its unread `steps` field and discard the driver's returned count.
  The source codecs world's `Decoded.steps` and its fuzzy bound assertion
  stay in temper. Add no lint exception or setup assertion for the omitted
  field. Other helpers are plain moves, with skein-forge imports.
Test shape: 21 focused + 1 fuzzy at the base, every name retained.
  Reuse `fixtures::{config,random,bot_tokens,assert_bot,assert_org,
  assert_other_hooks,write_routes,file_writes}`, `exchange::send` and
  the complete decoder helper set. Fake limits keep their source values;
  `config.max_response_items=2` is the existing documented override of
  the Forgejo cap 50, to make three rows require another page.
  No new tests or independent configuration profile in this move.
Proof:
  joint; `cargo nextest run -p skein-forge-setup-world` shows
  21 passing tests; baseline 0.122 s, repeat 0.075 s, each below 0.5 s.
  joint; `cargo nextest run -p skein-forge-setup-world --profile fuzzy`
  shows 1 passing test; baseline 1.315 s, repeat 0.744 s, below 5 s.
  This is the real reconcile/HTTP/codec/fake joint; a real Forgejo host
  remains the owner's process tier, outside this harness move.
Don't: copy Pipe or Cut; retain configuration parsing or filesystem paths;
  take the whole codecs module (it includes unrelated serde helpers).
Size: about 950 lines moved, 120 new.
Removes: downstream configuration parsing/dependencies in the skein copy;
  nothing in temper.

### 07.4 Add paging and independent-owner setup stories — dropped

Dropped by the scope cut, ruling 189 (2026-10-20). Moved to the GitHub draft, section 17 (ruling 189).
No implementation or proof of this increment runs in forge-kit; its earlier specification remains
in git history. Every retained move and its inherited coverage still runs.

## Done when

From skein, after retained increments 07.1–07.3 (07.4 is dropped):

- `cargo nextest run -p skein-forge-client-world` prints 31 passed;
  the same command with `--profile fuzzy` prints 2 passed.
- `cargo nextest run -p skein-forge-setup-world` prints the 21 moved tests passed;
  the same command with `--profile fuzzy` prints 1 passed.
- `rg -n '^    "tests/forge/(client|setup)",$' Cargo.toml` prints
  two member lines.
- `rg -n -i 'temper' tests/forge/client tests/forge/setup`
  prints nothing (rg exit 1). This session's portion of README Done when 5;
  its source-form search at the base finds the names the move removes.
