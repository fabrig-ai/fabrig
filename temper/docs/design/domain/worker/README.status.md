# worker: status

2026-11-21. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 2: a start that fails is the engine still away: no story takes an engine through two failed starts to the adoption of the runs a worker hosts, and no dial is due at the grace's end less the handshake deadline.

Section 3.1 has no known gap in reading the link: the worker reads it
and fires its deadlines on every pass, whatever the entrance holds, and
the held entrance holds the engine's cap to a worker in frames, from the
link's sizes, and three records more for the link's own state; a new
record past it faults the link (`temper-service/src/worker.rs`,
`Worker::new` and `drain_link`). `temper-service`'s
`a_worker_whose_engine_records_wait_keeps_its_link`,
`the_link_is_read_while_its_last_output_waits` and
`a_new_wire_record_past_the_entrance_faults_the_link` hold it. A link
left unread under load is the likeliest cause of a deployment's worker
relinking every two to two and a half minutes on 2026-11-08, but that is
unproven: its evidence is outside the repository, and no deployment on
record has run since.

- **Not yet:** 2, *Told to stop, the worker drains*: its agents stop at the end of their turns, with no drain's grace, no part relayed and no workspace kept [in flight: change 11471].
