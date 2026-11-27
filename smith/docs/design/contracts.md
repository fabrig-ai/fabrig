# Contracts of smith's drawn vocabulary, while it is built

Provisional, 2026-11-27. The shapes the changes building [drawn-agent.md](drawn-agent.md) and [shapes.md](shapes.md) share, and no other area reads (`temper/docs/design/draft/design-economy.md`, 4.3). They bind as any delivered design section does. Once those changes have landed, the code holds them and the goal's last change deletes this page and its status.

## 1. The vocabulary's code

`smith-session-world`'s public module `shapes` (`smith/tests/session/src/shapes.rs`), which the session, agent, host and local worlds name. Its words are shapes.md, 3's; each enumeration's `word` gives a variant's word.

```rust
pub const VOCABULARY: &str = "smith/1";

pub enum Tool { Read, List, Search, Write, Edit, Remove, Shell, Job, Delegated, Unknown }
/// One variant for each answer word of shapes.md, 3, and `Unknown`.
pub enum Answer { /* ... */ }
pub enum Reference { Path, Offset, Job }
/// The world's `JobClass`, moved here and named for every reference;
/// `Stale` added by the change that classes paths (section 2).
pub enum Class { Live, Held, Consumed, BeforeOpening, Foreign, Unissued, Stale, Unknown }

impl Tool { pub const fn word(self) -> &'static str; }
impl Answer { pub const fn word(self) -> &'static str; }
impl Reference { pub const fn word(self) -> &'static str; }
impl Class { pub const fn word(self) -> &'static str; }

/// What came of a call: `None` where its completion's end left it unanswered.
pub fn answer(returned: Option<&smith_domain_session::llm::Returned>) -> Answer;

/// One call of a completion, as shapes.md, 2 counts it.
pub struct Call { pub tool: Tool, pub answer: Answer, pub references: Vec<(Reference, Class)> }

/// What one case reached: its completions' shapes, and the answers and classes in them.
pub struct Reached { /* ... */ }
impl Reached {
    pub fn new() -> Reached;
    /// One completion's calls, in order; none for a completion with no call.
    pub fn completion(&mut self, calls: &[Call]);
    pub fn answered(&self, answer: Answer) -> bool;
    pub fn classed(&self, reference: Reference, class: Class) -> bool;
}

/// What a sweep's cases reached: skein's `Shapes` in `VOCABULARY`, and the cases
/// in which each answer and each class fell.
pub struct Sample { /* ... */ }
impl Sample {
    pub fn new(sweep: &'static str) -> Sample;
    pub fn ran(&mut self, case: &Reached) -> Result<(), String>;
    pub fn cases(&self) -> u64;
    pub fn answered(&self, answer: Answer) -> u64;
    pub fn classed(&self, reference: Reference, class: Class) -> u64;
    /// skein's lines, then `shapes answer SWEEP WORD N` and
    /// `shapes class SWEEP REFERENCE CLASS N` for every word, nought included.
    pub fn write(&self);
}
```

- **The session world gives each case's** `Reached` from its sessions' recorded turns: `World::reached(&self) -> Reached`, every completion of every session and sub-agent, each call with the references its record holds.
- **A call's references** are recorded beside its answer in the world's record of the call, as `JobCall`'s class is now: the job's from the world's classification of drawn jobs, the path's and the offset's from drawn-agent.md, 5's.
- **A world that classes nothing** (the agent, host and local worlds) folds a run's recorded turns:

```rust
impl Reached {
    /// Each assistant message's calls in `turns`, in order, each answered by the
    /// result of its id in the turns, `Answer::Unanswered` where none is; each call
    /// carrying a path, an offset or a job, decoded as the session world's translator
    /// decodes it, carries that reference classed `Class::Unknown`.
    pub fn turns(&mut self, turns: &[smith_domain_session::record::Turn]);
}
```

  A withheld `wait`, whose result comes with the next activation, is unanswered in the turns that hold its call alone.

## 2. Ownership while it is built

- **The module and `World::reached`:** the change that makes them; jobs' references alone.
- **Paths' and offsets' references,** `Class::Stale` among their classes: the change that adds the menu's bodies and slots.
- **The reach over answers and classes:** each change that adds a refusal's setup asks for its answer through `Sample::answered`.
- **`Reached::turns`,** and the agent, host and local worlds' sweeps folding it: the change that gives those worlds the vocabulary. Where `World::reached` walks records the same way, that change may make it call `Reached::turns`'s walk, keeping its classes.
