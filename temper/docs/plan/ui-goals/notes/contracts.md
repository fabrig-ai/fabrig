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

## A line's counts of its delegates (jig)

Both on `jig_client::contract::TaskLine`, and on the wire's `TaskLine`, beside `phases`:

```rust
/// Direct delegates ended done (jig/docs/design/client/domain/contract.md, 5.1).
pub delegates_done: u32,
/// Direct delegates live or ended, cancelled ones left out.
pub delegates_total: u32,
```

- The source is `TaskRecord.ended_delegates: Endings`, kept as each direct delegate ends, beside
  `ended_below`. `delegates_done` is its `done`; `delegates_total` is the live `delegates`' length plus its
  `done` and `failed`.
- A delegate closing as cancelled counts until it has ended. A moved delegate leaves the total.
- Goals' rows read these alone: no row reads its delegates to draw "6 / 9".
