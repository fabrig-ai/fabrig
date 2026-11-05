+++
id = "setup-one-world"
part = "forge-setup-world"
kind = "feature"
needs = []
files = ["skein/tests/forge/setup/**", "skein/crates/skein-fake-forge-forgejo/src/server.rs", "skein/crates/skein-fake-forge-forgejo/src/service.rs", "skein/crates/skein-fake-forge-forgejo/src/setup/repo_hooks.rs", "skein/crates/skein-forge-setup/src/tests/**", "skein/docs/design/forge-setup.status.md", "skein/docs/development/debt/D031-forge-setup-two-worlds.md"]
effort = "high"
minutes = 150
+++

# skein: the setup world's stories all run through the host

## Goal

Pay `skein/docs/development/debt/D031-forge-setup-two-worlds.md`. The 77 tests on the plans-only `World` move onto
`HostWorld`, through the host component and HTTP bytes (`skein/docs/design/forge-setup.md`, 7), and `World` goes, with
`exchange.rs` and `decode.rs`, which only it reads.

Acceptance:

- `rg -w World skein/tests/forge/setup -g '*.rs'` finds nothing; `world.rs`, `exchange.rs` and `decode.rs` are gone.
- Every test keeps its name, seed and assertions, but for the exceptions below. `rg -c '#\[test\]' skein/tests/forge/setup`
  sums to what it does at your base, plus any step test you add outside that directory counted apart.
- The branch's net lines fall by at least 400 (this cut's survey: 754 out, about 215 to 275 in).
- The "Departs: 7" entry in `skein/docs/design/forge-setup.status.md` about two worlds is gone, and D031 is deleted.
- `skein-forge-setup-world`'s focused and fuzzy runs stay inside their budgets. Your report gives their times before and
  after. **Over budget is red** (`skein/docs/foundation/testing-strategy.md`, 8.1): no test moves to escape it.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `skein/docs/development/debt/D031-forge-setup-two-worlds.md`: this spec's counts are newer.
- `skein/docs/design/forge-setup.md`, 6.3 and 7
- `skein/docs/foundation/testing-strategy.md`, 8.1

## Owns and may change

- **Owns:** `skein/tests/forge/setup/**`, including its `Cargo.toml` (`skein-http` goes, `exchange.rs` being its one
  reader; `Cargo.lock` follows).
- **Additions only, in `skein-fake-forge-forgejo`:** a request log on `Server`, a refusal knob on `Service`, and public
  repository-hook methods on the setup store (below). No existing behaviour of the fake changes, and temper reads this crate.
- **`skein-forge`:** if the log can't see a request the fake refuses before decoding (a 401 returns from `entrance` first,
  and `reconcile.rs`'s story near line 146 needs that `Request::User`), one small addition to
  `skein_forge::forgejo::server::route`, as a way to hand the decoded head out, is approved. Say so in the report.
- **One step test, and one more if missing,** in `skein-forge-setup/src/tests/` (the exceptions below).
- **`skein/docs/design/forge-setup.status.md`:** that one entry.
- **If a story can't move without changing what it asserts,** beyond the exceptions here: stop and ask, naming the story.

**The two exceptions, decided by this cut:**

1. **`draws`.** `hooks.rs`'s four `draws(&world)` assertions (near lines 137, 150, 163, 174) go, with the helper: the host
   draws inside itself, and its `Need` is only `Read` or `Write` (`forge-setup.md`, 6.3). The rule they pin is a plan's, and
   stays pinned at the tier that can see it: `skein-forge-setup/src/tests/hooks.rs` (a kept secret, a missing secret) and
   `steps.rs` (asked once for many repositories) already do. No step test covers a reconcile with an off-loopback listener:
   add it, and break that rule by hand. Each story's other assertions stay.
2. **`secrets_absent`** moves as `HostWorld::secrets_absent(&self) -> bool` over what the host world hears (`needs`,
   `heard`, `said`), the scope GitHub's stories use. It no longer renders a plan's `Need::Send` and `Event::Reply`, so if no
   step test renders those for a secret, add one to `skein-forge-setup`, and break it by hand.

## Steps

1. `HostWorld` gains, in a new `skein/tests/forge/setup/src/plans.rs` so that `host.rs` doesn't grow past need (`Ended` moves
   there):

   ```rust
   impl HostWorld {
       pub fn owned(seed: u64) -> Self;
       pub fn managed_paged(seed: u64, cap: u32) -> Self;          // as `picker_world` in suite/host.rs builds it
       pub fn reconcile(&mut self) -> Ended;                       // and adopt, check, reachable, take_over, create:
                                                                   // World's signatures, each by `run_plan` + `finish`
       pub fn outcome(&self) -> Option<&Outcome>;
       pub fn tally(&self) -> &fake::Tally;                        // today's `requests()`, renamed at its ~16 call sites
       pub fn requests(&self) -> &[Request];                       // whole Request values, in order, as World's
       pub fn secrets_absent(&self) -> bool;
       // and thin, as World's: adopted, adopted_ids, listening, org_owner, service_word, forge_number, admin_scopes,
       // repository_hooks, creation_domain, domain_mut, with_hook, delete_repository_hook, refuse, permit
   }
   ```

   `World::managed(seed)` is `HostWorld::new(seed, &[])`; `with_config(seed, false, config)` is
   `HostWorld::with_config(seed, &[], config)`. `host.rs` hardcodes `seeded_config(seed, false, ..)`: `owned` needs that
   parameter, and `seeded_config` moves in from `world.rs`. Given changes go through the host's own `Request::Remove` and
   `Request::Add`.

2. `fake::Server::take_request(&mut self) -> Option<Request>`, beside `take_head`, read where `host.rs` drops the heads. A
   refusal knob on `Service`, checked once a request is decoded, so `world.refuse(route)` stays as stories write it. Hook
   seeding through public store methods, not over HTTP; hook stories never read the tally. `connected_picker` stories reach
   `ListingPermission`, and `HostWorld` panics there (`.expect("repository domain")`): `exchange.rs`'s stub moves into `tick`.

3. Rewrite `fixtures.rs`'s `crate::World` helpers on `HostWorld`, and its two "plan world" comments. `file_writes`,
   `secret_writes` and `Read(WebhookSecret)` read `needs()` and `reads()`.

4. Move the suites, one commit each where you can: `reconcile`, `access`, `take_over`, `check`, `create`, `hooks`,
   `reachable`, `fuzzy/reconcile`. Then delete `world.rs`, `exchange.rs` and `decode.rs`. Read a failure before you change
   anything: an assertion that fails is either a gap in your `HostWorld` method or a question.

5. Measure: `heavy cargo nextest run -p skein-forge-setup-world` and its `--profile fuzzy` run, at your base first and at
   the end. `fuzzy/reconcile.rs` (32 seeds, two reconciles each, now through the full host) is the likely cost.

6. Delete the item and the status entry. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-forge-setup-world -p skein-fake-forge-forgejo -p skein-forge-setup --all-targets -- -D warnings` (and `-p skein-forge`
   if you changed it); `heavy cargo nextest run -p skein-forge-setup-world -p skein-fake-forge-forgejo -p skein-forge-setup -p
   skein-forge`; `heavy cargo check --tests -p skein-fake-peers -p temper-fake -p temper-service`: **this departs from the
   implementer's text:** the fake's change is additions only, so its dependents are checked, not run. The citations check.
   Commit and push.

## Report

The common note's items; the two step tests and what you broke; both budgets' times before and after; every `HostWorld`
method you found you didn't need, and any story whose moving surprised you.
