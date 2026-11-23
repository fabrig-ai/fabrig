# worker-link: status

2026-11-23. Where the tree's code lags or departs from [worker-link.md](worker-link.md).

- **Not yet:** 1.1: the dial due at the grace's end less the handshake's deadline; a redial waits at most a quarter of the grace, so an engine back in the grace's last quarter can miss the last dial (`jig-protocol-worker-link/src/component.rs`).

`README.md`, section 4.1, and `liveness.md`, section 1 have no known gap in agreeing output rooms. The v9
credential declares the worker component's room; the engine checks it
after authentication and payload, before capacity or replacement, and
refuses unequal rooms with both values and `io.output` in the refusal
and its opening diagnostic. The engine world's
`unequal_output_rooms_are_refused_with_both_values_and_the_setting`
holds agreement, both sides of inequality and authentication order;
the component's `a_resumed_start_is_fitted_to_the_engines_room` pins the
fit independently of the opening guard. temper's service shell renders
both values and the setting in its operator line.

`records.md`, section 1's hello carries endpoint bounds, not a model list; no known
implementation gap remains in that declaration or its fleet checks.
The hello's optional reply bound crosses both link translations and the
worker wire as `reply_bytes: option u32`; wire family 9 carries it in
place, with no version change. No implementation gap is known in that path.
Its logins rows also carry usable program statuses' optional email,
which reaches the accounts child through the fleet's accepted inventory;
there is no known gap in that path.

`records.md`, sections 1 and 3 have no known gap in the `draining` notice: worker
wire version 9's empty record, kind `0x010b`, which both sides'
translations carry (the translation tests of `jig-protocol-worker-link`
and `jig-protocol-engine-workers`), sent before
the answers of the runs a shutdown cancels (`../../domain/hosts/hub.status.md`,
7).

`records.md`, sections 1 and 2, and `README.md`, section 7 have no known implementation gap in `read list`
and `list read`, their bounded rows and failures, or retaining a key
grant while its read is under way. The hub hears the grant's name alone.

`liveness.md`, section 1, and `worker-link.md`, sections 1.1 and 1.2, and `testing.md`, section 1 have no known gap in a link's loss and a
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

`records.md`, section 2, and `liveness.md`, section 1, and `README.md`, section 10, and `testing.md`, section 1 have no known gap in the room. Each side's
component takes its stream's room, io's `largest_room`, and is not made
unless the room holds a `Refuse` and the largest frame of each kind it
sends, the worker's `Open` among them, the engine's assignment and job
alone exempt (each crate's `limits.rs`, `valid`). The engine's component
measures each record's frame before the channel takes it: an assignment
or a job past the room, or one the channel refuses at its entrance for
anything but a full queue, is answered refused with words naming its
bytes and the bound, a read list fails as one that cannot encode, any
other record is dropped and counted, and each is told once as
`Request::Unsent`, the connection's ping armed again an interval on
(`component.rs`, `send` and `unsent`). A job that cannot encode is
answered `job cannot encode`. The engine's world's
`a_start_past_the_room_is_answered_and_the_link_lives`, the component's
`the_room_holds_control_and_every_non_start_frame`,
`an_unencodable_job_is_answered_with_bounded_words` and
`a_channels_closed_entrance_answers_the_current_record_once`, the
worker's world's `a_room_below_a_frame_the_worker_sends_is_not_made`,
and the joined
`an_assignment_past_the_engines_room_keeps_the_link_and_fails_its_run`
hold it. That a start past the room is what silenced a deployment's
link on 2026-11-09, at fabrig `a66fcfb9f7`, which already held the
loss handling above, stands on reading alone: that evidence is outside the
repository, and its proof is that deployment's next `worker unsent`
line, which no record shows.

`records.md`, section 2, and `liveness.md`, section 1, and `README.md`, section 10, and `testing.md`, section 1 have no known gap in *A resumed start fits
its room first*. `jig-protocol-worker-wire`'s `fit::fit` keeps an
assignment whole when its frame fits; otherwise it cuts every host
answer longer than its words, `fit::cut_words`, and gives back from
the newest call the answers whose frame still fits, stopping at the
first that does not; a delivery's answer, and a host answer no longer
than its words, stay whole; an assignment still past the room is
returned cut, `Fitted::Past` with its bytes. `sizes::smallest_resumed`
measures the smallest resumed start. The engine's component fits each
assignment before measuring it, never a job, and refuses one `Past`
with the bytes so cut (`component.rs`, `send`); `valid` refuses limits
whose smallest resumed start passes the room (`limits.rs`). The wire's
`a_resumed_start_is_fitted_to_its_room` and
`the_smallest_resumed_start_is_measured`, the engine's world's
`a_resumed_start_fits_the_room` and
`the_room_holds_the_smallest_resumed_start`, and the joined
`a_resumed_start_fits_the_room_on_each_stock_profile` hold it.

- **Not yet:** 1.1, 1.2: TLS between hosts: the workers' listener and the worker's dial are plaintext on loopback or in-process (`jig/crates/jig-protocol-config/src/hosts.rs`, `address.rs`).
