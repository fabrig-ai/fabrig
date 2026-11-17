# Parts on task rows

Provisional, 2026-11-17. From `temper/docs/design/draft/page-round-trips.md`,
sections 4, 5 and 8, milestone R2 of
`temper/docs/design/draft/page-round-trips.execution.md`. It settles what
`README.md` left open on bundling an application's parts: a task query
names the parts it wants, and each of its rows comes with its own, so that
a list of tasks is drawn with no read per row. It gives a task query's
ended group a mark too, so that a client reads that group again only when
it would read otherwise. The engine's side is
`../../../domain/engine/views.md`, 2; the connector's,
`../../../domain/connectors.md`, 5.5.

## 1. In one page

- **A query names its parts** (2): a task query, read or watched
  (`README.md`, 4.1), names at most four of an application's parts, each by
  its owner, its kind and the owner's version.
- **A row carries them** (3): each row, live or ended, comes with one part
  per part named, in the query's order: the part's record in its owner's
  envelope, or why there is none, typed. A row is whole without one.
- **Made as reads are** (4): by the owner's connector, through the root, as
  the row is made for an answer or a delivery, from live state or the
  store, its loads a read's, held until durable with its row.
- **Marked with its row** (5): each kind names what it is made from; a
  commit to any of it marks the row, a note made, edited or deleted among
  them.
- **The ended group's mark** (6): a number every answer and delivery of a
  task query carries, which grows whenever its ended group would read
  otherwise, so a client reads that group again only when it grows.
- **On the wire** (7): jig's client wire, from version 28.

## 2. A query names its parts

- **`parts`,** a field of the task query, at most 4, each a `PartAsk`:
  `owner`, the application's family by its number (`README.md`, section 6);
  `kind`, the owner's number for one kind of part; `version`, the owner's
  version the client decodes. A query naming none is as before, and its rows
  carry none.
- **Part of the subject.** A query's parts are among its keys as a subject
  (`README.md`, 5.1): two watches naming different parts are two subjects,
  and a client shares a watch only where the queries are the same, parts
  and all (`../client/watches.md`, 1).
- **Checked by the engine's clients' component**
  (`../../../protocol/client/README.md`, section 6), as an envelope is: a
  part whose owner or version the engine does not serve is answered
  `not_served` on every row, naming the owner and the ranges it serves, as a
  read in a version not served is (`README.md`, section 6); a kind no
  connector of that owner declares is answered `unknown` on every row.
  Neither refuses the read or the watch, and neither asks a connector.
- **Admitted, windowed and clamped** as any task query is (`README.md`, 4.1
  and 5.2): parts add nothing to whom a query admits.

## 3. A row carries its parts

- **One per part named, in the query's order,** on every row of the query,
  live or ended, in a read's answer (both groups) and in every delivery.
  Each is a `RowPart`:
  - `value`: the part's record in its owner's envelope, `Owned { owner,
    version, record }`, at the version asked, as an application's read is
    answered (`README.md`, section 4). jig reads none of its bytes, on
    either end;
  - `not_served`, or `unknown` (2);
  - `failed`, with why: `busy`, the loads it needed were full;
    `too_large`, its record past its kind's bytes (4); `unreadable`, a
    load it needed failed or found nothing it could make it from.
- **A row is whole without one.** A part not made leaves its row drawn from
  what the line says, without what the part would have added, until the row
  comes again: its next delivery, or another read. Nothing reads per row in
  its place.
- **At the client,** jig's link checks that a row carries as many parts as
  its query names and that each value's envelope names the owner and the
  version its part asked; otherwise the answer or delivery is malformed
  (`README.md`, section 4). It hands a value's bytes to the application
  undecoded, as it hands an application's read (`../client/README.md`,
  section 13); the application decodes them as its own records.
- **The client's contract** (`jig_client_domain::contract`) names them
  as the wire does: `PartAsk { owner: u16, kind: u32, version: u32 }`,
  the owner a family's number; `TaskQuery`'s `parts`, a list of at most
  4; `TaskLine`'s `parts`, one `RowPart` per part asked: `Value(Family)`,
  the record under its owner as an application's read is handed
  (`boundary::Family { family, version, bytes }`), `NotServed(NotServed)`,
  `Unknown`, or `Failed(PartFailure)`, `PartFailure` being `Busy`,
  `TooLarge` or `Unreadable`; and `Tasks`' and `TaskPage`'s
  `ended_mark`, a `u64`.

## 4. Made as reads are

- **By the owner's connector, through the root,** as each row is made for
  an answer or a delivery: the core makes the rows, then asks each
  connector whose kinds the query names, once per answer, handing it the
  rows' lines as it made them, the kinds and their versions, and the party
  it answers (`../../../domain/engine/views.md`, 2). Nothing else asks for
  a part: there is no read of one by itself.
- **From live state or the store.** A connector makes a part from its own
  state, the core's and the store's, its loads bounded and admitted as a
  read's (`../../../domain/engine/README.md`, 5.3). What it keeps to save
  loads is its own and kept nowhere durable: parts are made when their rows
  are.
- **Its visibility is its row's.** A connector is handed only rows the
  party may see, and a part shows of its row's task what anyone who may
  see that task may see.
- **Durable before shown.** The answer or the delivery, parts and all, is
  held until every commit it reflects is durable, as any read's is
  (`README.md`, section 4, and 5.2).
- **Bounded by its kind.** Each kind declares its bytes, and the core
  answers a part past them `too_large`. A part holding a list bounds it in
  rows too, in a stated order, saying how many more; a connector cuts a
  text to what fits, saying so, and never leaves a part out for the client
  to read per row. An application fixes its kinds' bytes so that its
  client's window of rows, each with its line and its parts at their
  bytes, fits the client's document bytes (`../client/README.md`,
  section 11).

## 5. Marked with its row

- **Each kind names its inputs,** what it is made from, among: its task's
  line; its task's notes; its delegates' lines; its delegates' notes; and
  the application's own records of the task, which its connector marks
  (`../../../domain/connectors.md`, 5.5).
- **A commit to an input marks the row:** each watch whose query names a
  kind with that input, and which the row's task matches before the
  decision or after it, is marked changed (`README.md`, 5.2), and its next
  delivery makes the part again. A note made, edited or deleted in a task's
  scope is a commit to that task's notes and to its requester's delegates'
  notes. It moves no task's last activity
  (`../../../domain/engine/views.md`, 3).
- **Where the task has ended,** no live row moves: the commit grows the
  ended group's mark of the queries that may hold its row (6), and their
  watches are marked changed so that the mark reaches the client.

## 6. The ended group's mark

- **`ended_mark`,** carried by every answer of a task query's read and
  every delivery of its watch: a number that grows whenever the query's
  ended group would read otherwise, which is when a task it selects ends,
  when a task leaves its ended group (forgotten once read, or erased by
  retention), and when an ended row's part moves (5). It may grow more
  often than that, never less; a restart may grow it once. A query with no
  ended group carries 0.
- **Taken before the rows.** A read's answer carries the mark as it stood
  when the read began, before its load, so that an ending its load missed
  grows it after; a delivery carries the mark as it stands when the
  delivery is made.
- **Only a delivery tells.** Comparing a watch's live rows cannot show a
  task that started and ended between two deliveries, since deliveries
  coalesce (`README.md`, 5.2); the mark does.
- **The client reads the ended group again only when the mark grows:** on a
  delivery whose `ended_mark` is greater than that of the ended read it
  holds, it reads the ended group's first page again, and on no other
  delivery. What it does with the pages past the first is as before.

## 7. On the wire

jig's client wire, from version 28, adds to version 27's records:

```
record PartAsk { owner: Family kind: u32 version: u32 }
record TaskQuery { project: option u32 requester: option Requester root: option u64 parent: option u64 label: option u32 phase: option PhaseClass executor: option ExecutorKind tracked: option bool order: QueryOrder parts: list 4 PartAsk }
enum PartFailure { busy, too_large, unreadable }
record PartFailed { why: PartFailure }
enum RowPart { value: Owned, not_served: NotServed, unknown, failed: PartFailed }
record TaskLine { ... conversation: option ConversationRead parts: list 4 RowPart }
record Tasks { rows: bytes 16777216 more: bool ended_mark: u64 }
record TaskPage { live: bytes 16777216 live_after: option Cursor ended: EndedGroup ended_mark: u64 }
```

`TaskLine`'s other fields are version 27's, in its order. A line nested
in another document (an inbox entry's `reply`) carries no parts.

## 8. Open questions

- **Parts of other lists:** an inbox's rows carrying what their cards draw,
  as a task query's rows carry their parts
  (`temper/docs/design/draft/page-round-trips.md`, section 10). Default:
  task queries alone.
- **A part failed busy** comes again only with its row's next making.
  Whether the engine makes it again of its own accord once loads free.
  Default: it does not.
