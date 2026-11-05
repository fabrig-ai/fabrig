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

## A brief names its task

The brief's task section (`jig-domain-core/src/runs/brief_text.rs`, `task_read`) gets one line before its
`[Requested by …]` line: `[This task: 12, in project 3]`. A charter says "the task and project your brief's
task section names".

## The report note, as a run writes it

```json
{"scope":{"kind":"goal","project":P,"goal":G},
 "description":"report: Six of nine tasks landed; the store task is split in two.",
 "body":"I split the store task after two size failures.\n\nThe rest runs.\n\nLeft: the two halves.\nRisk: lazy loading touches every page.\nDecided: Split the store task, 05:40 · Retried it once, 05:12"}
```

A revision adds `"name":N,"recalled":R`, both from the brief's index. The stock's tool mask is 1,671 (1,543
and note, 128), its `notes` `Scopes::GOAL`; `recall`, 256, stays off.

## The address (`temper-client-domain/src/address.rs`)

```rust
Address::Goals { project: u32, filter: GoalsFilter }
pub enum GoalsFilter { All, Needs, Live, Ended }      // Clone, Copy, Debug, PartialEq, Eq; re-exported at the crate's root
```

`/projects/<project>/goals`, `?state=needs|live|ended`, as `parse_chats` and its print. An unread `state` is dropped.

## The domain's page (`temper-client-domain/src/pages/goals.rs`, `pub mod goals`)

```rust
pub struct Goals { pub filter: GoalsFilter, pub live: Handle, pub ended: Handle, pub needs: Handle, pub report: Handle, /* private */ }
impl Goals {
    /// The live watch's rows, by priority.
    pub fn live<'a>(&self, client: &'a Client) -> Option<&'a c::Tasks>;
    /// The ended page read last, and whether it has more.
    pub fn ended(&self) -> &[c::TaskLine];
    pub fn more(&self) -> bool;
    /// Rows held and read so far, for the headline: live rows, then every ended page read.
    pub fn count(&self, client: &Client) -> u32;
    /// The live watch has more, or the ended page does.
    pub fn unread(&self, client: &Client) -> bool;
    pub fn report(&self, task: u64) -> Option<&report::Report>;
    /// A needs row's `selection` names the goal.
    pub fn needs_person(&self, task: u64) -> bool;
}
/// The goals' query, `None` while `me` lists no goal template for the project.
#[must_use] pub fn query(me: &Me, project: u32) -> Option<TaskQuery>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State { Needs { funds: bool }, Paused, Planning, Running, Landed, Stopped, Failed }
/// First match wins (temper/docs/design/client/README.md, 3.5, Goals).
#[must_use] pub fn state(chip: &c::Chip, needs: bool, report: bool, delegates_total: u32) -> State;
/// The page's filter holds this state.
#[must_use] pub fn shown(filter: GoalsFilter, state: State) -> bool;
```

- `query`: `TaskQuery { project: Some(project), label: Some(template(me, project, shapes::GOAL)?.label), tracked: Some(true), order: Priority, requester: None, root: None, parent: None, phase: None, executor: None }`.
- `Pages::goals(&self) -> Option<&goals::Goals>`; `Act::GoalsFilter { filter }`, a move as `Act::ChatFilter`; `Act::MoreGoals`.
- `state`'s reads: **funds**, `chip.details.phase` is `ChipPhase::Held { reason: HoldCategory::Exceeded(_), .. }`; **paused**, `chip.details.details.hold`'s `placed` has `mode` Stop, `scope` `Subtree { task }` with `task == chip.task`, not `lifted`; **ended**, `ChipPhase::Ended`, its status from `chip.details.details.ending` (`ChipEnding`; `pages/task.rs`, `row_state`, is the pattern).

## The view (`temper-client-view/src/pages/goals.rs`, `pub mod goals`)

```rust
pub(crate) fn main(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, project: u32, times: &mut Times);
pub(crate) fn row(b: &mut Builder, words: &Words, bindings: &mut Bindings, times: &mut Times, line: &c::TaskLine, state: State, report: Option<&Report>);
pub struct Way { pub name: Box<[u8]>, pub line: Option<Box<[u8]>> }
pub fn ways(me: &Me, project: u32, words: &Words) -> List<Way>;
```

## The fact (`Domain::has_goals`)

```rust
impl Domain { #[must_use] pub fn has_goals(&self, project: u32) -> bool; }
```

`false` for a project not open, or while neither its watch nor its read has delivered. Its handles stay through every page.

## The look (`temper-client-view/src/look.rs`, from the next free token)

`GOALS` `temper-goals` (`.col`), and `temper-goals-`: `intro`, `kicker`, `headline`, `lede`, `byline`, `section`, `section-head`, `section-title`, `signal`, `empty`, `facts`, `rows`, `row`, `row-ended`, `top`, `line`, `filters`, `more`, `state-needs`, `state-running`, `state-planning`, `state-paused`, `state-landed`, `state-stopped`. Constants `GOALS_INTRO` and so on, as `HOME_*`; `look::GOALS_NAMES` is what the test lists.

```html
<a class="temper-goals-row[ temper-goals-row-ended]">
  <span class="temper-goals-top"><span>TITLE</span><span>6 / 9 · <span class="temper-goals-state-needs">needs funds</span></span></span>
  <span class="temper-goals-line">HEADLINE</span>
</a>
```

## Words (`temper-client-view/src/words.rs`)

From 3,189, in plan order; beside you, other plans take theirs from the file's next free number. A catch-up renumbers: no number is stored.
