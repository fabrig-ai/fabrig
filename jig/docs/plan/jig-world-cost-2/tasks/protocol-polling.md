+++
id = "protocol-polling"
part = "protocol-polling"
kind = "feature"
needs = ["naming-row"]
files = [
  "jig/testing/jig-client-protocol-kit/src/net.rs",
  "jig/testing/jig-client-protocol-kit/src/net/records.rs",
  "jig/testing/jig-client-protocol-kit/src/net/tests.rs",
]
effort = "medium"
minutes = 60
+++

# The protocol world's net does work when a socket's bytes changed, and reads only what's new

## Goal

`Net::advance` answers both wires of every socket at every step, and `check` parses each message's head and walks each record reader on every fragment that arrives. Ask a wire only when it has work, and read each message once as it arrives. Every cut, every record check and the order of events stay as they are.

Acceptance:

1. `Net::advance` asks `Wire::answer` only of a wire whose `work_pending()` holds, and passes a socket that's broken, has no failure to report and no armed upload cut without further work.
2. A message's head is parsed once, when its end has arrived, and `Records::feed` returns at once when no byte has been added and searches only what's new.
3. `Net`'s public items keep their signatures, so `client.rs`, `front.rs` and `jig/tests/ops/src/joint/bytes.rs` don't change.
4. The tests of step 4 pass and bite.
5. The focused tests of `jig-client-protocol-kit` and `jig-client-protocol-world`, and the fuzzy tests of `jig-client-protocol-world`, pass, and `heavy cargo nextest run -p jig-ops-world --profile fuzzy -E 'test(/^fuzzy::joint::/)'` passes: the joint fuzzy world reads this `Net`.

## Inputs

- `jig/docs/design/client/domain/testing.md`, 2, and `jig/docs/design/protocol/client.md`, 14, for what the protocol world holds. This task changes no design.
- `skein/testing/skein-world/src/stream.rs`, `Wire::answer` and `Wire::work_pending`: read it, and stay out.
- `jig/docs/plan/jig-world-cost-2/plan.md`, *Decisions*.
- `jig/docs/plan/jig-world-cost-2/notes/common.md`.

## Owns and may change

- **Owns:** the spec's `files`.
- **Neighbours:** `net.rs` gains `#[cfg(test)] mod tests;`. Nothing else outside `files` should need an edit.
- **Why skipping `answer` keeps the schedule:** `!work_pending()` means the wire is idle with nothing pending and no end to give. Then `answer` draws nothing, as `self.pending` is empty and an unmet read with no room reaches no `rng.chance`, and pushes no step and returns `None`. `Net::pending`'s comment already relies on this. Step 4's first test holds it. `eof` is read live, so keep `socket.engine.eof = socket.shell.finished` before the guard.
- **Mind the armed upload cut:** today's first block sets `upload_limit` and takes `self.cut_upload` for any keyed socket, **a broken one included**, so a broken socket with an unread keyed request can use up the armed cut. That's what the world does now, and it stays: the early pass over a broken socket holds only while `self.cut_upload` is `None`.
- **Options:**
  - Keep a `live` set of tokens instead of walking `self.sockets`, only if it stays plainly in token order and every path that breaks or fails a socket is plain from the code. The default is to keep the walk.
  - `whole` and `body` stay for the sources and the cuts, read once per message or only while a cut is armed. If `Message` makes either unused, it goes.
  - If a story fails because it relied on an extra attempt, that's a schedule change: it's your bug, not the story's. Find which wire drew, and fix the guard.

## Steps

1. `Net::advance`, inside the walk:

   ```rust
   if socket.broken && !socket.fail && self.cut_upload.is_none() {
       continue;
   }
   // ...the cuts, as they are...
   let shell_up = if slow || !socket.shell.work_pending() { None } else { socket.shell.answer() };
   // and `socket.engine.work_pending()` joins the engine's `let` chain, before `answer`.
   ```

2. A message read as it arrives, in `net.rs`, with two on each `Socket` (`reads: Reads { request, response }`):

   ```rust
   /// One direction's HTTP message: its head is parsed once, when its end has
   /// arrived, and each later look compares a length.
   #[derive(Default)]
   struct Message {
       /// How far the search for the head's end has come.
       searched: usize,
       head: Option<Head>,
   }
   struct Head {
       body: usize,
       /// `None` for a head that isn't UTF-8 or declares no length: never whole.
       length: Option<usize>,
   }
   impl Message {
       /// What `whole` answers for these bytes. Bytes are only ever appended.
       fn whole<'a>(&mut self, http: &'a [u8]) -> Option<&'a [u8]>;
   }
   ```

   `check` uses it for `socket.request` and `socket.response`. `whole(&socket.shell.received)` and `whole(&socket.engine.received)` stay as they are: they run once per message, at its completion. Share the head's parsing, so the two can't differ.

3. `Records` in `net/records.rs`: `feed` returns at once if `http.len()` hasn't moved since the last call, and the search for a line's end resumes where the last one stopped. `check` keeps its one call per socket and its order: request, then response.

4. Tests in `net/tests.rs`, child of `net`, so it reads private fields:
   - `answering_a_quiet_wire_changes_no_schedule`: two `Net`s of one seed take the same downs and engine writes through a whole exchange with fragments and room. One also calls `answer` on each `!work_pending()` wire before each `advance`, asserting `None`. At the end, each socket's `shell.trace` and `engine.trace` are equal between the two, and a message crossed. Break it once by guarding with `false`, so nothing is answered, and see it fail on the crossing.
   - `a_message_read_as_it_arrives_answers_as_whole_does`: at each prefix of a message, `Message::whole(prefix)` equals `whole(prefix)`. Rows: mixed-case `Content-Length`, no length, a head that isn't UTF-8, and bytes past the length. Break it once by caching a length of the first prefix's, and see it fail.
   - `records_fed_byte_by_byte_match_records_fed_at_once`: chunked `data: ` lines split in chunk headers, CRLFs and lines. Break it once by starting the line search one byte late, and see it fail.
5. Narrow runs by name, then the implementer's closing steps, then the fuzzy runs of acceptance 5. Commit and push.

## Report

The implementer's report, plus whether you kept the walk, each rule broken and the failure seen, and any story that failed on the way and why.
