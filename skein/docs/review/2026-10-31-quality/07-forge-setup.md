The setup crate is sound in design but uneven in execution. It is not bloated at 17.6k, though about 1.5k lines could go, mostly in tests. Its host also re-implements the TLS→HTTP→JSON connection that skein-forge-connection already has.

## 1. Scope read

Tree: `skein (at 3371027)`

- **Read closely:** in `crates/skein-forge-setup`, `reconcile.rs` with `reconcile/tokens.rs` and `orgs.rs`, `access.rs`, `call.rs`, `limits.rs`, `github/{app,write,convert,paste,reconcile,hook,limits,given}.rs`, `host/{run,settings,connection}.rs` (first 420 lines and the helpers), `host.rs` (first 200 lines), the head of `reachable.rs`, `src/tests/{fixtures,github_fixtures}.rs`, and the helper headers of every `src/tests/github_*.rs`. In the setup world, `src/{world,exchange,fixtures}.rs` and `tests/suite/reconcile.rs`. In the conformance tool, `lib.rs`, `main.rs`, `cases.rs`, `tokens.rs`, `write.rs`, `mask.rs` and `json.rs`. Also `skein-forge-conformance/src/{lib,facts,json}.rs` and the throwaway's `lib.rs`, `http.rs` and `instance.rs`. To compare, I read parts of skein-forge-connection's `connection.rs`, `settings.rs` and `limits.rs`.
- **Skimmed:** `create.rs`, `item.rs`, `host/limits.rs`, `boundary.rs`, `src/tests/{host,github_host,steps}.rs`, the world's `HostWorld`, the conformance's `facts.rs`, `deliveries.rs` and `tests.rs`.
- **Not read:** the tests for manifest, memory and states, the throwaway's CLI and its tests, the conformance's component, runner and turn modules.

**Split:** about 7.0k lines of production code against 10.6k of tests in the crate (60% tests). Production is roughly proportionate for 12 plans plus a host. Roughly 0.7k of production and 1k+ of tests could go.

## 2. Scores

1. **Cleanliness: 3.** It reads clean, but has dead guards, refusal arms the host can never reach, redundant `#[path]` attributes and explicit `drop()` calls.
2. **Organization: 2.** The shared plan vocabulary lives in Forgejo's `reconcile` module, Reachable mixes both dialects, and tests are mounted from three parent modules.
3. **Taste: 3.** The GitHub plans are crafted well. Halting works by feeding a plan a wrong event, and `closing: Option<bool>` stands in for a state.
4. **Naming: 3.** Test names are excellent. Type names collide (`Need`, `Event`, `Limits`, `Given`, `Refused`, `Place`), and failure subjects drift from the design.
5. **Primitives: 2.** `Reply` mixes the two dialects, the connection pump is duplicated, `Plan` does not encode its dialect, and an item is validated in three layers.
6. **Elegance: 3.** The plans follow the design's steps, but Forgejo's plans and GitHub's are written in two different state-machine styles.
7. **Economy: 2.** Test helpers are retyped in every file, the setup world has two harnesses, and fixtures are retyped across tiers.
8. **Complexity: 3.** It is mostly justified. The host keeps its state in flags, and dialect checks happen twice.

## 3. Strengths

- `github/paste.rs`, `github/reconcile.rs`, `github/write.rs` and `github/hook.rs` follow programming-model 5.4 exactly: each state variant holds its own data, they use the `mem::replace` idiom, secrets are dropped at terminal states, and none of them needs an `expect`.
- Test names are design sentences, for example `a_token_file_holding_another_users_token_is_made_again`. World tests assert on the fake's store, not on request logs.
- `item.rs` puts one total decoder per item in one file. Limit refusals name their `Place` pair as design 6.1 says.
- `skein-forge-conformance` already factors facts, masking, the leak scan and JSON across the Forgejo and GitHub tools.

## 4. Findings

Answers to the area questions:
- **Is 17.6k proportionate?** Mostly, yes. There is no per-step template, and the cost is the duplication described below.
- **Re-implemented connection mechanics?** Yes; see finding 1.
- **Conformance cases written once or twice?** Once, and run only against the throwaway Forgejo; the fake is never held to them (fake-forge.md section 9 makes that the default).
- **Fixtures:** see findings 3 and 12.

**1. The host's connection re-implements skein-forge-connection's transport** (Primitives, Economy)
- `host/connection.rs:28-50` has the same TLS→HTTP→tokenizer pump as `skein-forge-connection/src/connection.rs:126-140`: the seven queues `tls_up`/`tls_down`/`http_up`/`http_down`/`json_up`/`json_down`/`json_next`, and the same `connect`, `stream`, `http_down` and `json_down` functions.
- `host/settings.rs:87-102` `host()` is byte-identical to forge-connection's `settings.rs:75-90`. The `Transport` enum and `field()` (`connection.rs:655`) are copies too.
- The transport and address checks in `host/limits.rs:185-203` mirror forge-connection's `limits.rs:381-410`, down to the same `Unusable` variant names.
- The method mapping appears five times: setup `connection.rs:96` and `:168`, forge-connection `connection.rs:242` and `:275`, and `tests/forge/setup/src/exchange.rs:84`.
- **Fix:** move one transport primitive (the pump, `Transport`, `host`, the method mapping and the transport checks) into skein-forge or a small crate. The setup keeps only its one-request-per-connection policy and its decoding into `Reply`.
- **Size:** about 400 of the 760 lines in `connection.rs`, plus about 60 in `settings.rs` and `limits.rs`. This is unsure until forge-connection's keep-alive and redirect handling is weighed.

**2. The setup world has two harnesses** (Economy, Primitives)
- `World` (`world.rs`, with `exchange.rs` and `decode.rs`, 766 lines) drives the plans directly. `exchange.rs` builds its own HTTP client calls with its own `Host` and `Authorization` headers and its own method mapping, and panics on anything but a Forgejo token.
- `HostWorld` drives the real host and already serves Forgejo (the `hooks` and `host` suites use it).
- Design section 7 says the world "plays the host component". As things stand, the Forgejo stories in `reconcile`, `check`, `take_over`, `reachable`, `access` and `fuzzy/reconcile` never exercise the host's connection.
- **Fix:** port those stories to `HostWorld` and delete `World`. About 500 lines go.

**3. Unit-test helpers are retyped in every file** (Economy, testing-strategy section 10: write a setup once)
- `fn go` appears in 11 files, `silent` in 9, `failure` in 11, `send` in 8, `at` in 11 and `diagnostic` in 4 (`grep` over `src/tests`).
- `const APP: u64 = 19` is declared in 7 files, while `github_fixtures.rs:25` already hard-codes `id: 19`.
- `fixtures.rs:24,280,301,402` has four `Run` wrappers, each with the same `step` body.
- There are three host harnesses with the same fields: `tests/host.rs:104`, and `github_host.rs:112` and `:292`.
- `host::run::Machine` (`run.rs:84-118`) already dispatches `step` and `halt` for all 12 plans. The setup world's `World::drive` already takes a closure, which shows the shared helper is easy.
- **Fix:** one `go(&mut Machine, Event) -> Need` and one `silent(&mut Machine)`. Put `APP`, `BOT`, `URL` and `diagnostic` in `github_fixtures`, and keep one host harness. About 400 lines go.

**4. Forgejo's plans keep state data in `Option` and sentinel fields** (Primitives, rule 5.4)
- `Reconcile` (`reconcile.rs:139-154`: `admin`, `tokens`, `orgs`, `access`), `Create` (`create.rs:35-50`: `admin_login: []`, `branch: []`, `id: None`), `Access` and `Convert` (`convert.rs:34-43`, where `id: 0` and `slug: []` mean "not yet") hold per-state data in fields. About 17 `expect("... before ...")` calls exist only to unwrap them.
- The host does the same with flags: `connection.rs:28-50` (`closing: Option<bool>`, `closed`, `close_sent`, `answered`) and `run.rs:158-175` (`retry`, `retry_at`, `answered`). forge-connection uses a `Phase` enum instead.
- **Fix:** move the data into the state variants, as `paste.rs` does. This is about size-neutral and removes the `expect`s.

**5. Halting works by provoking an unexpected event** (Taste, Complexity)
- `run.rs:119-136` (`halt_begun`) feeds `Event::Begin` to a running plan, relies on it stopping with `Unexpected`, and then rewrites the `why`.
- `reconcile.rs:450-471` feeds `Written` or `Begin` for the same effect.
- The subject is also patched after the stop is built: `reconcile.rs:416-420` (a `let-else` ending in `unreachable!`) and `tokens.rs:239-246`.
- This is fragile: any state that accepted `Begin` would silently advance instead of halting.
- **Fix:** give every plan a `halt` that calls its own `stop(step, subject, why)`, as `create.rs:349-354` and `paste.rs` already do. About 50 lines go, along with two `unreachable!` calls.

**6. The shared plan vocabulary lives in Forgejo's reconcile module, and `Reply` mixes dialects** (Organization, Primitives)
- `Need`, `Event`, `Reply`, `Failure`, `Stage`, `Why` and `Outcome` are defined in `reconcile.rs`. 22 files import them, every GitHub plan among them. The `reply` and `unexpected` helpers live in `reconcile/tokens.rs:218-233`, the token step.
- `Reply` carries `Document`, `Rows`, `GithubDocument`, `GithubRows` and `Unsigned` together. So each of the 34 "reject the rest" arms lists the other dialect's variants; see `access.rs:134-208`.
- **Fix:** add a `plan.rs` module for the vocabulary and split the answer type per dialect. This saves about 70 lines and makes cross-dialect replies impossible to express.

**7. GitHub's plans contain near-copies** (Economy)
- `paste.rs:56-73` and `github/reconcile.rs:57-73` are the same child-runner function.
- The App check (id, then slug, then `lacks`) is written three ways: `convert.rs:140-150`, `paste.rs:84-111`, and `github/reconcile.rs:92-106` with its helper at `:166`.
- Two identical `Progress` enums exist (`hook.rs:22`, `installations.rs:41`), and two `Checked` structs.
- Children answer through a `Queue::uncapped()` that the parent then pops. `Progress::Pending(Need::Done)` is accepted by the type and then rejected at runtime.
- **Fix:** one `app::proved(...) -> Result<App, Why>`, and one `Progress` returned directly with no `Done` in it. About 80 lines go.

**8. `Plan` does not encode its dialect** (Primitives)
- `Dialect` and `supports` (`boundary.rs:93-109`, `settings.rs:47-53`) refuse a mismatch at `host.rs:73`. `run.rs:37-43` and `:249-251` refuse it again, and those arms are unreachable through the host.
- GitHub's `Reconcile { adopted }` is refused at runtime when `adopted` is non-empty (`run.rs:241-244`). `Reachable { installation: Option }` is checked in both directions (`run.rs:228-235`).
- `reachable.rs:29-44` mixes `Github*` states into one enum. Its GitHub constructor (`:86-101`) fills `admin_name: []` and `listing: Token::Admin`, which mean nothing for GitHub. Every other GitHub plan lives under `github/`.
- **Fix:** split `Plan` into Forgejo and GitHub plans, and move GitHub's picker to `github::reachable` (about 180 lines move). About 60 lines go.

**9. Given values and limits are parallel per dialect** (Economy)
- `limits.rs:35-40` and `github/limits.rs:83-88` hold the same deadline and backoff relation check.
- `settings.rs:55-68` maps both kinds into `Times` with identical arms.
- `Place` and `Refused` are each defined twice.
- **Fix:** one `Times { deadline, backoff, backoff_max }` with one check, embedded in both `Limits`. About 40 lines go.

**10. An item is validated in three layers** (Complexity)
- The host turns any read that does not fit its item into `None` (`run.rs:298-306`).
- Five plans call `reconcile::normal` in their `step` anyway (`reachable.rs:470`, `reconcile.rs:350`, `check.rs:130`, `create.rs:342`, `adopt.rs:129`).
- The plans then check again: `reconcile.rs:219` tests `!token.is_empty()`, which is dead after `normal`; `paste.rs:127` and `github/reconcile.rs` re-decode `app_id` and `app_key`.
- **Fix:** give the check one owner.

**11. Naming collisions and test layout** (Naming, Organization)
- Colliding names: `Need` and `Event` in both `reconcile` and `host::boundary`; `Limits` ×3; `Given`, `Refused` and `Place` ×2; `Reconcile` ×2; and a local `enum Token` in `reachable.rs:48` beside `skein_lib::Token`.
- They force aliases such as `reconcile as github_reconcile` (`run.rs:17`) and `tests::host as fixtures` (`github_host.rs:8`), next to a real `tests::fixtures`.
- Failure subjects `b"key"` (21 sites), `b"id"` (13) and `b"installations"` (12) are not in the design's subject list (forge-setup.md:315-319).
- Tests under `src/tests/` are mounted from three parents via `#[path]` (`host.rs:5,11`, `states.rs:113`), and `host.rs:428` carries a `#[path]` that changes nothing. Both `src/tests/host.rs` and `src/host/tests.rs` exist.

**12. Fixtures are retyped across tiers** (Economy)
- `tests/forge/setup/src/fixtures.rs:200-275` repeats the crate's `src/tests/host.rs:48-103` and `github_fixtures.rs:59-84`. That is 13 identical limit fields, the same GitHub given values, the same `crypto` seed, and the same `TAKEN` constant.
- **Fix:** a small `testing/` fixtures crate. About 120 lines go.

**13. Rule slips, in files that otherwise follow the long form** (Cleanliness)
- `let-else` or `if let` on the crate's own enums: `access.rs:70` (on a tuple), `:257`, `:261`; `orgs.rs:46`; `create.rs:339`, `:350`; `host.rs:69` (on a tuple); `reconcile.rs:417`.
- `call.rs:11-20` hand-writes `From` impls, which section 10.3 rules out.

**14. Conformance and throwaway** (Economy, Taste)
- The throwaway's `instance.rs:318-389` re-implements the JSON parse and two-pass write that `skein-forge-conformance/src/json.rs:15-46,226-242` already provides.
- Shim modules: the conformance's `json.rs` is a bare re-export, and `write.rs` wraps `Secrets` in a `Deref` newtype and classifies a secret by its length (40 or 64) at `:15-24`.
- `cases.rs:248-249` branches on row labels (`starts_with("branch-protection")`, `== "raw-file-range")`); the rows should carry the token and range instead.
- About 104 fact checks re-read the masked transcript they just wrote, by its string name (for example `tokens.rs:34` and `:101`), even though the exchange was in hand. This may be deliberate, so the facts judge the committed output (unsure).

## 5. Cross-area suspicions

- **skein-forge-connection:** the shared transport of finding 1. Its `settings.rs` and `limits.rs` likely carry the rest of the duplication.
- **skein-forge:** each dialect's `request::Method` should probably map to `skein_http::Method` once, there. Its token scopes are spelled out separately in `reconcile/tokens.rs:79-85`, the world tests and the conformance's `tokens.rs:12-14`; the dialect could name them.
- **tests/forge/github:** it likely repeats the conformance shims (it has its own `mask.rs`). Unverified.
- **Fake-forge step tests:** they likely restate the Forgejo facts on their own, since no check ties the fake to `facts.txt`.

## 6. Rule costs

The exhaustive "reject the rest" arms cost about 450 production lines, roughly 6%:
- about 53 `Event` arms, such as `Event::Begin | Event::Read(_) | … => Unexpected`;
- 34 `Reply` lists of 5 to 8 variants;
- the 27-variant `Document` match in `account()` (`reconcile.rs:308-347`) and three 20-variant `Row` lists.

Splitting `Reply` per dialect (finding 6) recovers part of this within the rules.