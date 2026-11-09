# channel: status

2026-11-09. Where jig's code lags or departs from [channel.md](channel.md).

Section 5.1's hello carries endpoint bounds, not a model list; no known
implementation gap remains in that declaration or its fleet checks.
The hello's optional reply bound crosses both link translations and the
worker wire as `reply_bytes: option u32`; wire family 9 carries it in
place, with no version change. No implementation gap is known in that path.
Its logins rows also carry usable program statuses' optional email,
which reaches the accounts child through the fleet's accepted inventory;
there is no known gap in that path.

Sections 5.1 and 5.3 have no known gap in the `draining` notice: worker
wire version 9's empty record, kind `0x010b`, which both sides'
translations carry (the translation tests of `jig-protocol-worker-link`
and `jig-protocol-engine-workers`), sent before
the answers of the runs a shutdown cancels (`../domain/hosts.status.md`,
6.7).

Sections 5.1, 5.2 and 8 have no known implementation gap in `read list`
and `list read`, their bounded rows and failures, or retaining a key
grant while its read is under way. The hub hears the grant's name alone.

Sections 9, 10.1, 10.2 and 12 have no known gap in a link's loss and a
worker's return, but the one below. Both components restart the silence
deadline on each read of the open phase, a piece of a frame included;
the engine's tells why each accepted connection ended, once, as
`Request::Ended` with its `Loss`, a socket turned away for room among
them, and the worker's each channel or dial, as `Fact` with its own
`Loss`; and a connection that leaves its open phase, at a stop too,
holds the stall deadline, past which the engine's component aborts its
socket (`jig-protocol-engine-workers/src/component.rs`, `fire`;
`jig-protocol-worker-link/src/component.rs`). The engine's world's
`a_worker_lost_fifty_times_each_way_is_taken_back_each_time`,
`each_connections_end_says_why_once` and
`a_close_with_a_send_held_below_closes_within_the_stall_deadline`, the
worker's world's
`a_frame_arriving_in_pieces_keeps_the_link_past_the_silence_deadline`
and `each_loss_says_why`, the joined
`a_link_busy_both_ways_for_minutes_is_never_lost`, and the protocol
world's `a_worker_cut_fifty_times_is_taken_back_each_time` hold it.
That a send held below, keeping a lost connection in its room, was
what stopped a deployment's engine taking its worker back on
2026-11-08 is unproven: that evidence is outside the repository, and no
deployment on record has run since.

- **Not yet:** 12: the engine's fifty-loss row loses its worker by
  silence, stall, a send held below, the stream failing and ending, not
  admitted, replaced and framing, but not by a full cap or the worker's
  refusal, which end through the same close (the engine's
  `component.rs`, `loss`) and are named once each
  (`each_connections_end_says_why_once`).
- **Not yet:** 1, 4.1, 8, 10.1, 10.2: TLS between hosts: the workers' listener and the worker's dial are plaintext on loopback or in-process (`jig/crates/jig-protocol-config/src/hosts.rs`, `address.rs`). 3, 6.4, 8: a worker whose root composes `smith-inline-agent`: none does, so no worker gives a tool begun with its call.
- **Departs:** 12: the `jig-world-cost` request says no story loses what it
  checks. Root checks at every step cover the six drawn fuzzy stories and
  the replaced-key and application-generation stories, but not the other
  stories, including `Facts` and the refusals. Those keep arrival, next-step,
  pre-replacement and surviving-host end checks, which can miss a transient
  value or one held by a killed host. The plan chooses this limit because
  those grants arrive through the same `Assignment` and `Granted` frames as
  the covered stories; extending coverage remains section 14's question.
