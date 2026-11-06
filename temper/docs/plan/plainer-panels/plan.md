# Plainer panels: the plan

The request is `temper/docs/plan/plainer-panels/inputs/request.md`.

## Parts and tasks

| Task | Tree | Needs | Does |
| --- | --- | --- | --- |
| `person-answered` | temper | none | item 1: `answered by: PARTY` |
| `hold-word` | jig | none | item 2's word: `held` in jig's words |
| `held-waiting` | temper | `hold-word`, `person-answered` | item 2: asks the form, then prints it |
| `help-says-so` | temper | `person-answered`, `held-waiting` | item 3, and closes the design |

## Critical path

`hold-word` and `person-answered`, then `held-waiting`, which waits there for the
user's answer, then `help-says-so`.

## What was decided, and why

- **Item 1 names who answered,** rather than saying nothing. The answerer already
  reaches the client: an ended person task's read carries its result, whose
  `chosen_by` holds it. `person_page` drops it today. So no wire or engine change
  is needed, and the request's first choice costs no more than its second.
- **A closing person task with no holder prints neither line.** Its live document
  doesn't say who answered, so the request's second choice applies there.
- **Item 2 keys on `task.hold` and `WaitsOn::Nothing`.** `WaitsOn` stays as it is:
  a `Held` variant would change the engine's documents and the wire, which the
  request doesn't ask for and which is hard to reverse.
- **The form of a hold is asked, not chosen.** The request gives `held` or `held:
  REASON` to the person who wrote the draft, and says the coding change that
  reaches it asks. `held-waiting` reports `Question:` before it writes code, and
  its text says the answer is the user's.
- **No `[[question]]`.** That table is for what no task waits on, and
  `held-waiting` needs this answer. A second ask would also split one answer across
  two places.
- **`hold-word` asks nothing,** since `held` opens both forms.
- **`held-waiting` needs `person-answered`,** to keep one writer on
  `temper-client-tools`; both edit `document.rs` and `task_words.rs`.
- **The help comes last,** since it prints the form the user chose. It also closes
  `tools.md`'s open question and `tools.status.md`'s entry, as the two code tasks
  are told design isn't theirs.

## Left as it is

`temper help task`'s Limits says ctl can't take a task or hand it back, though
`task take` and `task hand-back` do so for a person task. `tools.status.md` records
this gap. The request says nothing else changes, so `help-says-so` leaves the line
and reports it.

## Overlaps with other drafts

- **`temper/crates/temper-client-view/src/words.rs`, unavoidable:** a new jig word
  moves its counted `PHRASES` and `SEGMENTS`. `ui-walk-large/home-true`,
  `ui-walk-large/needs-proposal`, `look-round-6/call-line`, `look-round-6/reading`
  and `look-round-7/sign-in-wait-page` all change that file and those lines. A
  conflict there is settled by regenerating the count with
  `SKEIN_CODEGEN_WRITE=1`.
- **`jig/docs/design/client/domain/client.status.md`:** `chat-reading-a/turn-order`
  also changes it. `hold-word` adds and removes its own line.
- **`temper/crates/temper-docs/src/tests/rules.rs`:** `box-link-first/box-help`, already merged,
  also changed it. `help-says-so` adds one test.

Nothing else overlaps: no other draft names `temper-client-tools`, `task.md`,
`tools.md` or jig's `words`.
