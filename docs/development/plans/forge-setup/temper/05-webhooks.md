# Session 05: webhooks

temper takes skein's `sha256` (a repin in a commit of its own), the fake forge's protocol layer renders signed deliveries,
the forge component listens, verifies each delivery as its body streams and raises a hint, and `application.webhooks` with
each forge's `webhook_secret` configures it, `temper-service` hosting the listener (forge.md, 5; setup.md, 14, item 3). A
delivery is a hint: polling finds what one lost. The box (sessions 07 to 13) does not wait on this session; session 11
connects it to a real Forgejo.

## Read first

- temper `docs/design/`: forge.md 5, 7, 8 (the webhook connection's table), 9, 10; configuration.md 3 (`webhooks`,
  `webhook_secret`), 6, 7 (the listener's cross-checks); credentials.md 2, 5, 7; protocol.md 5, 6; setup.md 10.
- skein `docs/design/lib.md` 6.2 (`Hmac`, `equal`), `http.md` 5.1 to 5.5, `io.md` 3.2 (the accept);
  `docs/foundation/programming-model.md` 4.3, 5.4, 6.3, 7, 9, 10.2; `testing-strategy.md` 2.4, 2.5, 3, 6.
- This plan: `decisions.md` 2, 3, 5, 6; `findings/webhooks.md` 1, 3, 4, 6; `skein-main-in-temper.md`; `client-api.md` 2;
  `configuration-and-binary.md` 1, 2; `skein-sha256.md` 1; `temper/04-deliveries.md` (04.2's documents, 04.1's `deliveries/` and
  `hmac_sha256`); `temper/07-prepare.md` (the setup's view: `check::Forge`, `Members.listeners`, `Named`); client-api's
  `temper/05-server.md` (05.1, 05.2).

## What the design asks

- **forge.md, 5 and 8.** A loopback listener, `POST /hooks/<number>` per forge with a secret (else `404`; another method
  `405`), one request at a time per connection, kept alive; the body's HMAC-SHA256 in `X-Forgejo-Signature`, computed as the
  body streams and compared with `sha256::equal`; `204` queued before the hint goes up; `403`, `413`, and `400` for a verified
  body that does not decode; every refusal closes after its answer; a request not read within `hook_deadline` is closed with
  no answer. 05.2 builds it all; the events' decoders are 04.2's, the org hook is 09.3's.
- **forge.md, 5, 7, 9, 10.** A failed listener stops the engine, as the clients' does (drain, exit 1); `hooks` caps the
  connections, "beyond which the listener accepts no more until one closes"; the fake renders the signed request and the world
  writes it to the listener (05.1); the machine world scripts deliveries, the protocol world joins the fake's (05.2: one joint
  test; the connector world's hinted stories wait for session 06).
- **configuration.md, 3, 6, 7; credentials.md, 2, 5, 7.** The two members (`null` for none), a private file read at start
  within `hook_secret_bytes`, the listener's checks, the names (05.3); no secret across a domain boundary.

## Choices this plan makes

- The secrets live in the component's own table, `Webhooks.secrets`, not in `settings::Forge`: a secret exists only with
  its listener (forge.md 5), and the `kit::settings::Forge` literals stay as they are.
- `"webhooks": null` and `"webhook_secret": null` mean none; a missing member is refused, naming its path (answer 31, as
  `read::required` does). Every document and test configuration gains both in 05.3.
- The hint goes up when the exchange is `Done`, not at the body's end. A hint the root refuses is held by its connection,
  which reads no body while it holds one (a `Call` that arrives waits, bound by `hook_deadline`); `Offer` sends it up again, as
  it does answers. In the drain it is dropped.
- At `hooks` connections the listener leaves its one `Accepted` socket unbound until one closes: io announces no other
  (io.md 3.2), so a delivery waits in the kernel's backlog and none is refused (forge.md 7).
- The signature is compared as lowercase hexadecimal, as Forgejo writes it; port 0 never clashes in the uniqueness check.

## Rules that bind it

- **Limits come from the shipped profile** (development.md 5.6): `limits::profile(Profile::Small)` (`crates/temper-service/src/limits.rs:247`)
  holds decision 5's hook values; a test that needs another says so beside them.
- **`Cargo.lock` is never merged by hand;** only 05.1 moves skein. **No secret value** in a `Debug`, event, request, fact or
  diagnostic (credentials.md 7). **Don't (beyond the plan's):** no `sha2`, `hmac` or `subtle`; no hand-written comparison of
  secret-derived bytes.

## Increments

### 05.1 temper takes skein's SHA-256, and the fake forge renders signed deliveries

Scope: two commits. The first moves temper's skein, `Cargo.lock` alone, onto skein 01.2's landing or a later commit
(decisions.md 2, 3; `findings/skein-main-in-temper.md`). The second gives the fake forge's protocol layer (forge.md 9) a signed
request for each hook its domain sends, for a world to write to the listener (05.2), and replaces the conformance's stand-in
`hmac_sha256` with skein-lib's `Hmac`. Nothing connects here.
Needs: skein 01.2's landing (`sha256::Hmac`, `equal`); temper 04.2, and so 04.1 (`temper_forge_forgejo::delivery`,
`server::delivery`, and the tool's `deliveries.rs`). client-api's ruling 154 is met (on main at `c36858db7`).
Files:
- `Cargo.lock`, the repin's commit, alone: `<sha>` is `git -C ~/src/rust/skein fetch origin && git -C ~/src/rust/skein log
  origin/main --all-match --grep='^Plan: forge-setup$' --grep='^Step: skein-01.2$' --format=%H -1`; `cargo update -p
  skein-lib --precise <sha>`; `grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' Cargo.lock | sort -u` then prints
  jig's and smith's unchanged and skein's `<sha>`;
- new `testing/temper-fake-forge-protocol/src/delivery.rs`; `src/lib.rs:5-50` (`mod delivery`, `Config.hooks: Option<Endpoint>`);
  `src/service.rs:23` (`new` refuses an `Endpoint` with an empty secret or host);
- the three `Config` literals, among the 8 lines of `git grep -n 'default_paging_num:' -- '*.rs'` (`src/tests/fixtures.rs:22`,
  `testing/temper-fake-git/src/fixtures.rs:109`, `tests/engine/protocol-forge/src/fixtures.rs:156`), gain `hooks: None`;
- `tools/forgejo-conformance/src/deliveries.rs` (04.1's `hmac_sha256` and its callers take `skein_lib::sha256::Hmac`; the
  crate already depends on skein-lib) and `src/tests.rs` (04.1's `hmac_sha256_reads_rfc_4231_case_2` goes: skein 01 holds it);
- tests: new `src/tests/deliveries.rs` (and `mod deliveries;` in `src/tests.rs`), `src/tests/fixtures.rs`.
Shape: `delivery.rs` is the first sketch. A domain `Request::Hook` (fake-forge-domain `boundary.rs:34-49`) becomes, by its
`change` (06.1 refines this), 04.2's `delivery::Delivery`: Issue `Issue` and Comment `Comment` (an empty `Body::Kept`: no
key yet); Pull `Pull`; Review `Review { event: Approved }`; Push `Push { reference: refs/heads/<branch>, after:
routes::object(commit), pusher: by }`, or `Ref { created: false, .. }` when `commit` is none; Status `Job { end: Success,
.. }` (its commit standing for the run's); Wiki none. The repository is `Config.repositories`' id by `full`; `routes::object`
is `routes.rs:14`. `server::delivery::encode(&Delivery, &documents)` gives `Sent { event, body }`; the head is `POST
<delivery::hook_target(forge)> HTTP/1.1`, then `Host`, `Content-Type: application/json`, `Content-Length`,
`delivery::EVENT_HEADER` with `Sent.event`, `delivery::SIGNATURE_HEADER`, in that order. `sign` is
`Hmac::new(secret.expose())`, `update(body)`, `finish`, in lowercase hexadecimal. `server::down` (`server.rs:132`, its `Hook` arm :140) and
`Service::target` (`service.rs:94`) keep ignoring a `Hook`, as do the worlds that discard one (`tests/engine/forge/src/world.rs:358,530`); `Joint` (`joint.rs:197`)
changes in 05.2. temper 06.1 reshapes `Request::Hook` (a job's end for `Status`, bodies for issue, comment and review hooks, no
`Wiki`): whichever of 05.1 and 06.1 lands second adapts the mapping in place, a body going in as `Body::Kept`.
Test shape: `src/tests/deliveries.rs`, on `fixtures::config()` (`fixtures.rs:6`), `limits()` (:29) and `Harness::new` (:76), the
fake's own bounds. New in `fixtures.rs`: `hooked_config()` (forge 1, host `127.0.0.1:7200`, `FIXTURE_SECRET =
b"fixture-hook-secret"`) and `Harness::hooked()` (:99 with `hooked: true`). Expected values are literals: the body is 04.2's
encoding of that push, the signature the output of `printf '%s' '<body>' | openssl dgst -sha256 -hmac fixture-hook-secret`,
run once by the author and pasted beside the body. Reference test:
```rust
#[test]
fn a_push_hook_is_delivered_as_one_signed_post_to_the_forges_path() {
    let service = Service::new(fixtures::hooked_config(), &fixtures::limits(), 3).expect("service");
    let hook = domain::Request::Hook {
        repository: boxed(b"org/repo"), change: domain::api::Change::Push, number: None,
        branch: Some(boxed(b"main")), commit: Some(7), by: Some(2),
    };
    let delivery = service.delivery(&hook, &fixtures::limits()).expect("a hooked forge delivers");
    assert_eq!(&*delivery.body, PUSH_BODY, "04.2's encoding of this push, as a literal");
    assert_eq!(&*delivery.head, PUSH_HEAD, "POST /hooks/1, the fields in order, the openssl signature");
}
```
Tests (each < 20 ms): the above; `an_issue_hook_is_delivered_under_the_issues_event_naming_its_number`;
`a_status_hook_is_delivered_as_a_jobs_end_naming_its_commit`; `a_deleted_branch_is_delivered_as_a_delete_event`;
`a_wiki_hook_is_not_delivered`; `a_hook_with_no_endpoint_is_not_delivered`;
`the_delivery_signature_is_rfc_4231s_for_its_key_and_data` (case 2, key `Jefe`);
`a_domain_push_is_delivered_signed_to_the_world` (`Harness::hooked()`: the real fake domain's `Hook` becomes the literals).
Proof: world, `cargo nextest run -p temper-fake-forge-protocol -E 'test(deliver)'`: the fake domain's push to a hooked
repository comes out as a request whose signature is `openssl`'s for its body, in under 5 s; and the repin, `cargo check
--workspace --all-targets`, exit 0 with no warning, in under 120 s (48 s at the base). 05.2's joint test connects the bytes.
Don't: no client or socket machine in the fake; no payload encoder of its own; no hexadecimal or HMAC outside
`delivery::sign`; no change to `Server` or to `Config`'s other members; no CI or mergeability semantics (06.1).
Size: at most 120 lines.
Removes: the conformance's `hmac_sha256` and its RFC 4231 test.

### 05.2 The component's listener: signed deliveries become hints

Scope: forge.md 5, 7 and 8 in `temper-engine-protocol-forge`: `Listen` at `begin`, one skein-http server per accepted socket,
each body hashed with `sha256::Hmac` and decoded (04.2) as it streams, answered, then hinted; the five hook limits, their checks
and the worst case; a refused hint held and offered again; a lost listener stops the engine. The routing's arms come with it;
no listener is configured until 05.3.
Needs: temper 04.2 (`client::delivery::{decoder, first, step}`, `Delivery`, `hook_forge`, the headers); 05.1 (the world's deliveries).
Files:
- new `crates/temper-engine-protocol-forge/src/hooks.rs`, `hooks/connection.rs`, `hooks/hint.rs`; `component.rs` (:93 `begin`, :102
  `down` with `Close` :161, :178 `up` and its arm :238-246 that ignores `Listening` and `Accepted`, :250 `fire`, :282
  `next_deadline`, :451 the close), `boundary.rs`, `settings.rs:8`, `limits.rs` (`Stacked` :76, `Unusable` :85, `check`, `worst_case`), `lib.rs`;
- literals the new members break (`git grep -n -e 'start_backoff_max:' -e 'Settings { forges' -- '*.rs'`, 9 lines with
  `limits.rs:44`: the crate's `src/tests/fixtures.rs:24,98`, the world's `src/fixtures.rs:25,99`, `temper-service/src/forge.rs:76,129`,
  the conformance's `component/hosted.rs:105,134`) and the matches on `kit::Event` that must name the new variants (the crate's
  `src/tests/fixtures.rs:156`, the world's `machine.rs:180` and `joint.rs:204`, the conformance's `component/hosted.rs:255`,
  with its `Limits` literal at `hosted.rs:87`);
- `temper-service`: `forge.rs` (`hinted` beside `event` :93; `back` :101), `engine.rs:385` `forge_pass` (hotspot, client-api 05.1),
  `stop.rs` (`hooks` beside `forge` :78; not `listener`: `Stop::Listener`, `lib.rs:49`, is the clients' listener's),
  `lib.rs:46` (`Stop::Hooks`), `crates/temper/src/host.rs:57` (its words); `forge.rs` `problems` (:195, its matches :197
  and :212) names both limits of each new `Unusable` and `Stacked` as :212-221 do (`SecretBytes`: `hook_secret_bytes`
  alone; `ServerOverIo`: `hook_head_bytes`, `io.intake`), and `indexed` (:257) learns `Secret`: without them
  temper-service stops compiling at 05.2;
- the world: new `tests/engine/protocol-forge/src/hooks.rs`, `tests/{hooks,fuzzy_hooks}.rs`; `joint.rs` (:79 `hooked`, :197 the
  discarded `Hook`, :261 the `Listen` panic), `fixtures.rs`, `Cargo.toml`, `Cargo.lock` (a dependency `temper-service`); the
  crate's step tests in new `src/tests/hooks.rs`.
Shape (sketch 2):
- Limits: `hooks`, `hook_head_bytes`, `hook_body_bytes`, `hook_deadline: Duration`, `hook_secret_bytes`, flat, so the names are
  `forge_component.hooks` and so on. `forge::small` (`forge.rs:110`) sets decision 5's: 4, 8192, 1_048_576, 10 s, 128. `check`
  refuses a zero of any (`Unusable::Zero`); `hook_secret_bytes` below 32 (`SecretBytes`); a secret of no forge of `forges`, empty
  or past `hook_secret_bytes` (`Secret { forge }`); a listener off loopback (`Listener`); the server over io (`Stacked::ServerOverIo`:
  `skein_http::server::largest_read`, at least `hook_head_bytes`, within `io.intake`, as configuration.md 7 words it, answer 43;
  `largest_room` within `io.output`) and the tokenizer over the server (`JsonOverServer`: `largest_demand` within `read`). `hooks`
  within the sockets is 05.3's. `hook_http(l)` is
  `skein_http::server::Limits { head: hook_head_bytes, headers, body: hook_body_bytes, read: chunk.max(5), response: 256, send: 1 }`
  (the author checks `server::worst_case` is some); `hook_documents(l)` is `documents(l)` with `json.length =
  hook_body_bytes`. `worst_case` (:311) adds per connection the server, the tokenizer, the decoder as :329-334 prices it, an
  `Hmac`, a held hint and the socket as :335-338, times `hooks`, and `hook_secret_bytes` per forge.
- Machine: `Hooks` (`hooks.rs`) holds the listener's token, the waiting `Accepted` socket and a slab of `Hooked`
  (`hooks/connection.rs`): a `skein_http::server::Server` on the socket, the tokenizer, 04.2's decoder, an `Hmac` keyed by the
  forge's secret, a deadline, at most one held hint. `begin` issues `Listen`; `up` hands every event of a hooks' owner to `Hooks`;
  a `Failed` of the listener's token (a bind refused, or lost while serving) is `ListenerLost`; `drive_room` ends with `Closed`
  only once `Hooks` is closed. `hook_deadline` runs from accept or the last answer, in `next_deadline`; past it `fire` aborts the
  socket, with no answer.
- A delivery, in order: (1) `Call`: a target that is not `delivery::hook_forge`'s of a forge with a secret: `Discard`, `404`; a
  method other than `POST`: `Discard`, `405`; no `SIGNATURE_HEADER` of 64 hexadecimal digits: `Discard`, `403`. (2) The body is
  demanded in `read`-sized pieces, each into the `Hmac` and, if `decoder(EVENT_HEADER's value, ..)` is some, the tokenizer and
  `step`; a length past `hook_body_bytes` is skein-http's own `413` at the head. (3) At `End`, `sha256::equal` of the MAC in
  hexadecimal (`marker::hex`, `marker.rs:53`, takes one nibble: call it on `byte / 16` and `byte % 16` of each
  of the 32 bytes, as `marker.rs:23` does) and the header: wrong, `403`; right and `step` ended `Malformed`, `400`; right,
  `204`, and the hint if `step` ended in `Done` (an event of no row hints nothing). (4) `Respond` first; the hint goes up at
  `Done`. Every refusal ends the connection after its answer. The connection does not tell "not ready" from "no room": before the
  keep's restore `root::try_step` takes the hint (`Ok`, `admission.rs:46-52`) and `keep::hint` drops it (`keep.rs:642-646`, no
  live row), as forge.md 5 says; only `try_step`'s `Err` (no journal room) is held.
- The hint (`hooks/hint.rs`): `Issue`, `Comment` and `Review` give `Item(number)` with `key: marker::decode(body)` (`marker.rs:31`;
  a `Body::Long` has none); `Pull`, `Item(number)` with no key; `Push`, `BranchMoved { branch, head: after.bytes, actor: pusher }`,
  or `Branch(branch)` when `after` is all zero; `Ref`, `Branch(branch)`; `Job`, `Commit(commit.bytes)`. A tag (`Delivery::branch()`
  is none) or a repository id past `u32` hints nothing. The repository is `{ forge: n, repository }`.
- Boundary: `Event::Hint(Hinted)`, `Event::ListenerLost`, `Request::RefusedHint(Hinted)`. Routing: `forge::hinted(Hinted) -> root::Event`
  (`Event::Forge(Box::new(forge::Event::Hint { hint }))`); `forge_pass` takes a `Hint` to `root::try_step` and returns a refusal as
  `RefusedHint`; `ListenerLost` to `stop::hooks`, which begins the drain as `stop::forge` does, recording `Stop::Hooks` (new);
  `host.rs` words it `temper serve: webhooks listener failed`, exit 1.
Test shape: step tests (each < 20 ms) in `src/tests/hooks.rs` (the crate's `Harness`, `fixtures.rs:101`; `limits()` :5):
`a_hook_secret_past_its_bound_is_unusable_naming_the_forge`, `a_hook_secret_bound_below_32_is_unusable`,
`a_hook_limit_of_zero_is_unusable`, `a_hook_head_past_what_io_holds_is_unusable_as_a_stacked_server` (a smaller `io.intake`, named
beside the profile's), `the_worst_case_grows_with_the_webhook_connections`, `each_delivery_becomes_the_hint_forge_md_5_names` (a
row per `Delivery` variant). The world's `Listener` (`hooks.rs`) stands beside `Machine` (`machine.rs:48`): the real `Component`
with `Settings.webhooks`; `Listen` answered `Listening`; `accept()` makes an `Accepted` and answers its `Bind` with a
`skein_world::stream::Wire` peer (as `peer.rs:4`); `send`, `status`, `hints`, `refuse`, `close`, `lose`; a log by component entry,
so a hint is checked to come in no entry before its response bytes. Limits: `fixtures::limits()` (:6) with the five hook fields
from the profile's `forge_component`. New fixtures: `HOOK_SECRET`, `hook_limits()`, `hook_settings()`. Deliveries are 05.1's
`Service::delivery` over `fixtures::protocol_config()` (:141) with `hooks: Some(..)`. Reference test (ceiling 0.5 s):
```rust
#[test]
fn a_signed_push_is_answered_204_before_its_hint_goes_up() {
    let mut world = Listener::new(7, fixtures::hook_limits(), fixtures::hook_settings());
    let peer = world.accept();
    world.send(peer, &deliveries::push(b"main", 7)); // 05.1's Post: head, then body
    world.run_for(SECOND);
    assert_eq!(world.status(peer), Some(204));
    let [hinted] = world.hints() else { panic!("one hint") };
    let Change::BranchMoved { branch, actor, .. } = &hinted.hint.change else { panic!("a push hint") };
    assert_eq!((branch.as_ref(), *actor), (b"main".as_slice(), 2));
    assert!(world.hinted_after_answer(), "no hint in an entry before its 204's bytes");
}
```
Tests in `tests/hooks.rs` (each < 0.5 s): `a_wrong_signature_is_answered_403_and_hints_nothing`;
`a_body_past_its_cap_is_answered_413_and_hints_nothing`; `a_path_without_a_forge_and_secret_is_answered_404` (rows: forge 9,
a forge with none); `a_get_to_a_forges_path_is_answered_405`;
`a_verified_body_the_decoder_calls_malformed_is_answered_400`; `a_verified_event_of_no_row_is_answered_204_and_hints_nothing`;
`a_hint_the_root_refused_is_held_and_offered_again`; `a_connection_past_hooks_waits_until_one_closes`; `a_request_unfinished_at_its_deadline_is_closed_with_no_answer`; `a_failed_listener_is_told_to_the_root_as_lost`; `a_recorded_forgejo_push_signed_by_the_world_is_hinted` (04.1's `tools/forgejo-conformance/deliveries/push/request` body, signed
by the world); `a_domain_push_reaches_the_component_as_a_signed_delivery_and_a_hint` (joint: `Joint::hooked(seed)` sends the fake
domain's hooks through `Service::delivery` to a listener peer); `the_secret_appears_in_no_event_or_debug_text`
(`fixtures::token_absent`, :190); `two_deliveries_on_one_connection_are_both_answered_and_hinted`; `a_malformed_head_is_answered_4xx_and_closed` (skein-http's own); each refusal test also asserts the close. `tests/fuzzy_hooks.rs`, about 5 s: `a_delivery_split_at_every_point_is_answered_alike`.
`temper-service/src/tests/forge.rs`, beside :51: `a_hint_the_root_cannot_take_is_offered_again_next_pass`,
`a_lost_webhooks_listener_drains_the_service_and_stops_it`.
Proof: world, `cargo nextest run -p temper-engine-protocol-forge-world -E 'binary(=hooks)'`: scripted deliveries are
answered `204`, `400`, `403`, `404`, `405` or `413`, a good one is hinted after its answer, and a lost listener is told, in under
5 s; joint, the same binary's `a_domain_push_reaches_the_component_as_a_signed_delivery_and_a_hint`: the fake domain's push, signed
by 05.1's code, is read by the real component into one hint; and `cargo nextest run -p temper-engine-protocol-forge-world
--profile fuzzy -E 'binary(=fuzzy_hooks)'` in under 20 s. No process has a listener until 05.3, which runs the shipped binary.
Don't: no hint from an unverified body; no read of a body for a `404`, `405` or a missing signature (`Discard`); no secret
outside `Webhooks.secrets`; no external crate; no timer outside `next_deadline` and `fire`; no second held hint on a connection.
Size: at most 280 lines.
Removes: nothing.

### 05.3 `application.webhooks` and `forges[].webhook_secret`; `temper-service` hosts the listener
Also (ruling 135): `crates/temper/src/forge.rs:255` sets reconcile's `hook_secret_bytes` from
`profile.forge_component.hook_secret_bytes`, not `token_bytes`, beside `listener` (10-reconcile.md, Choice 8).

**Corrections against main (ruling 138, at temper 62ca62ade, after client-api's temper 05.1 3f8b3df6f and 05.2
3c0936cd0; they take precedence over the text below):**
- 07.1, 07.3, 08.1 and 10.1 are on main: 05.3 adds the setup's view and the wiring. `Named.forges[].webhook_secret`
  already carries each secret's path; only `Named.webhooks` is new.
- Files gain `crates/temper/tests/support/owned.rs:20-22` (a whole forge object: without `"webhook_secret": null`, 22 of
  the `forge` and `startup` binaries' 43 tests fail on `application.forges[0].webhook_secret: missing`),
  `crates/temper-service/src/tests/fixtures/forge.rs:134` (calls `forge::settings`),
  `crates/temper-forge-setup/src/tests/fixtures.rs:28,38` (two `check::Forge` literals), and
  `crates/temper-service/src/tests/fixtures/startup.rs:41` (`fixtures::files()`'s `with_capacity(8)` becomes 9).
  `git grep -l '"forges"' -- '*.rs'` lists 11 files, not 8: give each whole forge object `"webhook_secret": null`, and
  list in the commit body each further copy you change.
- `application::parse` (`application.rs:393`) is 99 lines, against clippy's bound of 100: move its workers loop into a
  function of its own before adding the members.
- Clippy's `manual_map` meets the disallowed `Option::map` in the `Option` reads: write them as `if let` assignments.
- An in-process name as `application.webhooks.listener` is refused by the address reader, as the clients' listener's is
  (`application.clients.listener: invalid form`, `src/tests/startup.rs:34`): configuration.md's row says "a socket
  address on loopback (in `127.0.0.0/8`, or `::1`), as the clients' listener's in standalone". `What::Loopback` is for a
  socket address off loopback; the Shape's "or an in-process name" goes.
- The reference test: `Post` is `support::webhooks`' request (path, body, event); `Post::signed` signs with
  `support::webhooks::SECRET`, the secret `webhooks::configuration` writes; `PING` is the event header's value `ping` (an
  event of no row); `Exit` is `skein_io::kernel::Exit`, as `support/mod.rs:75` imports it.
- Line references now: `application.rs` `Members` :11, `Forge` :43, `forge` :240, its literal :289, `parse` :393 with the
  `NotBuilt` push at :415, `files` :492, `resolve` :567, the `trust` read :147-156, `config::secret` :615; `checks.rs`
  `sockets` :254, `socket_count` :272, `process_io` :298; `startup.rs` `all_files` :126, `settings::Engine` :230, the
  `process_io` call :244; `problems.rs` the `NotBuilt` arm :179; `limits.rs` `profile` :249; `forge.rs` `problems` :205,
  `tag` :269, `indexed` :277; `src/tests/startup.rs` the test :78, `NotBuilt` at :88, 151, 154, 156; `src/tests/checks.rs`
  the sockets test near :157; `tests/fixtures.rs` the `settings::Engine` literal :84; `tests/one-process/src/fixtures.rs`
  :124-130; `support/mod.rs` `configuration` :21, `run_signal` :307-336; `forgejo.rs` edits the configuration at :142.
  `settings.rs:71`, `crates/temper/src/forge.rs:255`, `host.rs:57` and `names.rs` near :208 hold.

Scope: configuration.md 3, 6, 7: the two members read and checked (`null` for none, never missing), the secrets read at
start and handed to the component, the five limits' names and the listener's cross-checks, the listener's io tagged and
routed, and the shipped binary answering a delivery.
Needs: client-api's temper 05.1 and 05.2 on main (`server.rs`, `application.clients`): the ready check runs `git grep -n
'pub struct Server' -- crates/temper-service/src/server.rs` and `git grep -n 'pub struct Clients' --
crates/temper-service/src/startup/application.rs`, and checks the names below against what they land; temper 05.2.
Files (hotspots, README "Rules of this plan"; the checks after client-api 05.2):
- `startup/application.rs` (:12 `Members`, :21 `Forge`, :175 `forge`, :223 its literal, :327 `parse` with the `NotBuilt` push at
  :343, :417 `files`, :482 `resolve`), `startup/checks.rs` (:249 `sockets`, :267 `socket_count`, :293 `process_io`; the uniqueness
  check beside client-api's loopback check), `startup.rs` (:116 `all_files`, :216 `settings::Engine`), `startup/problems.rs` (:14, :74,
  :171), `settings.rs:71`, `limits.rs` (a `SOCKETS` constant beside :247), `forge.rs` (:21 `settings`, :195 `problems`, :249 `tag`),
  `limits/names.rs` (the arms beside :208);
- the documents that gain `"webhooks": null` (beside `application.clients`) and `"webhook_secret": null` in each forge (`git grep
  -l '"forges"' -- '*.rs'` lists 8 files; three write a whole document): the `APPLICATION` fixture
  `crates/temper-service/src/tests/fixtures/startup.rs:9-14`, `crates/temper/tests/support/configuration.rs:37-45`
  (`forgejo.rs:118-133` edits it), `tests/one-process/src/fixtures.rs:98-104`; the tests: `src/tests/{startup,checks,forge,limits}.rs`
  (`tests/fixtures.rs:82` has a `settings::Engine` literal), `crates/temper/tests/startup.rs`;
- the setup's view (whichever of 05.3 and 07.3 lands second adds it): `crates/temper-forge-setup/src/check.rs`, `startup.rs`
  (`Named`), `crates/temper/src/setup.rs`; the wiring (below), where 08.1 or 10.1 is on main: `crates/temper/src/forge.rs`; the
  process tier (hotspot, client-api to ~10-18 20Z): new `crates/temper/tests/support/webhooks.rs` (temper 11.1 extends it),
  `support/mod.rs:2-8`, `tests/process.rs`.
Shape (05.2's `Webhooks`, sketch 2, is what the component takes; here the reader's, beside client-api's `Clients`):
- Reading: `application::Webhooks { listener: config::Address }` in `Members.webhooks: Option<Webhooks>`; `Forge.webhook_secret:
  Option<Box<[u8]>>`, a path. Both are `read::required` (`read.rs:17`), then `null` is none, read as `trust` is
  (`application.rs:103-111`); a missing member is `What::Jig(config::What::Missing)` at its path. `files()` lists each secret
  (`private: true`, `max: members.limits.forge_component.hook_secret_bytes`, at `application.forges[i].webhook_secret`);
  `resolve` fills `settings::Engine.webhooks: Option<settings::Webhooks { listener, secrets }>`, a secret trimmed of its final
  newline by `config::secret`, as `application.rs:512` does.
- Checks, each by path: `application.webhooks.listener` off `127.0.0.0/8` and `::1`, or an in-process name: `What::Loopback`;
  equal to `application.clients.listener` at a nonzero port: `What::Jig(config::What::Unique)`; a `webhook_secret` while
  `webhooks` is none: `What::NoWebhooks` ("needs application.webhooks"), at the secret's path; `socket_count` adds `1 +
  forge_component.hooks` when `webhooks` is set. The sockets are temper's own, 48 (answers 30, 42): `limits::SOCKETS`, which
  `process_io` (`checks.rs:293`) gives `io.sockets` in place of the sum of jig's engine's and worker's, for `sockets`' check
  (:249) and for the io `startup.rs:229` hands `Service::new`; jig's `small` is untouched.
- The setup's view, by the names 07.1 and 07.3 give it (setup.md 8): `check::Forge` gains `webhook_secret`
  (`<directory>/webhook_secret` when `webhooks` is set), `Members.listeners` the webhooks' listener (an owned `listen` clashing
  with it is refused), `Named` carries `webhooks` and each secret's path, which the binary's `setup.rs` fills into the view.
- Limits: the five names in `limits/names.rs` (`hook_deadline_ms` by `duration`); `forge::problems`' and `forge::indexed`'s arms
  for the new `Unusable` and `Stacked` are 05.2's.
- Hosting: `forge::settings` (:21) gains a `webhooks` parameter and builds `kit::Settings.webhooks`; `forge::tag` (:249) tags
  `Listen { owner }` and `Bind { owner, .. }` with `tokens::FORGE` as it tags `Connect`; `Layer::up` untags (`accounts.rs:470`);
  `system.rs:503` already routes prefix-5 events. `crates/temper/src` is untouched but for the setup's view and the wiring.
- Wiring: the listener also reaches reconcile (`Settings.listener`, temper 10.1) and Forgejo's `ALLOWED_HOST_LIST` (`Prepare::new`'s
  `allowed_host`, temper 08.1), both in `crates/temper/src/forge.rs`: whichever of 05.3 and 10.1, and of 05.3 and 08.1, lands
  second wires it; temper 11.1 proves both.
- Process support: `webhooks::configuration(dir)` runs `support::configuration(dir, &Forge::Unreached)` (`support/mod.rs:19`),
  then sets `application.webhooks` to a free port and `webhook_secret` (`secrets/forge-1.webhook-secret`, 0o600);
  `webhooks::served(dir, ..)` drives `temper serve` as `run_signal` does (`support/mod.rs:304-333`), its referee making the posts
  on a std `TcpStream` once the link is printed, then signalling Terminate. Also `webhooks::free_address()` (port 0 bound and
  dropped) and `webhooks::signature(secret, body)` (`Hmac`, lowercase hexadecimal), under the names 11.1 uses.
Test shape: temper-service rows in the existing files (each < 20 ms), built as `src/tests/startup.rs:33-52` builds them
(`input.replace` on `fixtures::APPLICATION`) and run as `src/tests/checks.rs:6` runs them; limits from `limits::profile(Small)`
(`fixtures::engine()`), none other. Tests: `the_webhooks_listener_and_a_forges_secret_file_are_read`;
`a_null_webhooks_and_a_null_secret_read_as_none`; `a_missing_webhooks_or_webhook_secret_is_refused_naming_its_path` (the row
at :43 becomes this; the test is `an_unknown_missing_or_reserved_member_is_named_by_its_path`, `startup.rs:33`);
`a_webhook_secret_while_webhooks_is_null_is_refused_naming_its_path`;
`a_webhooks_listener_off_loopback_is_refused_naming_its_path`; `a_webhooks_listener_equal_to_the_clients_is_refused`;
`a_secret_file_past_hook_secret_bytes_is_refused_naming_it`; `the_hook_limits_have_dotted_overrides`;
`hook_connections_past_the_sockets_the_profile_leaves_are_refused` (a listener and `forge_component.hooks` 40: `engine.limits.io.sockets:
66 bytes, past its bound of 48`; the test at `checks.rs:182` takes `git_component.operations` 9: `49 ... of 48`);
`the_shipped_profile_admits_the_hook_limits_with_a_listener_and_a_secret` (`started` of `checks.rs:6` on `fixtures::document()`
with `webhooks` and a secret in `fixtures::files()`: no problem, so `forge::prepare`, the sockets and the memory sum all pass);
`forge_component_hook_limits_that_disagree_are_problems_naming_them` (rows beside `src/tests/forge.rs:88`). In
`crates/temper/tests/startup.rs`, as :21-35 does: `a_webhook_secret_readable_by_others_refuses_the_start_naming_it` (the exact
stderr line). If the setup's view is on main: `a_webhook_secret_off_the_forge_directory_is_refused_when_webhooks_is_set` in
`crates/temper-forge-setup/src/tests/members.rs`. Reference test, the process tier's new kind:
```rust
#[test]
fn temper_serve_answers_a_signed_delivery_204_and_an_unsigned_one_403() {
    let scratch = support::Scratch::new("webhooks");
    let listener = support::webhooks::configuration(scratch.path()); // "127.0.0.1:<free port>"
    let ended = support::webhooks::served(scratch.path(), &listener, &[
        Post::signed("/hooks/1", b"ping", PING), // an event of no row: verified, hints nothing, 204
        Post::unsigned("/hooks/1", b"ping", PING),
        Post::signed("/hooks/9", b"ping", PING), // PING signed in the test with skein_lib::sha256::Hmac
    ]);
    assert_eq!(ended.statuses, [204, 403, 404], "no forge 9");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "drained: {}", String::from_utf8_lossy(&ended.stderr));
    scratch.remove();
}
```
Proof: process, `sh -c 'cargo build -p temper --bin temper && NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper
--profile process -E "test(temper_serve_answers_a_signed_delivery_204_and_an_unsigned_one_403)"'`: the shipped binary on a
scratch deployment answers the signed delivery `204`, the unsigned one `403`, forge 9 `404`, and drains on a signal with exit 0,
in under 60 s (about 10 s). A hint is not observable here: the keep ignores a repository it does not hold
(`crates/temper-engine-domain-forge-client/src/keep.rs:622`, `hint`); 05.2's joint test shows a hint reaches the root, session 11's
hinted story its effect on an owned Forgejo.
Don't: no handling of `webhooks` in the binary but the setup's view and the wiring; no member beyond `listener` and `webhook_secret`; no
default listener; no secret in a diagnostic (problems name paths); `setup`, `user` as a file and the owned `listen` check are
07.1's, 07.3's and 10.1's.
Size: at most 250 lines.
Removes: `What::NotBuilt` with its words (`problems.rs:14,74,171`) and its uses in `src/tests/startup.rs:43,106,109,111`.

## Interface sketches

The fake's are 05.1's, the component's 05.2's.

```rust
// testing/temper-fake-forge-protocol/src/delivery.rs
use crate::{Limits, Service};
use alloc::boxed::Box;
use skein_lib::Secret;
use temper_fake_forge_domain as domain;
/// Where one forge's deliveries go and what signs them (forge.md, 5 and 9).
#[derive(Debug)]
pub struct Endpoint {
    pub forge: u16,      // the forge's number: the path `/hooks/<forge>`
    pub host: Box<[u8]>, // the listener's `Host` value
    pub secret: Secret,  // the hook's secret
}
/// One delivery as the HTTP request Forgejo's hook makes: the world writes `head`, then `body`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Post {
    pub head: Box<[u8]>, // request line, fields and the blank line
    pub body: Box<[u8]>, // the JSON payload, `Content-Length` long
}
impl Service {
    /// The signed delivery of a domain hook, or none: no hooks configured, a wiki, an unknown repository.
    #[must_use]
    pub fn delivery(&self, hook: &domain::Request, limits: &Limits) -> Option<Post> {
        todo!()
    }
}
/// `X-Forgejo-Signature`: the body's HMAC-SHA256 under the secret, lowercase hexadecimal.
#[must_use]
pub fn sign(secret: &Secret, body: &[u8]) -> [u8; 64] {
    todo!()
}
```

```rust
// crates/temper-engine-protocol-forge/src/hooks.rs
use skein_io::Addr;
use skein_lib::{List, Secret, Token};
use temper_engine_domain_forge_client::api;
/// The webhooks' listener and each forge's secret (forge.md, 5), in `Settings.webhooks`: present only with the listener.
#[derive(Debug)]
pub struct Webhooks {
    pub listener: Addr,                // a loopback socket address: plaintext HTTP
    pub secrets: List<(u16, Secret)>,  // a forge's number and secret; a forge with none has no path: `404`
}
/// A verified delivery's hint, up from the webhook connection that holds it until the root takes it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Hinted {
    pub connection: Token, // the component's token of the connection
    pub hint: api::Hint,   // what the delivery says (repository, change, key)
}
// boundary.rs gains Event::{Hint(Hinted), ListenerLost} and Request::RefusedHint(Hinted); limits.rs gains
// Limits.{hooks, hook_head_bytes, hook_body_bytes, hook_deadline, hook_secret_bytes}, Stacked::{ServerOverIo,
// JsonOverServer} and Unusable::{SecretBytes { bytes: u32 }, Secret { forge: u16 }, Listener}.
```

## Questions to raise when reached

None open. The sockets' constant (answers 30, 42) and the head's bound (answer 43) are settled by the design.

## Done when

1. `git -C ~/src/rust/skein merge-base --is-ancestor $(git -C ~/src/rust/skein log origin/main --all-match --grep='^Plan:
   forge-setup$' --grep='^Step: skein-01.2$' --format=%H -1) $(grep -o 'skein.git#[0-9a-f]*' Cargo.lock | sort -u | sed
   's/.*#//')` exits 0.
2. For N in 1, 2, 3: `git log origin/main --all-match --grep='^Plan: forge-setup$' --grep="^Step: temper-05\.$N$" --format=%h -1`
   prints a commit.
3. `git grep -n -w NotBuilt -- crates` and `git grep -n hmac_sha256 -- tools` print nothing; `git grep -n -c 'b"forge_component\.hook'
   -- crates/temper-service/src/limits/names.rs` prints the file and 5.
4. `git grep -c -E '"webhooks": ?null' -- crates/temper-service/src/tests/fixtures/startup.rs
   crates/temper/tests/support/configuration.rs tests/one-process/src/fixtures.rs` prints each file with 1, and so does the
   same with `'"webhook_secret": ?null'`.
5. `git grep -n -e '^sha2' -e '^hmac' -e '^subtle' -- crates/temper-engine-protocol-forge/Cargo.toml
   crates/temper-service/Cargo.toml testing/temper-fake-forge-protocol/Cargo.toml` prints nothing.
6. `cargo nextest run -p temper-engine-protocol-forge -p temper-engine-protocol-forge-world -p temper-fake-forge-protocol -p
   temper-service -E 'binary(=hooks) + test(hook) + test(hint) + test(deliver)'` passes.
7. README Done when 4's run (the process tier) covers 05.3's test, and the count of code-organization.md, 6, over each increment's paths, against the Sizes
   above, are in each landing's report.
