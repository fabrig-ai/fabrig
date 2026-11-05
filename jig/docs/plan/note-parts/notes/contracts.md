# note-parts: the contracts

The run's binding calls: `jig/docs/design/domain/engine.md`, section 10, in code.

## The tool

`note` keeps its one schema, with two new optional members:

| Member | Type | Says |
|---|---|---|
| `first` | integer, at least 1, u32 | the 1-based first line replaced; the body's lines + 1 to append |
| `remove` | integer, at least 0, u32 | how many lines go from `first`; 0 to insert |

- **The revision read is `recalled`,** the member that already says it, required with a range. The request's `revision` is this field:
  one fact has one name, and a call without `first` stays as it is.
- `scope`, `description` and `body` stay required. In the range form `body` is the text that replaces the lines and `description` is
  written as named. `references` left out keeps the entry's.
- **Both decoders** (`jig-protocol-tools/src/notes.rs`, `temper-engine-smith/src/notes.rs`) hold the same rules, each refusal a range
  problem on the member named:
  - `first` without `remove`, or `remove` without `first`: on the member that is missing;
  - `first` of 0: on `first`;
  - `first` without `recalled`: on `recalled`; `recalled` without `name`, or `name` without `recalled`, on `name`, as today.

## jig-domain-notes

```rust
// lines.rs
/// The lines a range write replaces (jig/docs/design/domain/engine.md, 10).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LineRange {
    /// The 1-based first line replaced; the body's lines + 1 to append.
    pub first: u32,
    /// Lines replaced from `first`; 0 to insert.
    pub remove: u32,
}

/// `old` with `range`'s lines replaced by `text`, or why not. Its length is checked against `bound` before it allocates.
pub fn splice(old: &[u8], range: LineRange, text: &[u8], bound: u32) -> Result<Box<[u8]>, Refusal>;

// boundary.rs, Refusal gains
/// The range starts before line 1 or runs past the body's end.
PastEnd { lines: u32 },
/// The body would pass `notes.body_bytes`; `bytes` saturates at `u32::MAX`.
PastBound { bytes: u32, bound: u32 },

// boundary.rs, Event gains
/// Replace a range of an entry's body lines with `entry.body`, naming the revision this writer recalled.
WriteLines { owner: Token, entry: New, recalled: u32, range: LineRange },

// domain.rs
/// What a write `valid_new` rejects is told: a body past its bound says so, anything else is oversized.
#[must_use]
pub fn new_refusal(entry: &New, limits: &Limits) -> Refusal;
```

**`splice`'s lines** are `line_change`'s: a `\n` ends a line, a final `\n` adds no further empty line, an empty body has 0. Where `L` is
`old`'s lines, a range holds when `1 <= first`, `first <= L + 1` and `first - 1 + remove <= L`; else `PastEnd { lines: L }`, with no
overflow on any `u32`s. The result is the bytes before `first`, then `text`, then the bytes after the range, with one `\n` added only:

- between the bytes before and `text` or what follows, when those bytes are not empty and end in no `\n`;
- between `text` and the bytes after, when `text` is not empty, ends in no `\n`, and bytes follow.

Nothing else changes. So:

| `old` | `first`, `remove` | `text` | result |
|---|---|---|---|
| `a\nb\n` | 2, 1 | `x\n` | `a\nx\n` |
| `a\nb\n` | 2, 0 | `x` | `a\nx\nb\n` |
| `a\nb` | 3, 0 | `c\n` | `a\nb\nc\n` |
| `a\nb\n` | 3, 0 | `c` | `a\nb\nc` |
| `a\nb\nc` | 3, 1 | (empty) | `a\nb\n` |
| (empty) | 1, 0 | `x` | `x` |
| (empty) | 2, 0 | `x` | `PastEnd { lines: 0 }` |
| `a\n` | 2, 1 | `x` | `PastEnd { lines: 1 }` |

**The machine** loads the entry as a write does. In order: no entry, `Missing`; `recalled` not its revision or `entry.scope` not its scope,
`Moved { revision }`; `splice` refuses, that refusal; the revision can't advance, `RevisionExhausted`. Else it saves as a write does, the
result as the body, `entry.references` when not empty and otherwise the entry's, its answer `line_change(Some(&old.body), &result)`.
A refusal emits no `Save`.

`Event::Write`'s and `Event::WriteLines`' shape check is `valid_new`, its refusal `new_refusal`: so `entry.body` past `body_bytes` is
`PastBound` in either form. A party's correction stays as it is.

## jig-domain-core

```rust
// vocabulary.rs, NamedAction gains
/// Replace a range of a note's body lines under a revision seen by the run.
NoteLines { entry: notes::New, recalled: u32, range: notes::LineRange },
```

Its `ToolKind` is `Note { scope }`, as `Note`'s. `entry.name` of 0 is `NoteRefused(Missing)`; the author is set as for `Note`.
`note_person_refusal` maps `PastEnd` and `PastBound` to `Limit { which: Words }`: a party's correction never meets them.

## The store

`core-v6.schema`, in place, with no version bump:

```
record NoteRefusalPastEnd { lines: u32 }
record NoteRefusalPastBound { bytes: u32 bound: u32 }
enum NoteRefusal { busy, oversized, full, exists, missing, moved: NoteRefusalMoved, revision_exhausted, past_end: NoteRefusalPastEnd, past_bound: NoteRefusalPastBound }
```

Nothing of a range is stored but its result. The client's wire doesn't change: a run's answer reaches a client as its words.

## The words

`jig-protocol-tools/src/render.rs` and `temper-engine-smith/src/answers.rs` say these alike:

```
Note refused: revision moved, now 7
Note refused: range past the end, 64 lines
Note refused: body past its bound, 70000 bytes of 65536
```

A written note's, each its renderer's own, `lines` closing the later write's so that an append's `first` needs no counting. `·` is U+00B7, `−` is U+2212:

```
Wrote note 12 revision 3, 64 lines                         (jig-protocol-tools)
Note 12 revision 1 written · 64 lines                      (temper, a first write, as today)
Note 12 revision 3 written · +18 −6 · line 31 · 64 lines   (temper, a later write)
```

## temper's engine

```rust
// temper-engine-domain/src/boundary.rs, Tool gains
/// Replace a range of a note's body lines by the revision this run recalled.
NoteLines { entry: jig_domain_notes::New, recalled: u32, range: jig_domain_notes::LineRange },
```

Its checks are `Tool::Note`'s, and `checked_call` answers a rejected `Note` or `NoteLines` with `notes::new_refusal`.

## `task notes`

`temper ctl DIR task notes TASK --project PROJECT [--after CURSOR] [--json]` asks `c::Read::Notes { scope: NoteScope::Task { project,
task }, description: None, after }`, the read a chat's document pane makes. Its text, one note after another, lowest number first:

```
note 1 revision 4: Away mode
THE BODY
```

then, for a body that came cut, a line `body cut; report it`, and for a further page `read on with: task notes TASK --project PROJECT
--after CURSOR`. No notes print `no notes`. Its `--json`, the record `notes`:

```json
{"notes":[{"number":1,"revision":4,"description":"Away mode","body":"THE BODY","cut":false}],"after":null}
```
