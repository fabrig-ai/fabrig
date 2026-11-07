# per-wake: the shapes that cross tasks

The run's binding calls. Every task builds to these names, fields and
signatures; inside them the choices are its own. The design is jig's
`jig/docs/design/domain/tasks.md`, section 3 and 8.6, and the sections it cites.

## 1. A run's reasons: `jig_domain_tasks::Reasons`

New module `jig/crates/jig-domain-tasks/src/reasons.rs`, exported at the crate
root (`pub use reasons::{Reasons, REASON_NAMES, ALWAYS_OFFERED_NAMES, offered};`).

```rust
/// Why a run runs, and the reasons a charter's part is for: `first` and the kinds that
/// choose parts, one bit each (domain/tasks.md, 8.6; protocol/store.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Reasons(pub u32);

impl Reasons {
    pub const EMPTY: Self = Self(0);
    pub const FIRST: Self = Self(1 << 0);
    pub const QUESTION: Self = Self(1 << 1);
    pub const QUESTION_ANSWERED: Self = Self(1 << 2);
    pub const QUESTION_ENDED: Self = Self(1 << 3);
    pub const QUESTION_OWED: Self = Self(1 << 4);
    pub const ESCALATION: Self = Self(1 << 5);
    pub const PROPOSAL: Self = Self(1 << 6);
    pub const PERSON_STALLED: Self = Self(1 << 7);
    pub const RESULT: Self = Self(1 << 8);
    pub const NOTICE: Self = Self(1 << 9);
    pub const NOTHING_AWAITED: Self = Self(1 << 10);
    pub const NEWS: Self = Self(1 << 11);
    pub const TIMER: Self = Self(1 << 12);
    pub const WORDS: Self = Self(1 << 13);
    /// Every known bit: the kinds a run under a charter with no parts is offered.
    pub const ALL: Self = Self((1 << 14) - 1);

    pub const fn union(self, other: Self) -> Self;
    /// Whether the two share a bit.
    pub const fn meets(self, other: Self) -> bool;
    pub const fn contains(self, other: Self) -> bool;
    pub const fn is_empty(self) -> bool;
    /// No bit outside `ALL`.
    pub const fn is_known(self) -> bool;
    /// The bit of a kind that chooses parts; `EMPTY` for an always-offered kind.
    pub fn of(kind: &MessageKind) -> Self;
    /// `first` or a choosing kind by its store name; `None` for any other name,
    /// an always-offered kind's among them.
    pub fn named(name: &[u8]) -> Option<Self>;
}

/// The store names of `first` and the choosing kinds, in bit order.
pub const REASON_NAMES: [&[u8]; 14] = [b"first", b"question", b"question_answered",
    b"question_ended", b"question_owed", b"escalation", b"proposal", b"person_stalled",
    b"result", b"notice", b"nothing_awaited", b"news", b"timer", b"words"];
/// The always-offered kinds' store names, which a part's `on` refuses.
pub const ALWAYS_OFFERED_NAMES: [&[u8]; 7] = [b"answer", b"proposal_decision",
    b"settings_decision", b"effect_not_made", b"met", b"amendment", b"finish_requested"];

/// Whether `word` goes to a run offered `offered` (domain/tasks.md, 8.6): an always-offered
/// kind, an entry (`Escalation`, `Proposal`) and a person's `Words` (`from` is
/// `Party::Person`) always; any other kind when `offered` meets `Reasons::of` its kind.
pub fn offered(word: &Word, offered: Reasons) -> bool;
```

`MessageKind` to bit: `Question` 1, `QuestionAnswered` 2, `QuestionEnded` 3,
`QuestionOwed` 4, `Escalation` 5, `Proposal` 6, `PersonStalled` 7, `Result` 8,
`Notice` 9, `NothingAwaited` 10, `News` 11, `Timer` 12, `Words` 13; `Answer`,
`ProposalDecision`, `SettingsDecision`, `EffectNotMade`, `Met`, `Amendment`,
`FinishRequested` none. Every `match` lists every variant.

## 2. The task's reasons and its claim's offered kinds

- `jig_domain_tasks::TaskRecord` gains `pub reasons: Reasons` (why its activation
  runs: `FIRST` when made, kept across a failed attempt and a run parked kept,
  `EMPTY` once a run parks idle or finishes, a claim setting it to the run's
  reasons) and `pub offered: Reasons` (the kinds its claimed attempt is offered;
  `Reasons::ALL` when made, a claim setting it).
- `jig_domain_tasks::RunContext` gains `pub reasons: Reasons`: the run's reasons,
  the record's `reasons` and every kind that wakes it at this activation (8.6).
- `jig_domain_tasks::Event::Claim` gains `reasons: Reasons` (the prepared run's,
  from its `RunContext`) and `offered: Reasons` (what its preparation's parts
  offer). The claim stores both on the record in its own commit.
- `jig_domain_tasks::CharterModels` gains `pub resumes: bool`, the charter
  version's `policy.resume`, filled wherever the core builds one.
- `jig_domain_tasks::Limits` gains `pub charter_parts: u32`; the limit's name is
  `tasks.charter_parts` (`jig-protocol-config/src/names.rs`), 8 in both profiles
  and every fixture.

## 3. A charter's parts: `jig_domain_core::CharterPart`

In `jig/crates/jig-domain-core/src/settings/charters.rs`, exported at the crate root:

```rust
/// A charter's part: instructions for the runs whose reasons meet `on` (domain/tasks.md, 8.6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CharterPart {
    /// Nonempty, known bits (`Reasons::is_known`).
    pub on: jig_domain_tasks::Reasons,
    pub instructions: Box<[u8]>,
}
// Charter gains, after `documents`:
pub parts: Box<[CharterPart]>,
```

In `jig/crates/jig-domain-core/src/limits.rs`, exported beside `charter_owned_bytes`:

```rust
/// The bytes a charter's parts add to its stored policy: each part's own size, its
/// instructions and the two bytes of the blank line that joins them (configuration.md, 6).
pub fn parts_owned_bytes(parts: &[crate::CharterPart]) -> Option<u64>;
```

`charter_owned_bytes` adds `parts_owned_bytes(&charter.parts)`. A charter's stored
policy is within the run bound when `policy_owned_bytes(&charter.policy)` plus
`parts_owned_bytes(&charter.parts)` is at most `core.run_bytes`.

## 4. A run's instructions and offered kinds (core)

In `jig/crates/jig-domain-core/src/routing/run/` (the parts-core task's), used at
preparation:

- The run's instructions: the task's own instructions when it has some; else its
  charter's `policy.instructions`, then, for each part in order whose `on` meets
  the run's reasons, `b"\n\n"` and the part's instructions.
- The run's offered kinds: `Reasons::ALL` for a charter with no parts or a task with
  its own instructions; else the union of the `on` of the parts it reads, without
  `FIRST` (the always-offered need no bit).
- Start messages: the context's inbox filtered by `jig_domain_tasks::offered(word,
  offered)`, then the first `run_inbox` of them.

## 5. A task's `later`

```rust
// jig_domain_tasks::llm, exported at the crate root:
/// The choice every run whose reasons hold no `first` takes (domain/tasks.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LlmLater {
    pub main: ModelEffort,
    /// Each named once, at most `MODELS`.
    pub subagents: Box<[ModelEffort]>,
}
// LlmChoice gains, after `subagents`:
pub later: Option<LlmLater>,

// jig_domain_people::llm, exported at the crate root, the same shape over its own
// `ModelEffort`:
pub struct LlmLater { pub main: ModelEffort, pub subagents: Box<[ModelEffort]> }
// people's LlmChoice gains `pub later: Option<LlmLater>`.
```

- A run takes `later` when its task's `reasons` hold no `FIRST` and its charter at
  the run's revision doesn't resume, as `LlmChoice { account, main: later.main,
  subagents: later.subagents, later: None }`; otherwise the first choice.
- Refusal of a `later` under a charter whose runs resume, at a batch, an amendment
  or a template's check: `Refusal::Llm { model: later.main.model, why:
  LlmFault::Charter }` (people's `LlmWhy::Charter`). No new refusal vocabulary.
- A run's `delegate` and `amend` calls (`jig-protocol-tools`) give `later: None`;
  a change's `llm` in temper gives `later: None`.

## 6. Stored records (formats change in place; versions stay)

- `settings-v1.schema`: `record CharterPart { on: u32 instructions: bytes 65536 }`;
  `Charter` gains `parts: list 8 CharterPart` after `documents`; `record LlmLater {
  main: ModelEffort subagents: list 64 ModelEffort }`; `LlmChoice` gains `later:
  option LlmLater`.
- `tasks-v9.schema`: `record LlmLater { main: ModelEffort subagents: list 64
  ModelEffort }`; `LlmChoice` gains `later: option LlmLater`; the live task's record
  gains `reasons: u32` and `offered: u32`.
- `people-v14.schema`: `LlmLater` as above; its `LlmChoice` gains `later: option
  LlmLater`.
- A decoder refuses a `reasons`, `offered` or `on` with a bit outside `Reasons::ALL`,
  and an empty `on`, as its family refuses any value out of shape.
- temper's `store-v11` keeps its change `llm` without `later`: its adapter writes a
  choice whose `later` is `None` and reads one back with `later: None`.

## 7. Configuration (`jig-protocol-config`)

- A charter's `parts`: `[{"on": ["first", "question"], "instructions": "part.md"}]`,
  optional, none when absent; `instructions` a configuration path read as the
  charter's own is, each through the same file list. Refusals, each `What` the
  crate already has for the like case: `charters[i].parts[j].on[k]` for an unknown
  name, an always-offered kind or a repeated name; `charters[i].parts[j].on` for an
  empty list; `charters[i].parts[j]` past `tasks.charter_parts`; `charters[i].parts`
  on a charter with `"resume": true`; `charters[i]` as `What::Bound` when its policy
  and parts pass `core.run_bytes`.
- A template's `llm`: `{"main": {...}, "sub_agents": [...], "later": {"main": {...},
  "sub_agents": [...]}}`, `later` optional, named and resolved as `main` and
  `sub_agents` are.

## 8. The client's records (`jig-client-wire 19`, in place)

- `client-v19-part1.schema`: `record CharterPart { on: u32 instructions: <the bound
  of Charter.instructions> }`; `Charter` gains `parts: list 8 CharterPart` after
  `documents`, its count bounded by a wire limit `charter_parts` (ceiling 8), as
  `documents` is by `charter_documents`.
- `client-v19.schema`: `record LlmLater { main: ModelEffort subagents: list 64
  ModelEffort }`; `LlmChoice` gains `later: option LlmLater`.
- `jig_client_domain`: `CharterPart { on: u32, instructions: Text }` and
  `Charter.parts: List<CharterPart>` (`contract/settings.rs`); `LlmLater { main:
  ModelEffort, subagents: List<ModelEffort> }` and `LlmChoice.later:
  Option<LlmLater>` (`contract/llm.rs`); `pub const REASON_NAMES: [&str; 14]`, the
  bit order of section 1, beside `CharterPart`.
- The engine's translation carries both whole, both ways. A settings change whose
  part has an empty or unknown `on` is the core's to refuse (the parts-core task):
  `Malformed::Settings { setting: Charter(n), fault: Shape, detail: None }`, and a
  part past `tasks.charter_parts` with `fault: Count`.
- A task's document's `llm` is `document_llm`'s: the next run's choice (section 5),
  with its `later` beside it while the task's reasons hold `first`.
