# ui-proposal-card: the run's binding shapes

The design: `temper/docs/design/client/README.md`, 5.13, *The card in a chat*; `skein/docs/design/ui-web.md`, 7, *A focus request*.

## A focus request comes into view (`focus-reveal`)

In `skein/crates/skein-ui-web/src/page.rs`, a new variant beside `Op::Focus`:

```rust
Reveal { node: NodeId },
```

`Page::apply` pushes it straight after the `Op::Focus` of a focus request, and only there: a focus moved by a removal or by keys
pushes none. `page/web.rs` draws it as `scrollIntoView` with `block` and `inline` both `nearest`.

## "Change something" (`change-something`)

`temper_client_domain::pages::Act::FocusComposer` goes. Its one producer is the card, so its place is taken by:

```rust
Act::ChangeProposal { proposal: c::ProposalKey },
```

routed as `FocusComposer` was. On `Chat`, public:

```rust
/// The proposal "Change something" was pressed on, until something is sent or it is decided (client/README.md, 5.13).
pub change_asked: Option<c::ProposalKey>,
```

- `Act::ChangeProposal { proposal }` sets it and moves `composer_focus` by one.
- It goes back to `None` where a message's ask goes out (`Chat::send`, `Sending::Message`, past its early returns), and in
  `Chat::proposals` when `task.proposals` no longer holds its key: that list is what any decision, from any place, changes.
- The view reads it in `session_composer`: `Some` says `Word::SettingsProposalChangeAsks`, "What should change?"; `None` says
  `Word::MessagePlaceholder` as today.

## The facts jig gives (`proposal-facts`)

The design: `jig/docs/design/client/domain/contract.md`, 5.1, *an activity line*. The store and the wire change in place, under their
current names (`tasks-v9`, `client-v19`): no compatibility, no version bump.

A line of a change to a proposal names it. In `jig-domain-tasks`' `activity.rs`, and likewise in the client's contract
(`jig-client-domain`'s `contract/activity.rs`):

```rust
Decided { revision: u64, proposal: Option<DecidedProposal> },

/// Which proposal of the task's a `Decided` line is about, and whether it was its making.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DecidedProposal {
    pub number: u64,
    pub made: bool,
}
```

- `positions::history` writes `Some`, `made` true only for `Change::Proposed`. `control.rs`' `save_history` writes `None`.
- The line stands in the proposer's activity, so its key is `ProposalKey { proposer: Proposer::Task(task), number }`.
- Wire, `client-v19.schema`: `record ActivityDecidedProposal { number: u64 made: bool }`, and `ActivityDecided` gains
  `proposal: option ActivityDecidedProposal`. Store, `tasks-v9.schema`: `record DecidedProposal { number: u64 made: bool }`, and
  `DecidedLine` gains `proposal: option DecidedProposal`.

An ended proposal can be drawn again:

- `ProposalHistory` gains `reason: DocumentText`, at the bound a live proposal's reason has (`proposal_reason` in
  `jig-domain-core`'s `documents/objects.rs`), in the core's document, the client's contract and `client-v19.schema`.
- `RetainedActionSettings` gains `projects: List<u32>` (wire: `list 32 u32`, `PROPOSAL_PROJECTS`). So that it survives the decision,
  `tasks::ProposalAction::Settings` gains `projects: Box<[u32]>` and the store's `ProposedSettings` gains `projects: list 32 u32`,
  set from `set.projects()` where the proposal is made (`routing/settings/dispatch.rs`, near its line 723). A party's proposal has no such
  record, so its ended form says no projects; a chat shows none of those.

## A card's place (`card-place`)

In `temper-client-view`'s `pages/chat.rs`, computed once for a drawing, before its lines:

```rust
/// Where a card stands among the window's lines (client/README.md, 5.13, *Where a card stands*).
enum Place {
    /// At the call that names it: `proposal_anchor`, as today.
    Call,
    /// After the line at this index, a committed turn.
    After(usize),
    /// At the conversation's end.
    End,
    /// Past the window's end, or out of it: not drawn.
    Outside,
}
```

In order, first that holds:

1. `proposal_anchor` finds it: `Call`.
2. A line `Activity::Decided { proposal: Some(DecidedProposal { number, made: true }), .. }` for this card, its key's proposer
   `Proposer::Task(chat.task)`: `After` the first `Line::Turn` following that line; with none, the end's rule.
3. No such line, and a making time known, the proposal's `waiting.since`: `After` the first `Line::Turn` whose `at` is at or after it;
   with none, the end's rule.
4. No line and no time, its first read still out: `End`.

The end's rule: `End` where `thread.lines.edges.reaches(End::Before)`, the window holding the newest lines, and `Outside` otherwise.

Commit position decides where it can: a turn resent after a busy answer can commit out of its order and `Wall` ties within a step, so
the time is only for a card with no line in the window. A `Line::Turn` is one completion: its call and that call's result are in it, and the
words after the result are the next turn's, so the reply follows the card.

## A settings card's facts (`card-reload`)

In `temper-client-view`'s `cards/proposal.rs`, replacing `settings_proposal` and the session's snapshots:

```rust
/// What a settings card draws, from a live proposal or its retained record (client/README.md, 5.13).
pub(crate) struct SettingsFacts<'a> {
    pub(crate) key: c::ProposalKey,
    pub(crate) projects: &'a [u32],
    pub(crate) reason: &'a c::DocumentText,
    pub(crate) items: &'a [c::RevisionItemHead],
    pub(crate) made: Option<u32>,
    /// The live proposal, for its acts and who it waits for; none once it has ended.
    pub(crate) live: Option<&'a c::Proposal>,
}
pub(crate) fn settings_facts(card: &Card) -> Option<SettingsFacts<'_>>
```

`Some` for a page that is a `Proposal` whose action is `Settings`, or an `Ended` whose value is a `Proposal` with `RetainedAction::Settings`.
