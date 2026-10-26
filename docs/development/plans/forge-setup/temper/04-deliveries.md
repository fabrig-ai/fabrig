# Session 04: the deliveries' facts and documents

Forgejo's webhook deliveries, recorded and then decoded. 04.1 records them on the throwaway Forgejo with the
conformance tool: the `[webhook]` host list, an org hook made as reconcile makes one, a std listener that holds each
delivery's answer, the `delivery-…` facts, and the raw deliveries in `tools/forgejo-conformance/deliveries/`. 04.2 puts
the documents in `temper-forge-forgejo`: `Delivery`, decoded by `X-Forgejo-Event` into the fields a hint reads (the
engine's side), encoded for the fake forge (the server's side), replayed on 04.1's recordings; with the listener's path
for a forge, `SUBSCRIBED` and the header names. Nothing here reaches the engine: temper 05.1 (the fake's signed
deliveries), 05.2 (the component) and 09.3 (reconcile's hook) use 04.2 by name.

## Read first

- Design: forge.md 5 and 13 (its last "To check" bullet); setup.md 10, 13.3 ("Org hooks", "The deliveries"), 6.2
  (`[webhook]`). decisions.md 5, 11, 13.
- Findings: webhooks.md 2, 4, 5; forge-and-tiers.md 1, 2, 3 ("A new group"); measurements.md 4 (the recording takes
  43 s and rewrites 34 transcripts).
- skein: programming-model.md 3, 6.2, 10.2; testing-strategy.md 2.1, 8. temper: workflow.md 2.
- Forgejo's sources, read-only, in `~/src/rust/tmp/forgejo17-notes/`: `hook.go` (payloads),
  `services_webhook_shared_payloader.go` (headers, signature), `deliver.go` (host list), `action.go` (job payload).

## What the design asks

- forge.md 5: the table of events, their headers, the signature, "Push payloads list commits, which the decoder skips",
  the listener's path `/hooks/<number>`: 04.1 records each row, 04.2 decodes the fields each hint reads.
- forge.md 13, setup.md 13.3: the deliveries as facts before the component relies on them (headers, fields, signature,
  the engine's own changes, a job's delivery against its status, a pull request's mergeability at its push's delivery,
  `mergeable` false while checked); the host list refuses loopback by default and allows it when named.
- setup.md 10: what Forgejo settles after it delivers is read again within `settle` (06.2), so the order recorded here
  is data for 06.

## Choices this plan makes

- **Deliveries live in `deliveries/<X-Forgejo-Event-Type>/{request,response}`** (`-tag` added for a tag's), beside
  `transcripts/`: tests/codecs replays every `transcripts/` directory as an API exchange
  (`tests/codecs/src/forgejo.rs:379`). `request` is Forgejo's delivery, masked; `response` the listener's answer.
- **The hook is an org hook** on the private org `hooked` (as reconcile makes an org on an owned forge, 02.1; its
  repository `hooked/specimen`), type `forgejo`, JSON, active, the table's events, a constant secret of the tool
  declared to `Secrets`, its URL `http://127.0.0.1:<listener port>/hooks/1`. It is provisioned through `Fixture::api`,
  not recorded (temper 02.2 records the hook routes).
- **Two instances,** each in a directory under the fixture's scratch (as `facts.rs:95` does): A, default settings,
  proves the refusal; B, with `webhook_hosts: Some("127.0.0.1")`, `max_response_items: Some(10)` (three jobs' statuses
  fit a page) and a runner, takes the scenario.
- **Probes run while the listener holds the answer** (Forgejo waits up to its `DELIVER_TIMEOUT`; a probe takes under
  2 s), so a read then is a read at the delivery. Settled may have settled a moment earlier; unsettled was certainly
  unsettled.
- **The signature is checked at record time** on the raw bytes by a tool-local `hmac_sha256` over skein-io's digest,
  a stand-in that temper 05.1 replaces with skein-lib's `Hmac`. 04.2 never verifies a signature.
- **The deliveries have a mask of their own,** written in scenario order, so no other group's captures renumber them
  and 04.2's ids are literals.
- **04.2 owns** the listener's path for a forge (`hook_target`, `hook_forge`), `SUBSCRIBED` and the two header names, in
  `temper_forge_forgejo::delivery`: reconcile, the component and the fake share one spelling of each.

## Rules that bind it

- `facts.txt` and `deliveries/` are generated. The recording rewrites 34 other transcripts: restore
  `tools/forgejo-conformance/transcripts` with `git checkout --` before committing. `Cargo.lock` changes by edges only;
  on a rebase conflict take origin/main's and build again.
- The Forgejo crate's `src/tests.rs` holds 639 lines: new tests go in `src/tests/delivery.rs`.
- Shared with other sessions (rebases take both): `decode/walk.rs` (`Kind`, `fields`, `element`) with temper 01.2, 01.3,
  02.1, 02.2; `tests/codecs/src/forgejo.rs` with 01.1; conformance `lib.rs` and `runner.rs`, the throwaway's `Settings`
  and `ini` with temper 01 to 03. None with client-api.

## Increments

### 04.1 The host list, the listener and the deliveries' facts
Scope: forge.md 13, setup.md 13.3; `webhook-deliveries` stops being unchecked.
Needs: nothing; `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` name the binaries.
Files: `testing/temper-throwaway-forgejo`: `src/lib.rs:9`, `src/instance.rs:261`, `src/tests.rs:6`.
`tools/forgejo-conformance`: `src/deliveries.rs` (new); `src/lib.rs:28-31` (call it; drop the Unchecked push);
`src/fixture.rs:20` (`Fixture::bare(forgejo, scratch, settings)`: instance, owner, writer, stranger; `new` calls it);
`src/runner.rs:198` (`start`, `Runner`, `command` `pub(crate)`; B's runner starts in B's directory, by B's fixture and,
before 03.3, B's repository prefix in place of `R`; 03.3 rewrites `start` to register by secret file, and whichever
lands second adapts); `src/mask.rs:11` (a second port, UUIDs); `src/write.rs:5` and `:65` (`SecretKind::HookSecret`;
`delivery` beside `case`, same scan); `src/tests.rs`; `README.md` (drop "left unchecked until webhooks are built");
generated `facts.txt` and `deliveries/`.
Shape:
- `Settings.webhook_hosts: Option<String>` (`None` in `small`); `ini` appends `[webhook]\nALLOWED_HOST_LIST = <hosts>\n`
  after `{cap}` for `Some`, nothing for `None`, so the golden at `tests.rs:6` holds.
- `deliveries::record(forgejo, scratch, out, secrets) -> Result<Vec<Fact>>`. A: `hooked` and one issue; no delivery
  within 3 s. B: `hooked` and the steps; each step names the hook event types it must deliver, awaited at most 10 s.
  Cases are written after, in scenario order, the first of each type.
- Steps (O is `fixture`, W is `writer`, acting as the engine does): 1 O opens an issue, marker body: `issues`. 2 W
  assigns it, O labels it, O sets its milestone: `issue_assign`, `issue_label`, `issue_milestone`. 3 W comments, marker
  body: `issue_comment`. 4 O branches `topic` and commits: `create`, `push`. 5 O opens a pull: `pull_request`; W
  assigns, labels, milestones it: `pull_request_assign`, `_label`, `_milestone`; W comments: `pull_request_comment`; O
  requests W: `pull_request_review_request`. 6 W reviews approved, request-changes, comment:
  `pull_request_review_approved`, `_rejected`, `_comment`. 7 W commits to `topic`: `push`, `pull_request_sync` (probe
  B). 8 O commits to `main`: `push` (probe B). 9 O makes and deletes branch `gone` and tag `v1`: `create`, `delete`. 10
  O commits the workflow `.forgejo/workflows/deliveries.yaml` to branch `ci` (jobs `pass`, `fail`, `after` with
  `needs: [fail]`; `on: push` of `ci`; `runs-on: temper-host`; and `held`, `runs-on: temper-held`, which no runner
  serves): `create`, `push`, `workflow_job_success`, `_failure`, `_skipped` (probe A); once these are had, O cancels the
  run, `POST R/actions/runs/{job.run_id}/cancel`, `204` (forge.md 13): `workflow_job_cancelled` (`held`). 11 one more
  issue, answered 500: no second delivery of it within 3 s.
- Probe A reads the combined status of `run.commit_sha` at a job end; probe B reads the pull at step 7's and 8's
  `push`. The answer follows the probe.
- Facts, 22, each named `delivery-` + `issues-row`, `pull-requests-row`, `comments-row`, `reviews-row`, `push-row`,
  `create-delete-row`, `job-ends-row` (each: every event of the row, the cancelled job end aside, its `X-Forgejo-Event`
  and `-Type`, and the paths named by the doc line of its `Delivery` variant in 04.2's sketch, typed as 04.2 reads them);
  `carries-the-repository-id`; `subscribed-events-read-back`; `signature-is-hmac-sha256-hex` (`X-Forgejo-Signature`,
  and `X-Hub-Signature-256` with `sha256=`); `refs-are-full-and-tags-are-told-apart`; `branch-deletion-sends-no-push`
  (may fail); `cancelled-job-end-is-delivered` (may fail; as `job-ends-row` for `workflow_job_cancelled`);
  `echoes-the-actors-own-writes` (`sender.id`, `pusher.id` are W's); `loopback-is-refused-by-default` (A);
  `loopback-is-allowed-when-named` (B); `failed-delivery-is-not-sent-again`; `pull-reads-not-mergeable-while-checked`.
  Pairs, exactly one of each holds: `job-end-follows-its-commit-status` or `job-end-precedes-its-commit-status` (status
  final at the job end, both jobs); `pull-is-settled-when-its-push-is-delivered` (every probe B, steps 7 and 8, read it
  mergeable) or `pull-is-checking-when-its-push-is-delivered` (one read it not mergeable at least, and each such pull,
  read again once the delivery is answered at 1 s, doubling, read mergeable within 30 s, the small profile's `settle`);
  each fact's line keeps probe B's reads. `webhook-deliveries` holds when every other fact holds,
  a pair by one of its two, and `branch-deletion-sends-no-push` either way.
- `Listener::start() -> io::Result<Listener>` (field `port`), `next(within) -> Option<(Received, Reply)>`,
  `Reply::answer(status)` closing the connection, `Received { head, body }` unmasked, a thread per connection; the mask
  numbers a delivery UUID and maps the listener's port to `127.0.0.1:3001`; `write::delivery` recomputes
  `Content-Length` once masked. `hmac_sha256(key, message) -> Option<[u8; 32]>` (key at most 64 bytes; the digest of
  the outer pad and of the digest of the inner pad and the message, by skein-io's one-shot `digest`) is transitional:
  temper 05.1 removes it for skein-lib's `Hmac`.
Test shape: `tools/forgejo-conformance/src/tests.rs` (mask test `:8`, `Exchange` literals `:31`), the throwaway's
`src/tests.rs:6`. Limits: Forgejo's `Settings::small()` (`lib.rs:18`); no temper limit applies; waits are above.
Tests: `the_app_ini_allows_the_webhook_host_it_is_given`; `hmac_sha256_reads_rfc_4231_case_2`;
`a_signature_holds_only_for_the_body_it_signs`; `masking_also_rewrites_the_listeners_port_and_delivery_ids`;
`a_masked_delivery_keeps_a_content_length_that_matches_its_body`;
`the_listener_holds_its_answer_until_the_probe_replies`; `the_roll_up_ignores_the_losers_of_the_pairs`. Ceilings: unit
20 ms; the listener's, on real sockets, 0.5 s. Reference (new kind):
```rust
#[test]
fn the_listener_holds_its_answer_until_the_probe_replies() {
    let listener = Listener::start().expect("listener");
    let mut client = TcpStream::connect(("127.0.0.1", listener.port)).expect("connect");
    client.write_all(b"POST /hooks/1 HTTP/1.1\r\nContent-Length: 4\r\n\r\nbody").expect("sent");
    let (received, reply) = listener.next(Duration::from_secs(5)).expect("a delivery");
    assert_eq!(received.body, b"body");
    client.set_read_timeout(Some(Duration::from_millis(100))).expect("timeout");
    assert!(client.read(&mut [0; 1]).is_err(), "nothing is answered before the reply");
    reply.answer(204);
    client.set_read_timeout(Some(Duration::from_secs(5))).expect("timeout");
    let mut answer = String::new();
    client.read_to_string(&mut answer).expect("answer");
    assert!(answer.starts_with("HTTP/1.1 204 No Content\r\n"), "{answer}");
}
```
Proof: process, the conformance recording whole, with both binaries in the environment (the real Forgejo and runner
deliver to the listener):
```sh
export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
export TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
sh -c 'cargo run -p forgejo-conformance -- --scratch target/forgejo-conformance --out tools/forgejo-conformance && cargo run -p forgejo-conformance -- --component --scratch target/forgejo-conformance --out tools/forgejo-conformance'
```
`facts.txt` and `deliveries/` then show Done when 2 to 4. Bound 150 s alone (43 s at the base, plus about 30 s for two
instances and the jobs). No live layer exists in this plan.
Don't: change a case of `transcripts/`, record the hook's creation, wait by a fixed sleep (every wait is a deadline on
an event), put the secret in a case, add a dependency.
Size: 300. Removes: the Unchecked fact and the README sentence.

### 04.2 The deliveries' documents (codec)
Scope: forge.md 5's table as a document, both sides; the listener's path for a forge, `SUBSCRIBED` and the headers.
Needs: 04.1's `deliveries/` on main (same session); the replay reads it.
Files: `crates/temper-forge-forgejo`: `src/delivery.rs`, `src/client/delivery.rs`, `src/server/delivery.rs`,
`src/tests/delivery.rs` (new); `src/lib.rs:5`, `src/client.rs:12`, `src/server.rs:12`, `src/tests.rs:1` (the modules);
`src/decode/walk.rs:27` (`Kind::Delivery(DeliveryKind)`), `:64` `fields`, `:165` `element`; `src/tests/fixtures.rs:61`
(the loop of `drive` becomes a `pump` over an event closure, as `drive` at `tests/codecs/src/forgejo.rs:124` is; `drive`
and a new `deliver` call it).
`tests/codecs`: `src/forgejo.rs` (`delivery`, `delivery_cases`, `decode_delivery` -> `Result<Delivery, Malformed>`),
`tests/forgejo_deliveries.rs` (new), `tests/fuzzy_forgejo.rs`, `Cargo.toml` (dev-dependencies `temper-service`,
`temper-engine-protocol-forge`).
Shape (sketches below):
- `DeliveryKind` is one field table per object entered, so a scalar knows its parent: a top object per family and `Repo`
  {`id`}, `Pusher` {`id`}, `IssueBody` {`body`}, `IssueNumber` {`number`}, `CommentBody` {`body`}, `ReviewContent`
  {`content`}, `Run` {`repository`, `commit_sha`}. Unknown fields are skipped, `commits` included; a missing or repeated
  key, a null, a wrong type or a string past `string_bytes` is `Malformed`, as the other families (`walk.rs:259`). A
  body past `body_bytes` is `Body::Long` and the delivery stands. Its own `Step`: a `Document::Delivery` would touch the
  component's exhaustive matches (`actions.rs:316`, `reads.rs:640`).
- `decoder` is `None` for an event with no row (the component verifies the body, decodes nothing, answers `204`);
  `Step::Malformed` is the component's `400` with no hint (forge.md 5). Header values: `issues`, `pull_request`,
  `issue_comment`, `pull_request_approved` (Approved), `pull_request_rejected` (RequestChanges),
  `pull_request_comment` (Comment, the review's), `push`, `create`, `delete`, `workflow_job_{success,failure,
  cancelled,skipped}`; one private table maps header and variant both ways.
- `branch()`: a push's `ref` without `refs/heads/`, or a create/delete's branch by `ref_type`; a tag gives `None`. The
  key is not decoded here: 05.2 calls `marker::decode` on `body`. `encode` writes the same fields, compact, and refuses
  a `Body::Long` (`writer::Refusal::TooLong`, as `server.rs:111`).
- `hook_target(forge)` is `/hooks/<number>`, the one spelling of the listener's path: the fake writes it as its request
  target, reconcile as its hook URL's path (09.3). `hook_forge(target)` is the component's read of a request's, and
  names a forge for that exact form alone: canonical decimal within `u16` (`id::unsigned`, then `Decimal::of` back), no
  query, sign, leading zero or trailing slash.
Test shape: unit tests in `src/tests/delivery.rs` over `fixtures::limits` and `small` (`fixtures.rs:12,22`) and
`deliver` beside `decode` (`:55`); replay in `tests/codecs/tests/forgejo_deliveries.rs` over `forgejo::delivery` and
`delivery_cases`, as `forgejo_start.rs:73`. Limits: the replay file's `shipped()` is the profile's builder,
`limits::documents(&temper_service::limits::profile(Profile::Small).forge_component)`
(`crates/temper-engine-protocol-forge/src/limits.rs:281`, `crates/temper-service/src/limits.rs:247`, `forge.rs:110`);
the unit tests use `fixtures::limits()` (string 1024, body 65 536, hand-set, above the profile's 512 and 16 384) and
`small()` (8, 16) for the cap tests, which need values a short body passes. Tests, unit:
`each_event_header_selects_the_delivery_it_decodes_to`; `a_push_skips_its_commits_and_keeps_ref_after_and_pusher`;
`a_delivery_missing_a_field_is_malformed_naming_it`; `a_repeated_repository_id_is_refused`;
`a_body_past_body_bytes_reads_long_and_the_delivery_stands`; `every_delivery_encodes_and_decodes_unchanged`;
`a_hook_target_reads_back_to_the_forge_it_names`; `a_target_naming_no_forge_reads_as_none`. Replay:
`recorded_deliveries_decode_to_the_fields_their_hints_read`;
`the_encoders_bodies_decode_unchanged_with_the_recordings_shape`;
`every_subscribed_event_has_a_recorded_delivery_the_decoder_knows` (while `delivery-cancelled-job-end-is-delivered`
fails, it names `workflow_job_cancelled` as the one unrecorded, kept by forge.md 5). Fuzzy (about 5 s):
`forgejo_delivery_decoders_answer_every_mutated_delivery_without_panicking`. Ceilings: unit 20 ms; replay 20 ms a case.
Reference (new kind: a recording):
```rust
#[test]
fn recorded_deliveries_decode_to_the_fields_their_hints_read() {
    for (case, expected) in forgejo::delivery_cases() {
        let sent = forgejo::delivery(case);
        let decoded = forgejo::decode_delivery(&sent.event, &sent.body, &shipped());
        assert_eq!(decoded, Ok(expected), "{case}");
    }
}
```
`delivery_cases()` is a table of literals written from the recordings (ids, numbers, masked shas as `forgejo::sha(n)`,
marker bodies).
Proof: unit, `cargo nextest run -p temper-forge-forgejo -p temper-codecs-world -E 'test(/delivery/) | binary(forgejo_deliveries)'`
(11 tests): 04.1's recordings decode to the fields of their hints, the encoder's bodies decode unchanged and the hook
path reads back; under 2 s. No layer above: nothing calls the documents until temper 05.2 and 05.1 connect them.
Don't: add a `Document` variant or a `Hint`; verify a signature or compute SHA-256; decode a marker; name an event or
spell the hook path in two places; read `trigger_user` or a job's `id`, `run_id`, `attempt`.
Size: 220 (codec). Removes: nothing.

## Interface sketches

```rust
// crates/temper-forge-forgejo/src/delivery.rs
use crate::{document::Body, id::ObjectId, request::ReviewEvent};
use alloc::boxed::Box;

/// A job's end: the `workflow_job_<end>` event.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum JobEnd {
    /// `success`.
    Success,
    /// `failure`.
    Failure,
    /// `cancelled`.
    Cancelled,
    /// `skipped`.
    Skipped,
}
/// A ref's `ref_type`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RefType {
    /// `branch`.
    Branch,
    /// `tag`.
    Tag,
}
/// The fields a hint reads: decoded by the engine's side, encoded by the fake's.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Delivery {
    /// `issues`: `number`, `issue.body`.
    Issue { repository: u64, number: u64, body: Body },
    /// `pull_request`: `number`; its key is its branches.
    Pull { repository: u64, number: u64 },
    /// `issue_comment`: `issue.number`, `comment.body`.
    Comment { repository: u64, number: u64, body: Body },
    /// A review: `number`, `review.content`; `event` says which.
    Review { event: ReviewEvent, repository: u64, number: u64, body: Body },
    /// `push`: the full `ref`, the new head, the pusher's id.
    Push { repository: u64, reference: Box<[u8]>, after: ObjectId, pusher: u64 },
    /// `create` (`created`) or `delete`: `ref`, `ref_type`.
    Ref { created: bool, repository: u64, reference: Box<[u8]>, ref_type: RefType },
    /// A job's end: `run.repository.id`, `run.commit_sha`.
    Job { end: JobEnd, repository: u64, commit: ObjectId },
}
impl Delivery {
    /// The branch a push, create or delete names; none for a tag or another event.
    #[must_use]
    pub fn branch(&self) -> Option<&[u8]> {
        todo!()
    }
}
/// The hook events a hook subscribes to: every event the table of forge.md 5 maps to a hint.
pub const SUBSCRIBED: [&[u8]; 22] = [
    b"issues", b"issue_assign", b"issue_label", b"issue_milestone", b"pull_request", b"pull_request_assign",
    b"pull_request_label", b"pull_request_milestone", b"pull_request_sync", b"pull_request_review_request",
    b"issue_comment", b"pull_request_comment", b"pull_request_review_approved", b"pull_request_review_rejected",
    b"pull_request_review_comment", b"push", b"create", b"delete", b"workflow_job_success", b"workflow_job_failure",
    b"workflow_job_cancelled", b"workflow_job_skipped",
];
/// The request header naming the event.
pub const EVENT_HEADER: &[u8] = b"X-Forgejo-Event";
/// The request header holding the body's HMAC-SHA256 in hexadecimal.
pub const SIGNATURE_HEADER: &[u8] = b"X-Forgejo-Signature";
/// The listener's path for a forge's deliveries: `/hooks/<number>` (forge.md 5).
#[must_use]
pub fn hook_target(_forge: u16) -> Box<[u8]> {
    todo!()
}
/// The forge a request target names; none for any other target.
#[must_use]
pub fn hook_forge(_target: &[u8]) -> Option<u16> {
    todo!()
}
```

```rust
// crates/temper-forge-forgejo/src/client/delivery.rs
use crate::{decode::Malformed, delivery::Delivery, limits::Limits};
use skein_json::tokenizer;

/// A delivery's decoder; its state is private.
#[derive(Debug)]
pub struct Decoder(());
/// One tokenizer event in, the next demand or the terminal out.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    /// The next tokenizer demand.
    Demand(tokenizer::Request),
    /// The body read whole; believed only once the signature holds.
    Done(Delivery),
    /// Not a delivery of its event; terminal.
    Malformed(Malformed),
}
/// `None` for an `X-Forgejo-Event` with no row of the table: verify, do not decode.
#[must_use]
pub fn decoder(_event: &[u8], _limits: &Limits) -> Option<Decoder> {
    todo!()
}
/// The first demand.
#[must_use]
pub fn first(_decoder: &Decoder) -> tokenizer::Request {
    todo!()
}
/// Advances by one event.
pub fn step(_decoder: &mut Decoder, _limits: &Limits, _event: &tokenizer::Event) -> Step {
    todo!()
}
```

```rust
// crates/temper-forge-forgejo/src/server/delivery.rs
use crate::{delivery::Delivery, limits::Limits};
use alloc::boxed::Box;
use skein_json::writer;

/// What the fake sends: the `X-Forgejo-Event` value and the JSON body to sign.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Sent {
    pub event: &'static [u8],
    pub body: Box<[u8]>,
}
/// Writes a delivery; a `Body::Long` is refused.
pub fn encode(_delivery: &Delivery, _limits: &Limits) -> Result<Sent, writer::Refusal> {
    todo!()
}
```

## Questions to raise when reached

1. Answered by the designer (2026-10-16): `skipped` comes from a job whose need fails, `cancelled` from cancelling
   the run of `held` by the API (forge.md 13). If it delivers nothing, `SUBSCRIBED` keeps it, unrecorded (forge.md 5).
2. Answered by the designer (2026-10-16): the pull pair as the Facts say, Forgejo's order varying (forge.md 13); the
   fake always checks after a move (forge.md 9; 06.1), and `settle` needs nothing more (domain/forge.md 5.2).
3. If `branch-deletion-sends-no-push` fails, the document gains `Delivery::deleted()` (a push whose `after` is all
   zero), and the designer confirms that a deleted branch's hint then comes from that push (forge.md 5).
4. If a create's or delete's `ref` is short, `refs-are-full-and-tags-are-told-apart` fails and `branch()` reads
   `ref_type` alone.

## Done when

```sh
cargo nextest run -p temper-throwaway-forgejo -p forgejo-conformance -p temper-forge-forgejo -p temper-codecs-world
cargo nextest run -p temper-codecs-world --profile fuzzy
```
Both exit 0. After 04.1's Proof command:
```sh
F=tools/forgejo-conformance/facts.txt
grep -c '^unchecked' $F                                    # 0
grep -x 'holds webhook-deliveries' $F                      # the line
grep -c '^holds delivery-' $F                              # 22 minus the next count
grep -c '^fails delivery-' $F                              # 2, plus one for each of the next line's last two failing
grep '^fails delivery-' $F | grep -v -e 'job-end-\(follows\|precedes\)-its-commit-status' -e 'pull-is-\(settled\|checking\)-when-its-push-is-delivered' -e 'branch-deletion-sends-no-push' -e 'cancelled-job-end-is-delivered'   # nothing
grep -c -x -e 'holds delivery-job-end-follows-its-commit-status' -e 'holds delivery-job-end-precedes-its-commit-status' $F   # 1
grep -c -x -e 'holds delivery-pull-is-settled-when-its-push-is-delivered' -e 'holds delivery-pull-is-checking-when-its-push-is-delivered' $F   # 1
ls tools/forgejo-conformance/deliveries | wc -l            # at least 21
grep -c 'left unchecked until webhooks' tools/forgejo-conformance/README.md   # 0
grep -c 'pub fn hook_target\|pub fn hook_forge' crates/temper-forge-forgejo/src/delivery.rs   # 2
git checkout -- tools/forgejo-conformance/transcripts      # the recording rewrote 34
B=$(git log --all-match --grep='^Plan: forge-setup$' --format=%H | tail -1)^   # the parent of the plan's first commit
git diff $B --stat -- tools/forgejo-conformance/transcripts   # nothing
git diff $B --name-only -- crates/temper-forge-forgejo tools/forgejo-conformance testing/temper-throwaway-forgejo tests/codecs | grep '\.rs$' | xargs -r grep -nE 'actual|genuine|legacy|compat|workaround'   # nothing
git diff $B --name-only --diff-filter=A -- crates/temper-forge-forgejo tools/forgejo-conformance testing/temper-throwaway-forgejo tests/codecs | grep '\.rs$' | xargs -r wc -l | awk '$2 != "total" && $1 > 800'   # nothing
```
Each landing's report gives its hand-written production lines by code-organization.md's command over its Files, against
Size 300 and 220 (codec).
