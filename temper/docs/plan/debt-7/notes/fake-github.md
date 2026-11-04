# debt-7, fake-github: the surface, and what stays the same

Part `fake-github` pays `skein/docs/development/debt/D029-fake-github-http-faces.md`. Every `github-*` task
names this note beside `notes/common.md`. The contract is `skein/docs/design/fake-forge.md`, 10.6.

## 1. The surface

`github-fake-surface` creates exactly these, in `skein-fake-forge-github`. Later tasks build to them and
change none of them; a face that needs another name asks.

```rust
// src/inline.rs, `pub mod inline`
pub struct Lent<'a> {
    pub domain: &'a mut domain::Domain,
    pub env: &'a Env<domain::Config>,
    /// `release` runs ahead of each call of the pass.
    pub hooks: bool,
}
/// One call at zero latency, as admitted: `step_admitted`, then `fire` until its reply, then `reclaim`.
pub fn call(domain: &mut domain::Domain, env: &Env<domain::Config>, event: domain::Event)
    -> Result<domain::api::Answer, domain::api::Error>;
/// Fires every timer due at `env.now`, hands each hook to `Service::hook`, then polls the service.
pub fn release(service: &mut Service, domain: &mut domain::Domain, env: &Env<domain::Config>);
/// Drives one connection until it waits on bytes or holds an untaken head; at once if it holds one.
pub fn pass(server: &mut Server, service: &mut Service, lent: &mut Lent<'_>, env: &Env<Limits>,
    above: &mut Queue<Event>, below: &mut Queue<Down>);

pub struct Inline { pub service: Service, pub domain: domain::Domain, pub config: domain::Config, limits: Limits }
impl Inline {
    pub fn new(service: Service, domain: domain::Domain, config: domain::Config, limits: Limits) -> Self;
    /// One whole request on a fresh API connection; the whole response's bytes.
    pub fn serve(&mut self, request: &[u8], wall: Wall) -> Box<[u8]>;
    /// The same on the blob listener.
    pub fn blob(&mut self, request: &[u8], wall: Wall) -> Box<[u8]>;
}

// src/fixture.rs, `pub mod fixture`
pub const APP: u64 = 7;
pub const BOT: u64 = 9;
pub const OWNER: u64 = 22;          // acme's account
pub const PERSON: u64 = 31;         // alice's account
pub const PATH: &[u8] = b"acme/widgets";
pub fn key() -> &'static [u8];      // keys::KEYS, the first
pub fn app() -> App;
pub fn accounts() -> List<Account>;
pub fn repositories() -> List<Row>;
pub fn widgets(id: u32) -> List<Row>;
pub fn config(repositories: List<Row>) -> Config;
pub fn limits() -> Limits;
pub fn install_one(service: &mut Service, at: Wall);
pub fn install_three(service: &mut Service);
pub fn domain_config() -> domain::Config;
pub fn setup(id: u32, name: &[u8]) -> domain::api::Setup;

// Beside them
impl Server { pub fn credential(&self) -> Option<&[u8]>; pub(crate) fn charset(self) -> Self; }
impl ContentType { pub const fn charset(self) -> &'static [u8]; }
impl Service { pub fn ambiguous_updates(&mut self); }
impl Installations { pub fn minted_token(&self, mint: u64) -> Option<&Secret>; }
```

**The fixture's values** are the machine world's, so the largest reader moves without a change of value:

| Item | Value |
|---|---|
| `app()` | id `APP`, slug `test-app`, owner 1, no permissions, no events, `key()`, webhook `test-webhook` |
| `accounts()` | `acme` (`OWNER`, organization), `alice` (`PERSON`, user) |
| `repositories()` | owner `acme`/`OWNER`, in this order: 4 `tools` (not organization, public), 2 `widgets` (organization, private), 6 `absent` (organization, public) |
| `widgets(id)` | `acme/widgets` alone at `id`, organization, private |
| `config(rows)` | `delivery::Config::standard()`, quota 5000, window 3600 s, `TOKEN_LIFE`, base `http://127.0.0.1:3000`, blob `http://logs.example.test`, no third origin or redirect, `app()`, `AppFacts::Complete`, `Shown::Ruleset`, ruleset 17, both booleans false, hook `https://hook.test/start`, `rows`, `accounts()`, person `alice`, no tokens, no faults, bot `BOT` |
| `limits()` | as `skein/tests/forge/machine/src/github.rs`, `Fake::configured`, today, `inner` written as there |
| `install_one(s, at)` | `acme` (`OWNER`, organization), `Reach::All`, at `at`: id 1 |
| `install_three(s)` | as `Fake::configured` today: `acme` listed `[2, 4]` at 1000 s, `example` (23, user) at 1001 s, `other` (24, user) at 1002 s, all `Reach::All` but the first; 3 suspended |
| `domain_config()` | as `skein-fake-forge-github/src/tests/fixtures.rs`, `domain_config`, today |
| `setup(id, name)` | no workflows, default `main`, empty tree and labels, no protection, not hooked; `Checks` with no cues or contexts, zero latency, nothing silent, `passes` 1000, no reruns, no cue |

A world varies them by struct update (`Config { hook, ..fixture::config(fixture::repositories()) }`) and
names no other field.

## 2. What the worlds keep

Each face stops parsing requests, writing heads and looping over `Outcome`. It keeps what is the world's:

| Face | Keeps |
|---|---|
| `skein-forge-connection/src/tests/github_files/fake.rs` | `Fake`'s fields and its `new`, `with_files`, `answer`; `object`; the domain's content |
| `skein/tests/forge/machine/src/github.rs` | every public name of `Fake`, and `WALL_BASE`, `BLOB_ADDRESS`, `BLOB_ORIGIN`, `app_key`, `bearer`: the setup world and temper read them; `fixture_domain` and its helpers |
| `skein/tests/forge/machine/src/joint/github.rs` | the pipes and cut draws, `heads`, `line_indices`, the tally and `route`, the start and request counts |
| `skein/tests/forge/protocol/src/github.rs` | the pipes, `Trace`, both framing observers, `Accounting`, `secrets` and `mints`, `lose_creation`, `release_hooks`' timing, `replace`, `rotate`, `direct` |

`machine::Machine`'s `Peer` and scripts stay as they are: they hold a whole request and write whole responses.
Domain content (branches, pulls, commits) stays in each world. D030's one io harness and D032's shared pieces
are not this part: don't merge the worlds' peers.

## 3. What stays the same, and how each task checks it

A test's events and outcome stay. The check for every face: **the suite's test files don't change**, but for
the lines section 4 lists; the test count of each package named here equals its count at base; all pass.

| Family | Stays | What differs below the tests | Extra check |
|---|---|---|---|
| `skein-forge-connection`, `tests::github_files::*` and the heap ceilings | every `Answer`, `cost`, ceiling | a `Date` and a charset on replies, no `Content-Type` on an empty body | the two `largest_github_*` peaks, printed, base and branch, in the report |
| `skein-forge-machine-world`, `github_*`; `skein-forge-setup-world` | statuses, bodies, heads the tests read | a head's field order; no `Content-Length` on 204 or 304; no `Content-Type` on an empty body | a differential run (below) |
| `skein-forge-machine-world`, `github_joint`; `temper-engine-protocol-forge-world` | `tally`, `request_lines`, `start_counts`, `request_counts`, answers | rate fields and `ETag` now reach the component, so `Answered.rate` is `Some` and a log poll may be answered `304`; `Date` as before | `the_joints_request_lines_are_the_same_after_many_polls` unchanged; request lines of three named stories compared, base and branch |
| `skein-forge-protocol-world`, GitHub stories | `accounting`, `lines`, costs, `secrets_absent`, `github_mints`, the referee's verdicts | the real media type, which only corrects: a `Location` on a redirect; one more field on an issue's `201`; no `Content-Length` on a `304` | request lines of three named stories compared, base and branch |
| `skein-fake-peers-tests`, `skein-fake-forge-tests`, `github-conformance` | everything: `Peer::github` is byte for byte the same | nothing | run them after any change to `server.rs` |

**A seeded sweep** can land its byte cuts elsewhere when a response's length moves. Its assertions don't change.
If one goes red (`cuts > 0`, `cut_heads > 0`, a count), that's a test's outcome changing: stop and ask, with the
seed and what moved.

**The differential run** is throwaway, never pushed. While `Fake::serve`, `blob` and `page` go through `Inline`,
keep beside them, untracked, a second `(Service, Domain)` built the same way and answered by the old loop (the
base's `finish`, `page` and `domain_reply`, copied), feed it every request, and assert status, body, `Location`,
`Link`, `ETag` and the rate fields equal. Run `skein-forge-machine-world` and `skein-forge-setup-world` with it.
Two services of one seed fed the same requests draw the same tokens and ids. The report gives the requests
compared.

**Request lines compared** means: at base and on the branch, in a throwaway print, the `lines()` or
`request_lines()` of the stories the spec names, at cut chance 0, diffed. They must be equal, line for line.

**A budget** moves in no task: a face adds no test and drops work per request. Each face's report gives its
packages' nextest totals at base and on the branch, on one machine. If a package's total rises by more than a
tenth, or a test passes a budget (`skein/docs/foundation/testing-strategy.md`, 8.1), stop and ask, with both
numbers.

## 4. What a test may change

This departs from `notes/common.md` ("a test may change only where it names a moved item") only for these:

- `skein/tests/forge/machine/tests/suite/github_peer.rs`, `request`: one line, `Host: github.test`. The fake's
  server refuses an HTTP/1.1 request without `Host` (`skein-http/src/server/head.rs`), and this helper wrote
  none. No assertion moves.

Nothing else. An assertion, a timing constant or a seed that would change is a stop.

## 5. Size

Caps are per branch, `git diff --shortstat` against its base, net:

| Task | Cap |
|---|---|
| `github-fake-surface` | +520 (this task adds lines: `src` at most +400, its tests at most +150) |
| `github-connection-face` | -250 |
| `github-machine-face` | -330 |
| `github-protocol-face` | -160 |
| `github-joint-face` | -80 |
| `github-fixture-readers` | -130 |
| `github-close` | -30 |

The part ends near -600 net. The faces' gross fall is about 1.2k, as the review put it; the surface and the
fixture's literals stay. A negative cap is a floor on what falls: ending above it (less removed) is passing it.
