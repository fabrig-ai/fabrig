# ui-goals: the run's binding shapes

What crosses tasks. A task builds to these; a name here is never another task's to change.

## The report (`temper-client-domain/src/pages/report.rs`)

```rust
/// A report note's description opens so (temper/docs/design/domain/agent.md, 8.3).
pub const OPENING: &[u8] = b"report: ";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// The description past its opening, its first line.
    pub headline: Box<[u8]>,
    /// In order; a paragraph's lines are joined by one space.
    pub paragraphs: List<Box<[u8]>>,
    pub left: Option<Box<[u8]>>,
    pub risk: Option<Box<[u8]>>,
    /// `Decided:`'s items, split at " · ", each kept whole with its time.
    pub decided: List<Box<[u8]>>,
    /// The note's `changed`.
    pub at: Option<Wall>,
}
/// Total: any note reads, a body cut as far as it goes.
#[must_use] pub fn parse(note: &c::Note) -> Report;
/// `Read::Notes { scope: NoteScope::Task { project, task }, description: Some(OPENING), after: None }`.
#[must_use] pub fn read(project: u32, task: u64) -> Read;
/// The newest by `changed` whose description opens with `OPENING`; none is a normal state.
#[must_use] pub fn newest(rows: &List<c::Note>) -> Option<&c::Note>;
```

- jig's filter finds the description's words anywhere, so `newest` tests the opening itself.
- A note with `changed` none is older than one with any; the larger note `number` wins a tie.
- A line starting `Left: `, `Risk: ` or `Decided: ` is that field, wherever it stands; a later one of the
  same field replaces an earlier. Those words elsewhere in a line are the paragraph's.
- A body's blank lines separate paragraphs; surrounding spaces of a line are not kept.
