+++
id = "joint-cost"
part = "joint-cost"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/routing/children.rs",
  "jig/crates/jig-domain-core/src/routing/view/marks.rs",
  "jig/crates/jig-domain-core/src/documents.rs",
  "jig/crates/jig-domain-core/src/tests/documents/offer.rs",
  "jig/crates/jig-domain-core/src/tests/classify_subjects.rs",
]
effort = "high"
minutes = 180
+++

## Goal

jig's fuzzy suite went from 67.2 to 70.9 CPU s to 152.1 to 158.0 with `model-catalogue-1`'s landing, against a budget of 120. It's all in `jig-ops-world::suite fuzzy::joint`. Compare a profile from before that landing with one from your base, say the cause with figures, and fix the cause at its lowest place.

Acceptance:

1. **The cause is named from the comparison,** with the table under `## Report` filled in.
2. **`jig-ops-world`'s fuzzy tests read under 70 CPU s** on your builder, by `heavy cargo nextest run --profile fuzzy -p jig-ops-world`, and close to what they read at `e7bb41d364` on the same builder if the cause allows. 70 is derived: of 153 in the full batches, that binary's 17 fuzzy tests read 103, so the other 50 plus 70 is the 120. The trunk's end check reads the suite itself.
3. **No story does less.** `jig/tests/ops/tests/suite/fuzzy/joint.rs` and `jig/tests/ops/tests/suite/client_stories.rs` are unchanged: every check, seed, press and wait stays. A story may shrink only where what it stops doing is shown repeated work, with the figures showing it.
4. **`me` still moves when it should:** every invalidation in `jig/docs/design/domain/engine.md`, 11.1 still marks it, and the rows that hold this pass (below).
5. **If it can't get under 70 without changing what a story checks:** commit and push what holds, and make your report's first line a `Question:` carrying the measurements, for the user. Never move a test, raise a budget or skip one.

## Inputs

- `temper/docs/plan/ui-fuzzy-joint/inputs/request.md`, whole: its figures, its limits, and `For every task of this plan`, which binds you.
- `jig/docs/design/domain/engine.md`, 11.1: its `Marked` line says what must mark `me`.
- `jig/docs/design/client/domain/contract.md`, 7.2: what a delivery is, and that the engine keeps a mark and a window per watch, never a backlog.
- `jig/docs/design/client/domain/testing.md`, 5 and 9.
- `skein/docs/foundation/testing-strategy.md`, 8.1.

### What reading the code found (a lead, by reading: nothing here was run)

Treat this as the first thing to test, not as the answer.

- **Unchanged Frames may be delivered on every account event.** The landing made `Touch::Accounts` mark `Subject::Frame` (`documents.rs`, `Subject::touched_by`). `route_account` (`routing/children.rs`) calls `touch_accounts(party)` on every `accounts::Event`, and `account_party` is `Some` only for `SignIn` and `SignInFailed`, so nearly every event marks every watched Frame. Nothing compares: `watch::changed` (`jig/crates/jig-domain-views/src/watch.rs`) makes the document again, and it goes out whole. Fits: `fuzzy::client::*` didn't move, because the scripted engine never runs these touches.
- **A delivery is dear in this world.** `skein_world::stream::Wire::answer` moves 1 to 31 bytes at a 70% chance, about 11 bytes a round, and each round is one iteration of `Bytes::pump` (`jig/tests/ops/src/joint/bytes.rs`) with `Net::advance` in it. So pump cost follows bytes on the wire, and each delivered document also costs a `World::build`. That would tie the profile's two halves to one variable.
- **The request's two other suspects look unlikely in this binary.** `Bytes::new` takes `jig_engine_clients_kit::fixtures::settings`, not `Composition` (where the request saw `picker: true`), and runs no `jig_test_service::clients::Clients` (where `server_finished` is). Its own server is `jig_test_service::server::Server`. Rule them out with figures.
- **Others the comparison must rule in or out:** a bigger Frame (`offer` and `defaults` per project role); a dearer make (`Core::offer` and `preview_llm` per role, in `documents/small.rs`); a dearer view build (`ObjectsBorrowed::get` clones, in `tree::sections::inbox`); and a grown bound that something allocates by (`frame_offer_bound` in `document_with_context_bound`).

## Owns and may change

Owned: the `files` above.

The comparison picks the fix. Each option is pre-approved, and the report says why the figures pick it.

**A. Frames delivered more often, mostly unchanged (the default if the figures agree).** Mark a Frame on an account event only when that event moved it:

```rust
// documents.rs: marks `Subject::Accounts` as `Touch::Accounts` does, and never a Frame.
pub enum Touch { .., Accounts(Option<u64>), AccountLines(Option<u64>) }

// routing/view/marks.rs
impl Core {
    pub(crate) fn touch_account_lines(&mut self, party: Option<u64>);
    /// Each watched Frame's party and document, held only across one account's step.
    pub(crate) fn watched_frames(&self, env: &Env<Limits>) -> List<(u64, Result<Document, Missing>)>;
    /// Marks each watched Frame whose document differs from `before`, or can't be made on either side.
    pub(crate) fn touch_moved_frames(&mut self, env: &Env<Limits>, before: &List<(u64, Result<Document, Missing>)>);
}

// routing/children.rs, route_account
let before = core.watched_frames(env);
super::step::step_accounts(core, env, event, &mut out);
core.touch_deployment();
core.touch_run_waits();
core.touch_account_lines(party);
core.touch_moved_frames(env, &before);
```

- Frames come from `documents::make(core, &env.limits, party, subject, window, env.wall)`, as `routing/view/routes.rs` calls it: `small::frame` is private to `documents`, and it reads neither the window nor the wall, so the watch's own window serves. No snapshot outlives `route_account`, and no document is kept per watch (`contract.md`, 7.2). With no Frame watched, it computes nothing.
- `touch_accounts` and `Touch::Accounts` stay for the other callers, unchanged: `Timer::Fleet`, `Timer::Account` (an offer that time moves still needs its alarm), `route_fleet`'s host shape, `people::Event::Roles`, and `routing/party/holdings.rs`.
- If comparing two whole Frames per account event shows up in the profile at more than about 5% of the story, compare only `offer` and `defaults`, or classify `accounts::Event` in a match that lists every variant. Say which and why.

**B. A bigger Frame matters.** Make it smaller in `jig/crates/jig-protocol-engine-clients/src/translate/owner.rs` and `translate/llm.rs`, or in the wire schema. A protocol may change freely (the request); regenerate what it makes stale.

**C. A dearer make.** Remove repeated work in `documents/catalogue.rs` and `documents/small.rs`: coverage, host views and host facts computed again per model or per role. Offers and previews stay as they are; `tests/documents/offer.rs` holds them.

**D. A dearer view build.** Remove the repeated cloning in `jig/crates/jig-client-view/src/tree/sections.rs`, `tree/sections/`, or `state/ui/borrowed.rs` (where `ObjectsBorrowed` is), keeping what's drawn.

**E. A grown bound that something allocates by.** Allocate on use, not at the bound, in the crate that allocates. No bound is lowered to save time.

**F. Only after the cause is fixed, if the figure is still 70 or above, or far from the `e7bb41d364` reading:** repeated work in the harness that predates the landing, in `jig/tests/ops/src/joint/bytes.rs` and `jig/testing/jig-client-protocol-kit/src/net.rs` and `net/records.rs`. A seed must keep its schedule, and every record check stays. The report names this separately from the cause.

**Reader files, all options:**

- `classify_subjects.rs`: `marks` widens for `AccountLines`.
- `tests/documents/offer.rs`: as in the test sketch below.
- Any test whose oracle or helper the correct behaviour trips is yours, keeping what it checks for everything else. Name it in the report.

**A departure from your text:** one line of `docs/design/` that your fix makes false may be corrected, named in the report (the request allows it). Option A should need none: it is what 11.1's `Marked` line already says.

**Not to touch, whatever the figures say:**

- `skein/testing/skein-world/src/stream.rs`: the `Wire`'s schedule is every tree's seeded world, and it isn't what moved.
- `.config/nextest.toml`, `tools/tiers/budgets.toml`, and any `#[ignore]` or filter.
- `model-catalogue-2`'s work (below). Don't take it on.

**Overlap:** `model-catalogue-2` is changing `documents/catalogue.rs`, `documents/small.rs`, `jig-domain-core/src/lib.rs`, `runs/llm.rs` and `translate/llm.rs`. Option A touches none of them. Options B and C do, and whichever lands second catches up.

## Steps

**1. Measure, all on one builder.** Record `hostname` and `date -u`. Keep scratch untracked in the slot and remove it before pushing. Take these at two commits: `e7bb41d364` (`git switch --detach e7bb41d364` in your slot, with nothing uncommitted, then back) and your base.

- **The suite:** `heavy cargo nextest run --profile fuzzy -p jig-ops-world --status-level all`, for wall per test. For CPU, run the same build's binary as `/usr/bin/time -f '%U user %S sys' <suite binary> --test-threads=1 fuzzy::`, from `jig/tests/ops`.
- **The profile:** `valgrind --tool=callgrind --callgrind-out-file=<scratch> <suite binary> --exact fuzzy::joint::q_asks_for_an_environment_in_a_chat_and_accepts_what_it_costs`, read with `callgrind_annotate --inclusive=yes`.
- **The counts:** Frame makes and deliveries, the encoded size of a delivered `me`, and which `accounts::Event` variants reached `route_account`, from callgrind's call counts where they show and otherwise from a local counter you don't commit.

A departure from your text: these two binary runs build nothing, so send them through `heavy` if it takes the command. If it takes only cargo, run them directly, one at a time, with nothing else of yours running, and say so. If `valgrind` isn't on the builder, don't install it: compare by the counts, the CPU figures and `perf` if present, and say so.

**2. Name the cause** from the two columns, and pick the option above.

**3. Fix it, with a test that bites.** New cases are rows of existing tables:

- In `tests/documents/offer.rs`, `account_login_host_and_charter_changes_mark_me_with_unrelated_negatives` gains a negative: an account event that moves no offer or preview (for example `accounts::Event::Visited`, or whichever variant the fixture can send that moves nothing) leaves `marked(&core)` false. Its `Limited` positive stays, and must move the offer. If its account number doesn't name the fixture's key, make it name it, keeping the assertion.
- In `classify_subjects.rs`, `AccountLines` marks `Subject::Accounts` under the same party rule as `Accounts`, and no Frame.
- Break it both ways and see each fail for that reason: always mark (the negative fails), never mark (the `Limited` positive fails). The report names both failures.

Other options add a row of the same kind where the rule can fail, or say why the existing rows already hold it.

**4. Measure again** on the same builder: the suite's figures and the profile, at your tip.

**5. Run the readers.** This changes what a shared item means, so it runs every package that reads it. Through `heavy`, one command at a time:

- Focused: `cargo nextest run -p jig-domain-core -p jig-domain-views -p jig-core-world -p jig-ops-world -p jig-client-world -p jig-engine-clients-world -p jig-test-service -p temper-engine-domain-world -p temper-client-world`
- Fuzzy: `cargo nextest run --profile fuzzy -p jig-ops-world -p jig-core-world`

Fewer deliveries change the bytes a seed's schedule meets. A seed that now fails is a finding to fix, never a seed to change. Before calling a failure outside your change, run it once on the pristine base (the request).

**6. Close out** as your text says: `cargo fmt`, `old_forms.py`, `heavy cargo clippy` on every package you changed, commit and push. Your change's commit message carries the cause and the before and after figures.

## Report

Your text's report, and this table. A figure you couldn't get reads `unknown`, never a guess.

| Figure, on one builder (`hostname`) | `e7bb41d364` | base | tip |
|---|---|---|---|
| `jig-ops-world` fuzzy, CPU s | | | |
| `p_story_survives_seeded_commit_crashes_and_random_presses`, wall s | | | |
| `q_story_survives_seeded_commit_crashes_reload_and_random_presses`, wall s | | | |
| `q_asks_for_an_environment_in_a_chat_and_accepts_what_it_costs`, wall s | | | |
| That story's instructions, total | | | |
| `Bytes::pump`, inclusive, and its calls | | | |
| `Net::advance`, inclusive, and its calls | | | |
| `World::build`, inclusive, and its calls | | | |
| Instructions per `World::build` | | | |
| Frame makes, and Frame deliveries | | | |
| Bytes of a delivered `me` | | | |

Then: the cause in a sentence or two, with these figures; which suspects the figures ruled out; which option you took and why; whether real clients paid for it (the request asks, and if a Frame was going out unchanged they did); the bite failures; and any slow-tier story left to after-landing.
