+++
id = "exchange-gone"
part = "exchange-gone"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-engine-clients/src/boundary.rs", "jig/crates/jig-protocol-engine-clients/src/component.rs", "jig/crates/jig-protocol-engine-clients/src/tests/public.rs", "jig/crates/jig-protocol-engine-clients-http/src/call.rs", "jig/crates/jig-protocol-engine-clients-http/src/lib.rs", "jig/crates/jig-protocol-engine-clients-http/src/tests.rs", "jig/crates/jig-protocol-engine-clients-http/src/tests/public.rs", "jig/testing/jig-engine-clients-kit/src/peer.rs", "jig/testing/jig-engine-clients-kit/src/peer/**", "jig/tests/engine-clients/tests/suite/gone.rs", "jig/tests/engine-clients/tests/suite/main.rs", "jig/docs/design/protocol/client.status.md"]
effort = "medium"
minutes = 110
+++

## Goal

jig's engine end tells the clients' component when an exchange's call
ends before its reply, `FromTransport::ExchangeGone`, as it tells a
stream's end, `StreamGone`. The component lets that caller's connection
go and keeps the exchange's pending work, so a close no longer waits on
a request whose caller has gone, and temper's drain reaches
`End::Drained` (the request's red-81).

Acceptance:

- `jig-protocol-engine-clients`'s `FromTransport` has
  `ExchangeGone { exchange: Token }`. The component handles it as it
  handles `StreamGone`, by one private method both arms call, holding
  today's `StreamGone` body (`component.rs`, `transport`): the public
  read's correlation released, the connection removed, a stream's
  connection marked closed. `pending.exchanges` and `pending.carried`
  are left alone: the core's or the application's answer, when it
  comes, settles as it does today for a lost reply.
- `jig-protocol-engine-clients-http`'s `Call::finish` pushes
  `Request::Component(FromTransport::ExchangeGone { exchange: self.name })`,
  before its `Http { .. Next }`, for a call that handed the component an
  `Exchange` or a `PublicRead` and has had no `Reply` or `Public` from
  it. A stream's call still says `StreamGone`, and nothing else; a call
  answered, or ended before it handed anything over, says nothing.
- A new world file, `jig/tests/engine-clients/tests/suite/gone.rs`
  (package `jig-engine-clients-world`, binary `suite`), holds the red,
  and it fails on the base before the fix (step 1):
  `a_close_does_not_wait_for_a_family_request_whose_caller_has_gone`.
  For each order `[leave, Stop, Close]`, `[Stop, leave, Close]` and
  `[Stop, Close, leave]`: a signed-in peer, with family 1 served
  (`settings.served` as `requests.rs`'s
  `a_carried_value_is_kept_before_the_application_sees_its_request`
  sets it), posts a family keyed request with no value; the test takes
  its `ToApplication::Request` and never answers it (the component's
  `Waiting::Ask`); "leave" is the peer's connection cut
  (`Peer::cut_exchange`). After the last step, `peer.closed() == 1` and
  `peer.answered(socket) == None`. In `[Stop, Close, leave]`,
  `peer.closed() == 0` before the leave: the close still waits for a
  caller who is there.
- In the same file,
  `an_answer_that_comes_after_its_caller_has_gone_settles_with_no_reply`:
  a family request carrying a value (`requests.rs`'s `carrying_request`),
  the value kept (`keep_carried`), its application request taken, the
  peer's connection cut, then the application's `Answered`: no reply is
  written, the carried value is kept as used (`peer.carried(number)`),
  and a `Close` then says closed once.
- `jig-protocol-engine-clients-http`, `src/tests.rs`: a new test,
  `a_call_ended_before_its_reply_says_its_exchange_gone_once`, over the
  rows: an exchange handed and closed (`http::Event::Closed`) says
  exactly `[ExchangeGone { exchange }]`; one handed, answered by
  `fixtures::reply` and then ended says none; a call closed before its
  body's end says none; a stream's call closed says `StreamGone` and no
  `ExchangeGone`; a public read (`fixtures::public_mount`) handed and
  closed says `[ExchangeGone { exchange }]`.
- The kit's `Peer::public_gone` (`jig-engine-clients-kit`,
  `src/peer/public.rs`) and the component's unit test at
  `src/tests/public.rs`, which sends a public read's exchange gone, send
  `ExchangeGone`, the message the engine end now says for it.
- The **Not yet** entry `4.2, 14 and 17.4: ExchangeGone: ...` in
  `jig/docs/design/protocol/client.status.md` is removed whole; nothing
  else in that file changes.
- The readers' focused tests pass (step 5), and the red's story runs as
  step 6 says.

## Inputs

- `temper/docs/plan/red-81-drain/notes/common.md`: every rule there
  holds.
- `jig/docs/design/protocol/client.md`, 4.2 (the messages, *One reply
  per exchange* and *A caller gone lets its exchange go*), 14 (*The
  component's world*, *The default mapping's ends*) and 17.4 (*A peer's
  end is the call's end*). The cut wrote these; build them.
- `jig/docs/design/protocol/service.md`, 5 (*A shutdown drains*), for
  the drain the component's close is part of.
- `temper/crates/temper-service/src/stop.rs`, `advance`: the gates
  `engine.clients.closed` holds. temper's code doesn't change: its stop
  sequence already sends the component `Stop`, then `Close`, and waits
  for `Closed`, and its `Clients::down` sends nothing after `Closed`.

## Owns and may change

- Owns: the files in the header.
- Readers and neighbours, call sites and expectations only:
  - A test in a jig package or in `temper-service` that asserts the
    exact transport messages after a call that ended unanswered may
    gain the `ExchangeGone`, or lose a reply it now never sees; change
    only that expectation, and name each in the report. A test whose
    rule the change breaks otherwise is a red to report, not to edit.
  - `jig/tests/engine-clients/tests/suite/requests.rs`: `carrying_request`
    may become `pub(super)` so `gone.rs` uses it; nothing else there.
  - A harness that hands its component a transport message after
    `Closed` (the component's assert, "the owner sends nothing after
    Closed", fires), now that a call ending after the close may say
    `ExchangeGone`: guard it as `temper-service`'s `Clients::down` and
    the engine-clients kit's `mount_step` do (`if !closed`), one line,
    and name it. Likely place: `jig/testing/jig-client-protocol-kit/src/engine.rs`,
    `event`.
- Options, each pre-approved:
  - The peer's orderly end. temper's server ends a connection whose peer
    ends during a call (`PeerEnd::Connection`,
    `temper/crates/temper-service/src/limits.rs`); the kit's server
    keeps the call (`PeerEnd::Call`). If the kit can deliver a peer's
    end (`skein_io::Event::Stream { owner, up: Up::End }`) to a server
    built with `PeerEnd::Connection` (`Peer::signed_in_with_server_bounds`)
    without changing its other exchanges, add `Peer::end_exchange(socket)`
    and run the red test's three orders for both leaves. Otherwise keep
    the cut alone, and say why in the report.
  - If the base's test already passes in some order (it should not:
    `closing.rs`'s `settled` waits on every connection), report the
    order and what let it close; keep the test.
  - The component's `admitted()` and `active()` keep counting a pending
    exchange whose caller has gone until its answer comes: they bound
    work in the core. Don't change them.

## Steps

1. Write `gone.rs` and its `mod gone;` first. Run the red test alone on
   the base (`heavy cargo nextest run -p jig-engine-clients-world --test
   suite -E 'test(=gone::a_close_does_not_wait_for_a_family_request_whose_caller_has_gone)'`,
   count 1) and see it fail, `closed()` 0. Quote the failure.
2. `ExchangeGone` in `boundary.rs` (a doc line as `StreamGone`'s
   neighbours have, citing client.md, 4.2), and its arm in
   `component.rs`.
3. The engine end: a `Call` field recording that the component owes the
   call a reply (set where `call.rs` pushes `Exchange` or `PublicRead`,
   cleared where `lib.rs`'s `component` takes a `Reply` or `Public`),
   and `finish` saying `ExchangeGone` while it is set. Name the field
   for what it holds; the call keeps its scoped `#[expect]` on bools.
4. The mount's test, the kit's `public_gone`, the unit test, then the
   red again: it passes. Break the rule once: drop the `ExchangeGone`
   push in `finish`, see the red test and the mount's test fail for it,
   and restore. Run
   `sign_in::a_lost_links_credential_check_spends_the_link_like_a_lost_core_answer`
   and `closing::close_keeps_an_exchange_until_its_released_answer_is_written`
   alone: both still pass, unchanged (a link spent after its caller has
   gone; a close waiting for a caller still there).
5. Once: `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-protocol-engine-clients -p
   jig-protocol-engine-clients-http -p jig-engine-clients-kit -p
   jig-engine-clients-world --all-targets -- -D warnings`. The mount's
   messages change meaning, so its readers' focused tests run:
   `heavy cargo nextest run -p jig-protocol-engine-clients -p
   jig-protocol-engine-clients-http -p jig-engine-clients-world -p
   jig-test-service -p jig-client-protocol-world -p jig-ops-world -p
   temper-service`. Check `temper` compiles: `heavy cargo check -p
   temper --tests`.
6. The red's story, by name and alone, three times, once the branch is
   committed: `heavy cargo nextest run -p temper --profile
   process-temper --test process -E
   'test(=temper_open_adopts_a_repository_by_commands)'`. It failed
   about once in 20 runs, so three passes are a smoke check; the world
   test is the proof. Report each run's result and time. A failure is
   read from its log first: if it hangs in the drain again, report it as
   the first line of `## Left` with the captured lines; if the builder
   can't start the story (its Forgejo, say), report it unrun and why. Run
   no other process test.
7. Commit (`jig: say an exchange gone when its call ends unanswered`,
   saying why: red-81's drain), run the citations check
   (`notes/common.md`), push.

## Report

The implementer's report, with: the red test's failure on the base and
its pass after; each test added or changed and the rule each bites (the
break of step 4 and what failed); each reader expectation changed,
by file and test; which leaves the red test covers; the story's three
runs; any option taken.
