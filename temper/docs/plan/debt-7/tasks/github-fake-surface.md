+++
id = "github-fake-surface"
part = "fake-github"
kind = "interface"
needs = []
files = ["skein/crates/skein-fake-forge-github/src/inline.rs", "skein/crates/skein-fake-forge-github/src/fixture.rs", "skein/crates/skein-fake-forge-github/src/lib.rs", "skein/crates/skein-fake-forge-github/src/server.rs", "skein/crates/skein-fake-forge-github/src/service.rs", "skein/crates/skein-fake-forge-github/src/writes.rs", "skein/crates/skein-fake-forge-github/src/installations.rs", "skein/crates/skein-fake-forge-github/src/tests.rs", "skein/crates/skein-fake-forge-github/src/tests/inline.rs", "skein/crates/skein-fake-forge-github/README.md"]
effort = "high"
minutes = 100
+++

# fake-github: the surface every face builds on

## Goal

Add to `skein-fake-forge-github` what four worlds need to drive `server::Server` instead of an HTTP face of their
own, and the fixture they build from: `skein/docs/design/fake-forge.md`, 10.6. Additions only: no reader moves
here, and `Peer::github` stays byte for byte the same. The faces follow in their own tasks.

Acceptance:

- Every name, signature and value in `temper/docs/plan/debt-7/notes/fake-github.md`, 1 exists as written.
- `Inline::serve` and `blob` run a request through `Server` with `charset` on; a `Server` built by `Server::new`
  or `Server::blob` answers as at base.
- The new tests below pass and each bites.
- `skein-fake-forge-github`, `skein-fake-peers`, `skein-fake-peers-tests`, `skein-fake-forge-tests` and
  `github-conformance` pass with no test file changed but `src/tests.rs`'s one `mod` line.
- Net lines at most +650 (`src` +500, tests +187). This task adds lines; the part falls in the tasks after it.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`; `temper/docs/plan/debt-7/notes/fake-github.md`, 1 and 5.
- `skein/docs/design/fake-forge.md`, 10.6, then 10 and 5 (the peer's rules of sharing, which `Lent` mirrors).
- `skein/AGENTS.md`; `skein/docs/foundation/programming-model.md`, 10: this crate is step code: no closure, no
  trait object, no `Vec`, every loop bounded by a named constant.
- `skein/crates/skein-fake-peers/src/forge.rs`, `serve_event` and `dispatch`: what a host does with each `Event`.
- The face code this replaces, read only: `skein/tests/forge/machine/src/github.rs` (`finish`, `domain_reply`,
  `Fake::configured`), `skein/tests/forge/protocol/src/github.rs` (`release_hooks`, `domain_call`).

## Owns and may change

- Owns: `src/inline.rs`, `src/fixture.rs`, `src/tests/inline.rs` (new); `lib.rs`'s two `pub mod` lines; a paragraph
  in the crate's `README.md` naming `inline` and `fixture`.
- `server.rs`: the field and `charset`, `credential`, `respond`'s one media line, and the `#[expect]` of the answer
  below. Nothing else there.
- `service.rs`: `ContentType::charset`, the `ambiguous_updates` field and setter.
- `writes.rs`: `Stage::Updated if error == a::Error::Conflict` answers `texts::UPDATE_STALE` when the service's
  flag is set, `texts::UPDATE_CONFLICT` otherwise.
- `installations.rs`: `minted_token`, reading `Issued.mint`.
- `tests.rs`: `mod inline;`.
- **Not here:** `src/tests/fixtures.rs` and every other reader. `fixture.rs` copies their values; the readers move
  in `github-fixture-readers` and the faces' tasks.
- **Options.** If `Server` needs a `pub(crate)` read for `pass` (a pending head), add it and say so. If a private
  helper of `service.rs` or `pages.rs` must widen to `pub(crate)`, do it. A clippy bound on argument count at
  `pass`: a scoped `#[expect]`, with its reason.

## Steps

1. **`fixture.rs`**, to the note's table. `limits()`'s `inner` keeps the machine world's expression (the longest
   of `github::setup::EVENTS` and `PERMISSIONS`). Module doc cites `skein/docs/design/fake-forge.md`, 10.6.

2. **`inline.rs`**. `pass` is the loop every face wrote, once:

   ```rust
   // More events than one request's admission and continuations raise.
   const EVENTS: u32 = 256;
   pub fn pass(server, service, lent, env, above, below) {
       for _ in 0..EVENTS {
           if server.head_pending() { return; }
           if let Some(event) = above.pop() {
               match event {
                   Event::Domain(event) => {
                       if lent.hooks { release(service, lent.domain, lent.env); }
                       let reply = reply(lent.domain, lent.env, event);
                       server::down(server, service, env, reply, lent.domain, above, below);
                   }
                   Event::Rate { key, limit, window } => {
                       let result = domain::rate(lent.domain, lent.env, key, limit, window);
                       server::admitted(server, service, env, result, above, below);
                   }
                   Event::Close | Event::Closed => {}
               }
           } else if server.has_work() {
               server::resume(server, service, env, above, below);
           } else {
               return;
           }
       }
   }
   ```

   `reply` is private: `step_admitted`, then at most `FIRES` (64, "more timers than a fixture's queue and CI can
   have due at once") `fire`s while the queue is empty, then the one `Request::Reply`, `assert!` nothing else is
   queued, `domain.reclaim()`. A `Hook` there is `unreachable!`: a hooked world sets `Lent::hooks`. `call` is
   `reply` with the `Result` taken out.

   `release` is `skein/tests/forge/protocol/src/github.rs`, `release_hooks`, on the arguments' wall: a `Hook` that
   renders a post is a fixture error, `assert!(post.is_none(), "App hook has no endpoint")`.

   `Inline::exchange` (private; `serve` and `blob` differ in `Server::new` or `Server::blob`):

   - `env` is `Env { now: Time::from_nanos(wall.as_nanos()), wall, limits }`, and the `Lent`'s the same over
     `config`, `hooks: false`. Owner: a named `Token`. The server has `charset` on.
   - An `Intake` of `request.len()` or `skein_http::server::largest_read(&limits.http)`, the larger, holds the
     request; a `List<u8>` of capacity `limits.http.response + limits.response` takes the output, and `into_boxed`
     ends it. `start`, then a bounded loop: `pass`; drop `take_head()`; pop `below`. As
     `skein/tests/forge/machine/src/pipe.rs`, `End::up`, answers one demand: `Demand { read, room }` is
     `up(Bytes)` if `intake.meet(read)` has bytes, else `up(Room)` if `room > 0`, else nothing; `Send` goes to the
     writer; `Finish` ends it. It ends when `below` is empty and the server has no work.

3. **The small additions**, as the note has them. `credential`:

   ```rust
   /// The request in hand's Authorization value, until it is dispatched. A world's tally reads it beside
   /// `take_head`, which blanks the head's own.
   pub fn credential(&self) -> Option<&[u8]> { self.call.as_ref()?.header(b"authorization") }
   ```

   `ContentType::charset`: `Json`, `Html` and `Plaintext` with `; charset=utf-8`, `Octets` as `bytes`.

4. **Tests**, `src/tests/inline.rs`, over one `forge(seed) -> Inline`: `fixture::domain_config`, one repository from
   `fixture::setup(2, fixture::PATH)`, `fixture::BOT` granted write, `fixture::config(fixture::widgets(2))`,
   `fixture::limits`, `install_one`. A `request(method, target, authorization, body)` writes a `Host` line.

   | Test | Shows |
   |---|---|
   | `an_app_request_is_answered_whole` | JWT on `/app/installations`: `200`, `Date`, `Content-Length` equal to the body, `application/json; charset=utf-8` |
   | `a_token_read_is_admitted_then_called` | a minted token reads a branch: `X-RateLimit-Used: 1`, the commit in the body (`Event::Rate`, then `Event::Domain`) |
   | `a_conditional_read_admits_after_its_call` | `If-None-Match` on the repository, twice: `304`, no `Content-Length`, the quota paid once more only when the validator misses |
   | `a_blob_request_records_its_headers` | a tampered path on `blob`: `403`, empty, one record with `range` set |
   | `a_request_without_host_is_refused` | `400` from the HTTP server, and the service records nothing |
   | `a_pass_stops_at_a_head_and_lends_its_credential` | `start`, `up` a head: `take_head` blanks it, `credential` has the value, after the next `pass` it has none |
   | `a_hooked_pass_releases_due_hooks_ahead_of_its_call` | a hooked repository, a write, then a call in a pass with `hooks: true`: the delivery is recorded before the reply |
   | `an_ambiguous_update_answers_the_stale_refusal` | through `fixtures::Harness`: `422` with `texts::UPDATE_STALE` on a conflict, `UPDATE_CONFLICT` without the flag |
   | `a_minted_token_is_read_by_its_number` | `minted_token(1)` is the token; `minted_token(2)` is `None` |
   | `the_fixture_builds_a_valid_service` | `config(repositories())` and `config(widgets(1))` under `limits()`; `install_three` gives `[1, 2, 3]` with 3 suspended |
   | `the_peers_server_names_a_bare_media_type` | a `Server::new` driven with `pass`: `Content-Type: application/json` |

5. Break each rule by hand (common note). `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy
   -p skein-fake-forge-github --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-fake-forge-github -p
   skein-fake-peers -p skein-fake-peers-tests -p skein-fake-forge-tests -p github-conformance`. Citations. Commit and
   push: `fixture.rs`, then `inline.rs`, then the small additions, each with its tests.

## Report

The common note's items; the break table; net lines, `src` and tests apart; which `pub(crate)`s you widened.

## Answers

2026-11-04. Writer or `List<u8>` for the output, and how large an intake? **`List<u8>`, no `Writer` change.** A
`Writer` is for a length computed first (`skein-lib/src/writer.rs`); this one is known only as it fills, and `List`
is bounded and allocates as it fills. `Down::Send` pushes its bytes in a loop over them, `expect`ing the push, and
both ends answer `into_boxed`. **The intake:** step 2 now names `skein_http::server::largest_read(&limits.http)`,
the server's own bound for the stream below, the larger of it and the request's length. It replaces the pushed
`max` of `head` and `read`.

Three faces ran on the pushed work and found what this task must finish, so the next run finishes all of it:

- The rest of the task: `Service::ambiguous_updates` and its `writes.rs` branch, `Installations::minted_token`, the
  three tests not written, the README paragraph, every test's break. `github-protocol-face` can't compile without
  the two methods.
- `heavy cargo clippy -p skein-fake-forge-github --all-targets -- -D warnings` has 17 errors at the pushed commit.
  `#[must_use]` and assert messages where it asks. `partial_pub_fields`: `Inline::limits` is `pub`, and
  `notes/fake-github.md`, 1 now says so; no exception. `struct_excessive_bools` on `Server`: a scoped `#[expect]`
  with its reason, the one this task may add there.

**Size.** The cap was set by the cut. The push measures +604 (`src` +406, tests +198) before the work above, about 70
lines more. First compact what can go without losing a test or a rule: the shared parts of `Connection` and of the
whole-request tests above all, and `src` by what `List` saves. Why the cap can't hold: the fixture's table alone
takes `fixture.rs` 204 lines, and the lints add a line each. So the cap rises by the 25% the plan's owner allows: **+650,
`src` +500, tests +187**. Past any of the three after compacting, stop and ask, with the measurements.
