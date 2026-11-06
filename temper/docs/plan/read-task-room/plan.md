# read-task-room: the plan

One part, one task: `run-read-room`. It needs nothing, so it is the critical path.

## The route, confirmed

A run's reads do not pass through a wider path. `read_task` goes `temper-engine-domain/src/route/tools.rs`, `relay_payload` → `NamedAction::ReadTask` → jig's `reads/run_task.rs`, `value`, which cuts at `limits.documents.text_bytes`. So the request stands, and the fix is in jig and temper.

Rendering already fits: `temper-service/src/route.rs`, `render_host` renders within the smaller of `core.call_answer_bytes` less the tool's name and the local agent's reply, and `temper-engine-smith`'s `task_read_words` cuts there and prints `next:`.

## What was decided

- **The root names the room with the read,** as a new field of the in-process `NamedAction::ReadTask`, and jig holds it to `core.call_answer_bytes`. No protocol, stored format or configuration name changes, so no `Question:`. Using `core.call_answer_bytes` alone was refused for the two faults below, and because a read would then copy up to 135,168 bytes to render 10,662.
- **Three bounds, not one.** The room is the local agent's `host_reply_text`, within `core.call_answer_bytes` and within `root.journal.transcript_bytes` less the answer's metadata. `Decision::deliver` refuses a delivery past the journal's bound and `host_route::read` asserts its door took it, so a chunk past it must not be possible. Standard's journal bound is 65,536, under its 135,168 call answer.
- **A process with no local agent stays at `documents.text_bytes`.** An engine alone doesn't know its remote agents' reply bound (`engine.host_reply_bytes` is `None`). A wider answer would reach its run as smith's `TooLarge` at every offset, so a long spec could not be read at all, where today it reads slowly. Kept as an open question in `temper/docs/design/domain/agent.md`, 10.
- **One holder of the reply's figure:** it moves from `temper-service`'s `Engine` to the domain's `Limits`, which `render_host` reads too.

## What a read holds

By reading, at `temper init`'s limits; the task measures it with the real `size_of`s:

- **One read:** words at most 10,662 (`tool_payload`), and no more than the source has left. Metadata 4,576 bytes at most: 16 dependencies at 8 bytes, 82 question credits at 48 bytes, a 512-byte hold account. Then its carrier, unknown until measured. About 15.2 KB and the carrier, where it was about 5.1 KB and the carrier.
- **At once:** `fleet.calls`, 256, bounds the reads held, so about 3.9 MB and the carriers at the very most. A box runs one run at a time, so a real read holds one or a few of these, from jig's answer until it is rendered.

A retained task's row is loaded whole as it is now (`core.load_bytes`), so that is unchanged.

## Overlaps

No draft beside this one lists `reads/run_task.rs`, `route/tools.rs`, `temper-engine-domain/src/limits.rs` or the suite's `host_calls.rs`. A `managed-listener` cut in flight may write in `temper/docs/design/protocol/setup.md`; its files are not known yet. This plan's edits there are two small passages in 8.1 and 16. It touches no client view, shell or look file, so fabrig/ui keeps its priority.

## Proof

The tests prove the bound. fabrig/qa's next goal's `read_task` count is the proof that it pays.
