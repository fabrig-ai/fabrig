# forge-connection: status

2026-11-11. Where skein's code lags or departs from [connection.md](connection.md).

Unlisted sections have no known gap. Sections 3.5 and 5.2's key checks
are present: access admission parses without signing, and a key refused
only at signing stops the forge before any request goes out on it.

- **Not yet:** 4.6, 7.2, by plan `forge-polls-quiet`: no read but the
  delivery log's first page sends a validator, nothing is kept for a
  `304`, and the limits have no `kept_bytes` [in flight: change 155].
- **Not yet:** 4.2: `Settings` reads the forge though a row is kept, a
  `404` of `PullFor` or of an Actions read drops the row, and a write
  refused `Forbidden` keeps it (`skein-forge/src/forgejo/plans/reads.rs`,
  `failure.rs`) [in flight: change 159].
- **Departs:** 8: `skein/docs/foundation/programming-model.md`, 5.4 asks
  for one explicit state enum per lifecycle, each state holding exactly
  its data, no lifecycle in flags or `Option`s beside it, exhaustive
  matches and one handler per cell;
  `skein-forge-connection/src/installations.rs`'s `GitHub` keeps mint,
  read and identity flags and `Option`s beside its state enums; no
  reason is recorded. `skein-forge-connection/src/connection.rs`'s `Exchange`
  keeps `json_body` and `json_next` beside `receive`, and `Receiving::Json`
  holds neither, because `discard` leaves a demand in `json_down` that `step`
  still gives `json_demand`, and the request of 2026-11-05 moved them only if
  a reading found that safe.
  8.1: which deadline runs is the `until` a state holds, which
  `Connection::deadline` reads, where the section has it a function of the
  phase set in one place: a holder sets it with `Connection::until` above
  `exchange.start` as it takes a connection, and six more `until` calls in
  `component.rs` and `component/` set it, five as one leaves and one, in
  `start_heard`, as a forge's start goes on to its settings read on a
  connection still held, because a connection knows no holder's deadline, and
  the request of 2026-11-05 decided how the deadline is read, not who sets it.
  8.1: a connection that has connected reaches Idle rather than Sending,
  an answered exchange goes through Idle before the next Sending, and a
  Closed connection is retired and a new one opened, where the table has
  each go straight on; no reason is recorded. `written` stays beside the
  state, as a fact of the last send read after its exchange has gone.
  8.1: Closed and Idle, which the table has holding nothing, hold what
  `skein-forge-connection/src/connection.rs`'s `State` keeps past an
  exchange, because the request of 2026-11-05 moved no test's events or
  times: Closed its ending and its close under way; each a redirect's
  `draining`. For that reason Idle does not always run `idle`: a connection
  that connects with no holder keeps the deadline it opened under, and
  `component/github.rs` leaves one with no deadline, which
  `component/calls.rs`'s sweep reads as passed.
  8.4: Head holds a hint the root refused after `Next` went, offered
  before the next body, and Body one refused during it, which a `204`
  drops for its own and any other answer keeps; no reason is recorded. A
  refused head, a wrong signature and a malformed body wait in Answering
  until their answer's `Done`, where the table has them go straight to
  Closing, because Closing takes nothing more from the stream and could
  lose the answer. A fault's Closing stays under the deadline it had until
  its close goes, where the table has io's alone; no reason is recorded.
  7.1: a forge's dialect is not asked again, and the request of 2026-11-05
  that gave Forgejo's plans their states has it so too;
  `component/calls.rs`'s step matches `(side, &mut entry.dialect)`, a tuple,
  for every call it steps, because a GitHub call takes `gh` from its entry
  either way, and that request's debt item did not name this file.
  7.2:
  each forge must have its own connection in addition to the calls'
  `connections` limit; `skein-forge-connection/src/component.rs`'s
  `drive_forgejo` gates `open_own` on `call_connections`, which excludes
  only GitHub's own connections, and `component/calls.rs`'s `assign`
  reserves only GitHub's own connection from call reuse, so Forgejo's
  own connection still shares the calls' pool; no reason is recorded.
  7.4: each refusal names its limits, but
  `skein-forge-connection/src/limits.rs`'s `check` answers
  `Unusable::Zero`, which names none, for what `skein_tls::client::valid`,
  `skein_http::client::valid` and `skein_http::server::valid` refuse; no
  reason is recorded.
  10: the item the `debt-24` request of 2026-11-05 paid has the moved tests use
  the fake's own server, and the forge worlds share one io harness. `hand`'s
  tests keep their literal bytes, because they carry faults and identities no
  fake serves, and each world keeps its own arms, because a shared table ended
  131 lines up against 100 down, by that plan's owner. The `seam` feature holds
  `component/seam.rs`'s 108 lines and, with the crate's tests, `fixtures.rs`'s
  190, beside the step code.
  10: the `debt-26` request of 2026-11-06 has its leftovers' net lines fall, but
  they stand 7 up: their code fell 54 and their tests rose 61, 58 of those the
  gate on the seam's one world, which the same finding asks for.
