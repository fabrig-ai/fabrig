# reference-m5: contracts

The shapes that cross this plan's tasks, binding as written. The design they build is
`temper/docs/design/domain/forge.md`, 8.1 and 8.11; `domain/agent.md`, 4.3.1 and 8;
`protocol/configuration.md`, 13.2, 13.4 and 13.9; `client/README.md`, 4.6; `client/forge.md`,
9.2; `client/tools.md`, 5.5 (area 6's `change show`). Names below are Rust paths unless a
JSON or schema spelling is given.

## C1. The route and the bound (`temper-engine-domain-forge-change`, `boundary.rs`)

```rust
/// How a change takes its agents' questions (domain/forge.md, 8.11).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Route {
    #[default]
    None,
    Foreman,
    Up,
}
```

- `Limits.questions: u32`, the questions a change takes from its making or its last release,
  beside `repairs`; `forge.change_policy.questions` by name; 4 on `small`
  (`temper-service/src/limits.rs`). `temper-engine-domain-forge`'s `Limits::valid` refuses
  more than `QUESTIONS_OPEN - 1` (C2).
- `Hold::Questions`, after `Authority(..)`: "Another question would exceed the change's
  bound." It escalates as `Hold::Repairs` does (`tasks::Hold::Procedure`), its words
  `its questions are spent: N of N taken; question Q of task T waits; release it to pass the
  question up` (N the bound, Q and T the question's number and asker).
- On the wire (`client-wire-v1.schema`'s `enum Hold`), `questions` appended after
  `authority`; `ctl`'s word for it `questions`; the web client's `ChangeWord::QuestionsSpent`,
  "its questions are spent".

## C2. The relay's record (`temper-engine-domain-forge-change`, `boundary.rs`)

```rust
/// Open questions a change's record keeps at most (domain/forge.md, 8.11).
pub const QUESTIONS_OPEN: usize = 16;
/// Bytes of a question's first line kept on the record and shown on the panel.
pub const QUESTION_LINE: usize = 256;

/// A change's relay of its agents' questions (domain/forge.md, 8.11).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Questions {
    pub route: Route,
    /// Questions taken since the change was made or last released.
    pub taken: u32,
    /// Open questions in the order taken; the first is the one being worked.
    pub open: Box<[Open]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Open {
    /// The asking task: the change's producing task, a repair or a resolution.
    pub asker: u64,
    /// The question's number in the change's inbox, which its answer names.
    pub number: u64,
    /// Its words' first line, at most `QUESTION_LINE` bytes, cut on a UTF-8 boundary, marked "…".
    pub line: Box<[u8]>,
    pub stands: Stands,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Stands {
    /// Waiting its turn, its foreman's delegation or its passing up.
    Taken,
    /// With its foreman: `None` while the delegate is asked, its task once made.
    Foreman { task: Option<u64> },
    /// Asked of the change's requester, the change kept by its own question.
    Asked,
    /// Answered by the requester and held while the change is kept by a pending dependency.
    Held { answer: Box<[u8]> },
}
```

- On the connector's row: `temper_engine_domain_forge::ChangeRow.questions: change::Questions`
  and `ChangeRow.foreman_charter: u32` beside `review_charter`.
- In the store (`temper-protocol-store/schema/store-v11.schema`, in place at version 11 as
  reference-m1's tasks changed it): `enum Route { none, foreman, up }`; `enum Stands { taken,
  foreman: option u64, asked, held: bytes 65536 }` (or the schema's spelling for a variant
  with a value, as `ChangeHold` has); `record Open { asker: u64 number: u64 line: bytes 256
  stands: Stands }`; `record Questions { route: Route taken: u32 open: list 16 Open }`;
  `ChangeRow` gains `foreman_charter: u32` after `review_charter` and `questions: Questions`
  at its end. A held answer's words are at most the deployment's message bytes, which the
  schema's 65536 covers.
- The question's whole words are never on the record: they are read from the change's
  question positions (jig's `TaskRecord.question_lines`, its activity line) when a foreman is
  briefed or the question is passed up.

## C3. Parameter 14, the change's questions

- A change task's spec parameter 14, `Number`: 1 `foreman`, 2 `up`; absent, `none`. The root
  writes it only for `foreman` or `up`; `start_change` decodes it into
  `ChangeRow.questions.route`, and refuses any other value as it refuses an unknown
  parameter. Parameters 11 to 13 are left to the wave's other plans (reference-m2, -m3,
  -m7b), which add `change`'s parameters beside these.

## C4. The foreman's charter

- The stock's charter 6, a foreman's (`temper-service/src/stock.rs`, `foreman()`, its
  instructions `stock/foreman.txt`), read-only as charter 4 is: inspect, no modify, no shell,
  no agents. Kind 3's number 3 is fixed at charter 6 by the binary until temper's kinds are
  settings (configuration.md, 13.2), as number 2 is at charter 4.
- `temper-engine-domain`'s `route/forge_route.rs` holds `const FOREMAN_CHARTER: u32 = 6`;
  `start_change` sets `foreman_charter: FOREMAN_CHARTER` as it sets `review_charter: 4`; a
  change whose route is `foreman` is granted delegation to `Charter(row.foreman_charter)`.
- The stock's rules and policy 1 delegate to charter 6 beside 1, 2 and 4.

## C5. The foreman's contract and brief

- Its contract: `tasks::Contract::Verdict` with three choices, in order: code 1 `answer`,
  code 2 `finish`, code 3 `escalate`, each with words required and no fields. Its result's
  words are its text.
- Its spec: words, the brief below; parameters 1 (the change's repository resource) and 2
  (the change's landing branch), so its workspace is a checkout of the landing branch's tip;
  no parameter 3. Its authority: the review's (`gate_authority`: the read family and the
  forge's read grants on the change's repository), so it holds no Message family.
- Its brief, in order: `You are the foreman of change task C, a question from task T (its
  question N).`; a blank line; `The change's words:` and the change's spec words whole;
  `The question:` and the question's words whole; `The change's branch: B, pushed at H.`
  (or `The change has pushed nothing yet.`); then `Answer with a verdict: answer, finish or
  escalate.` The brief is bounded as a review's (the deployment's brief bytes), the change's
  words cut from their end and marked first, the question's never.

## C6. `change`'s `questions` member

- JSON: `"questions": "none" | "foreman" | "up"`, in each change's object beside `landing`
  and `after` (domain/agent.md, 4.3.1); a plan member, so a run without jig's control family
  naming it is refused as for the other plan members.
- `temper_engine_domain::ChangeCall.questions: Option<ChangeQuestions>`, with
  `pub enum ChangeQuestions { None, Foreman, Up }` in `boundary.rs`, exported as
  `ChangeCall` is; `ChangeCall::words` leaves it `None`.
- Decode (`temper-engine-smith/src/calls.rs`, `fn change`): any other string is
  `Problem::Range(b"questions")`, a non-string `Problem::Type(b"questions")`.
- The root (`route/forge_route.rs`, `change_batch` and `chat_change`): `None` takes the
  project's kind 6, which is `none` until temper's kinds are settings; `Foreman` or `Up`
  writes parameter 14 (C3). A change taking `Foreman` is given `Limits.questions` more tasks
  (one foreman each) beside its six, and delegation to its foreman's charter (C4).

## C7. The relay's step (`temper-engine-domain-forge-change`, `relay.rs`)

The relay is decided purely, beside `step`, and applied by the root and the connector:

```rust
/// A question one of the change's own agents asked, taken at this step.
pub struct Asked<'a> { pub asker: u64, pub number: u64, pub words: &'a [u8] }

/// How the open question's foreman ended.
pub enum Settled<'a> { Answer(&'a [u8]), Finish(&'a [u8]), Escalate(&'a [u8]), Failed(&'a [u8]) }

/// What a step heard for the relay.
pub struct RelayHeard<'a> {
    pub asked: &'a [Asked<'a>],
    pub foreman: Option<(u64, Settled<'a>)>,
    /// The requester's answer to the change's question, its words.
    pub answer: Option<&'a [u8]>,
    /// Askers among the open questions' that have ended.
    pub ended: &'a [u64],
    /// The change's task is kept by a pending dependency.
    pub kept: bool,
    /// The change was released from `Hold::Questions` since its last step.
    pub released: bool,
}

pub enum RelayAct {
    /// Delegate a foreman for the first open question.
    Foreman { asker: u64, number: u64 },
    /// Answer the asker's question with these words.
    Answer { asker: u64, number: u64, words: Box<[u8]> },
    /// Ask the requester the first open question, with the foreman's reasons if any.
    Ask { asker: u64, number: u64, reasons: Box<[u8]> },
    /// Cancel a foreman whose asker ended.
    Cancel { task: u64 },
    /// Hold the change past its bound.
    Hold,
}

pub struct Relayed {
    pub questions: Questions,
    pub acts: Box<[RelayAct]>,
    /// Work is left for a later step: the room ran out before the open questions settled.
    pub more: bool,
}

/// `room`: the messages (`Answer` and `Ask` acts) this step may still send, its batch
/// (`tasks.batch`) less the replies it already holds.
pub fn relay(questions: &Questions, heard: &RelayHeard<'_>, limits: &Limits, room: usize) -> Relayed;
```

- The `Finish` text opens with `Finish: the work done holds; list the rest under Left.` and a
  newline, added by `relay`; the passed-up reasons are `Foreman:` and a newline then the
  foreman's text or failure words, empty when no foreman ran. The root composes the question
  to the requester from the question's words whole and those reasons, cut from the end to
  the message bytes and marked.
- `relay` never reads the clock or the forge; `Questions.open` never exceeds
  `QUESTIONS_OPEN` entries: a question taken past it is answered as 8.1's change that takes
  none answers, by the root, and never reaches `relay`.
- A step whose `Relayed.more` is set is followed by another step at once: the root makes
  the change due again as it does when its facts change.

## C8. The panel's questions

- `temper_engine_domain_forge::Panel.questions: PanelQuestions { route: change::Route, taken:
  PanelUsed, open: Box<[PanelQuestion]> }`, `PanelQuestion { asker: u64, number: u64, line:
  Box<[u8]>, stands: PanelStands }`, `PanelStands { Taken, Foreman(Option<u64>), Asked, Held }`
  (a held answer's words are not shown), the first 8 open questions.
- On the wire, appended to `versioned record ChangePanel`: `questions: Questions` with
  `enum Route { none, foreman, up }`, `enum Stands { taken, foreman: option u64, asked, held }`,
  `record Question { asker: u64 number: u64 line: bytes 256 stands: Stands }`, `record
  Questions { route: Route taken: Used open: list 8 Question }`, in the wire schema's own
  spelling.
- `ctl change show` prints, on the line after `repairs … · updates …`, `questions: ROUTE ·
  TAKEN of LIMIT taken`, then one line per open question, `  question N of task T · STANDS
  · LINE`, STANDS one of `taken`, `with foreman F`, `with foreman` (not yet made), `asked of
  requester`, `answered, held for dependencies`; with route `none` and nothing taken, the
  one line `questions: none`. Its JSON carries `"questions": {"route", "taken", "limit",
  "open": [{"asker", "number", "line", "stands", "foreman"}]}`, `foreman` present only with
  a made foreman.

## Revisions

- **C2, revision 2 (task `relay-up`).** jig offers a procedure no reader of a question's
  words by number, so the record keeps them: `Open.words: Box<[u8]>`, the question's words
  whole (at most the deployment's message bytes), after `line`; the store's `record Open`
  gains `words: bytes 65536` after `line`. C2's bullet that the words are never on the
  record, and that they are read from the question positions, no longer holds; the
  question positions stay the history (`domain/forge.md`, 8.11, "Its history"). Task
  `relay-surface` builds C2 as first written; `relay-up` adds the field and its fallout.
- **C5, an option (task `relay-foreman`).** If the worker makes no workspace for a delegate
  without parameter 3, the foreman's spec carries parameter 3, the change's branch, as a
  review's does, its charter keeping it read-only; the report says so.
- **C2 and C7, a reader (task `relay-stall`).** The pure step's `Facts.asking` is true while
  `row.questions.open` holds a question whose asker is the task of `row.delegate`; a phase
  waiting on that delegate then waits with no deadline and no stall (`domain/forge.md`,
  8.4).
- **C2 and C7, revision 3 (task `relay-up`, which owns `relay.rs` and the store's
  `Stands`; it supersedes `relay-policy`'s rules where they differ).** A step's inputs are
  taken from jig's inbox whole whatever room it has (jig keeps nothing back of what a step
  read), so every input lands on the record before any act is sent:
  - `Stands` gains `Up { reasons: Box<[u8]> }`, a question to pass up with these reasons
    (store: `up: bytes 65536`, in place at v11 with revision 2's `words`). A route-`Up` head,
    a foreman's `Escalate` or `Failed` (reasons `Foreman:\n` and its text), and the head past
    the bound (reasons empty) become `Up`; an `Up` head is asked of the requester as soon as
    the step has room, the bound never checked again for it.
  - Past the bound the head becomes `Up { reasons: [] }` and the step acts `Hold` once; after
    the release, it goes up with no foreman.
  - The requester's answer, and a foreman's `Answer` or `Finish` text, make the head `Held {
    answer }` at once; a `Held` head goes down as soon as the step has room and the change
    has no pending dependency. So `Held` is "answered, to go down", and a room-short step
    loses nothing.
  - `heard.foreman` settles the head only when its task is the head's `Foreman { task:
    Some(f) }`; any other foreman's end is dropped.
  - A question already open by (asker, number) is not taken again.
  - `heard.released` goes: any release of the change sets `questions.taken` to 0, as a
    release resets repairs (the pure crate's release, `domain.rs` near line 223).
  - `heard.kept` is the change's task having a pending dependency (`waiting_on` not empty),
    not `landing_route::kept`, whose landing joins would let an answer down early.
  - Every `Answer` the root sends is cut from its end to `tasks.message_bytes` and marked
    `…`, as an `Ask` is.
  - The panel (C8) shows an `Up` head as `taken`: `relay-up` adds that arm where
    `questions-panel` maps `Stands`.
  - The relay's row change takes effect only with the step jig accepts: a refused step
    leaves `row.questions` as it was (apply `Relayed` with the accepted step, or undo it on
    the refusal as `DelegateAdmissionRefused` undoes a delegate); `route/asks.rs`'s refusal
    arms may be edited for that alone.
- **C5 and C6, revision 2 (task `relay-foreman`; `relay-story` for the world's sizing).**
  - Each foreman verdict choice's words are bounded by `tasks.message_bytes` less 64 (the
    `Finish` line's room), within `tasks.result_bytes`.
  - The foreman's step bookkeeping sits beside `forge_delegating`
    (`temper/crates/temper-engine-domain/src/domain.rs`, a map keyed by (task, step), which
    this revision lets `relay-foreman` add), since `forge_route.rs` holds one delegate per
    step in that map; a step whose foreman jig refuses settles the question `Failed` with
    the refusal's words, never the change's "Delegate refused" hold
    (`forge_route/holds.rs`'s `refused_step`).
  - `chat_change`'s executor kinds take up to four (a person gate and `Charter(6)` beside
    the change's two charters); its tasks are `(6 + questions bound).min(what the run has
    left − 1)` for a foreman change, a foreman left uncovered passing its question up.
  - The domain world (`temper/tests/engine/domain/src/forge_routes.rs`) gains a read-only
    charter 6 and `Charter(6)` in its rules' and policy's delegation, its executor limits
    raised to fit it, as `relay-foreman`'s stories need; `relay-story` sizes its own world's
    tasks for a goal and three foreman changes (1 + 3 × (1 + 6 + 4)) in its own setup, as
    `goal_changes.rs` sets its `tasks.batch`.
