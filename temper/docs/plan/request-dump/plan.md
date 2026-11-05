# request-dump: the plan

A worker can keep each model request in a file, for the comparison: `temper/docs/design/draft/dogfood-hacks.md`, 2.10, written by this cut with its status lines.

## Parts and tasks

| Part | Task | Trees | Needs | Minutes |
|---|---|---|---|---|
| `dump-kit` | `dump-kit` | skein, smith | none | 120 |
| `dump-worker` | `dump-worker` | temper | `dump-kit` | 35 |

**Critical path:** `dump-kit`, then `dump-worker`: about 155 implementer-minutes, as estimated, not measured. One slice, `walk = false`: the next comparison's run is the proof, as the request has it.

## What was decided

- **The seam is `skein_llm::client::Client::prepare_measured`,** where `rendered` exists. Nearer the wire the bytes are gzip or TLS, and a head beside them holds the credential. The encoders take the prompt and limits, never a credential or an endpoint, which is the proof that a file holds none.
- **The agent process writes, not the worker.** Every model request leaves from `temper internal agent`; the worker never sees one.
- **The worker gives agents an empty environment** (`temper-service/src/worker.rs`, `process_spawn`), so the variable can't be inherited: it's carried as that environment's one entry. Commands and the program take theirs from the agent's configuration, so it reaches neither. There's no sandbox to allow the write: agents run as plain children (`smith/docs/design/protocol/agent.status.md`).
- **Pop methods, not new events:** `pop_body`, `pop_given` and `pop_dump` follow the shell's existing `pop_given_up`. `Measure`, the connection's `Event` and `Below` stay as they are, so no exhaustive `match` moves. It's small and easy to take out.
- **The writer is in `smith-agent-shell`, the variable in temper.** smith names no temper: its world can use the real writer in a scratch directory, and temper adds only the name, the carrying and the 4 GiB.
- **The bound is a sum of `DIR`'s sizes before each file,** since each agent is a process of its own and they share no counter. It can pass by one request per agent writing at once.
- **`dump.log` beside standard error:** a worker shows an agent's standard error only as a failed run's tail, so a stop on a healthy run would otherwise be read by no one.
- **`start.txt` is in:** `start_ready` sees the charter, so the file costs about 40 lines. Its text is in the requests already, so it holds what they don't: limits, tools granted, models.

## Departures

- **`dump-kit` crosses skein and smith,** where a lead's rule keeps a branch in its tree. The request says one or two tasks and that it comes out whole; three tasks, one per tree, would keep the rule and break the size. skein's commit comes first, so the kit's order holds. The two tasks and the cut's documents all land in one merge of the plan's trunk, which is what a revert takes out.
- **A file holds the body before gzip,** on an endpoint that compresses: readable, and equal to what a provider inflates. No endpoint temper configures compresses today, as far as a search of its crates shows.

## The gap, for the user

**Claude Code makes its own calls, so temper can't dump a Claude run's requests.** It keeps what it gives the program: argv with model and effort, instructions, MCP tools, each input line, and `system/init`. It can't read the program's own system prompt and built-in tools' descriptions, its later requests' messages, or effort as sent. Reading those would take a proxy in front of the program, which is not as cheap as this and changes the path a subscription's call takes; it isn't planned.

## Overlaps with other drafts

None of substance: no other draft's files include `skein-llm`, `skein-llm-connection`, the smith crates here, `temper/crates/temper/src/lib.rs` or `temper-service/src/worker.rs`. The cut's own commit adds a clause to `smith/docs/design/STATUS.md` and `temper/docs/design/STATUS.md`, and a line to the status files named above; `temper/docs/design/STATUS.md` is a paragraph other retirements edit, and a merge there is by hand if it meets one. No `temper-client-*` file is touched, so the UI drafts keep their priority.

## Readers of what changes

Surveyed by reading: `Client`, `MeasuredRequest`, `Backend`, `Stages`, `Agent` and `Worker` are each built in one place; `run_decoded` has one outside caller, temper's, which only `dump-worker` changes. `smith/docs/design/**` is read by tests, but the one reader found includes `shell.md`, which nothing here touches.
