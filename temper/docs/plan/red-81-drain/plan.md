# red-81-drain: the plan

temper's service sometimes never finishes draining after the terminal
leaves (`inputs/request.md`). The terminal had a family request admitted
and unanswered, and the clients' component, closing, waits for that
exchange's connection. Nothing ever tells the component the caller has
gone: jig's engine end, `jig-protocol-engine-clients-http`'s
`Call::finish`, says a stream's end (`StreamGone`) and nothing for an
exchange's. One task, in jig, with no task in temper: temper's stop
sequence already waits for the component's `Closed`, and gets it once
the component lets the caller go.

The cut wrote the design first:

- `jig/docs/design/protocol/client.md`, 4.2: the message
  `ExchangeGone { exchange }`, and the bullet *A caller gone lets its
  exchange go*: the component lets the caller go and keeps the
  exchange's work, and a close never waits for a caller gone.
- 14: the component's world and the mapping's ends gain their rows.
- 17.4: *A peer's end is the call's end* names `ExchangeGone` beside
  `StreamGone`.
- `jig/docs/design/protocol/client.status.md`: one **Not yet** entry,
  on its own bullet, which the task removes.

## Parts and tasks

| Part | Task | What | Needs | Effort | Minutes |
|---|---|---|---|---|---|
| `exchange-gone` | `exchange-gone` | `FromTransport::ExchangeGone`; the component handles it as `StreamGone`; `Call::finish` says it for a call handed over and unanswered; the red in the engine-clients world, failing on the base first; the mount's rows; the readers' focused runs; the red's story three times by name | | medium | 110 |

`notes/common.md` holds the rules the task keeps.

## Decisions

- **Where the fix goes: jig.** The request offered temper's stop
  sequence if jig already had a way to end an exchange. It doesn't, at
  the boundary that knows: only the engine end sees a call's end, and it
  tells the component of a stream's alone. temper's stop sequence can't
  tell which callers have gone. The component's `StreamGone` handler
  happens to drop any connection by its token, but 4.2 names it the
  stream's message, so the fix names the exchange's end, `ExchangeGone`,
  and both share one handler.
- **The pending work stays.** "Settles as cancelled" means for the
  caller: the component removes the logical connection, never the
  pending exchange. Dropping the pending exchange would lose what the
  core's answer settles: a link spent
  (`sign_in::a_lost_links_credential_check_spends_the_link_like_a_lost_core_answer`
  pins it), a carried value used or erased, a sign-out committed. The
  component's `settled` waits on connections, not on pending exchanges,
  so this is enough for the close. After `Closed`, temper's
  `Clients::down` sends the component nothing, so a late answer is
  dropped. Durable effects already committed are kept.
- **Said once, only when owed.** `finish` says `ExchangeGone` only for a
  call that handed the component an exchange or a public read and has
  had no reply. That keeps the message off every ordinary call's end, so
  the mapping's existing message-list tests keep their meaning.
- **The red's world** is `jig-engine-clients-world`: the component, the
  engine end and the reference server, with a scripted core and
  application and a peer stepped by hand. It is the lowest world that
  has both a peer's exit and the component's `Stop` and `Close`, the
  events temper's stop sequence sends. A family request taken and never
  answered is `Waiting::Ask`. The test runs three orders of the leave,
  the stop and the close, and fails on the base with `closed()` 0. It
  is in the focused suite. The kit's server keeps a call on a peer's
  orderly end (`PeerEnd::Call`), unlike temper's (`PeerEnd::Connection`).
  So the leave is a cut connection, and the orderly end is added where
  the kit allows it (a pre-approved option).
- **The deadline doesn't change.** The story's 60 s patience
  (`temper/crates/temper/tests/support/terminal.rs`, `exit`) stays.
- **The story runs three times**, by name and alone, as the request
  allows. It failed 4 times in about 92 runs, so three passes are a
  smoke check, never the proof.

## Critical path

`exchange-gone`, about 110 minutes, alone.

## Risks after landing

- A family request the root never answers while its caller is still
  there still holds the close until the caller's own deadline ends the
  exchange. The terminal then leaves, and `ExchangeGone` frees the drain.
  Why temper's root left this request unanswered through the drain is
  not this plan's. The request names the cause as the caller gone, and
  after-landing's runs of `process-temper` will show whether the drain
  still waits.
- A harness that hands its component transport messages after `Closed`
  may now trip the component's assert, when a call ends after the
  close. The task guards any it finds, as temper's and the engine-clients
  kit's owners already do.

## Overlaps with other drafts

None. No draft beside this one changes `jig-protocol-engine-clients`,
`jig-protocol-engine-clients-http`, the engine-clients kit or world, or
`jig/docs/design/protocol/client*.md`. `ignored-misc/drain-placement`
changes `jig-domain-host` and the worker wire, which this task doesn't
touch.
