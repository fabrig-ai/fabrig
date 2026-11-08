# ui-ask-m3a: the shapes that cross tasks

The run's binding calls. Where a spec and this note differ, this note
wins; say so in the report. The design they serve:
`temper/docs/design/domain/forge.md`, 8.13 (the path),
`temper/docs/design/protocol/configuration.md`, 13.10 (the stock ways),
`temper/docs/design/client/README.md`, 4.6, 5.13 and 5.16 (the wire, the
acts and the views), `temper/docs/design/domain/agent.md`, 8.4 and 8.4.1
(`see ways`, the instructions).

## 1. The crate `temper-engine-domain-ways`

`temper/crates/temper-engine-domain-ways`, a step crate (lint profile
`step`, `#![cfg_attr(not(test), no_std)]`, `alloc`), its manifest's
description "The stock ways and a branch's path, derived from the
settings". Dependencies: `skein-lib`, `temper-engine-domain-forge`,
`temper-engine-domain-forge-change`; nothing of jig's. The root
(`temper-engine-domain`) depends on it, and so do `temper-service`
(the stock's data, the family's encoding) and `temper-engine-smith`
(`see ways`'s text). All types derive `Clone, PartialEq, Eq, Hash, Debug`.

```rust
// lib.rs re-exports every item below at the crate's root.

/// Who acts at a stop (domain/forge.md, 8.13).
pub enum Actor { Person, Agent, Code, Bar }

/// What a stop is, one per kind of step of 8.13's order.
pub enum Step {
    Ask, Draft, Plan, Produce, ProduceMany, Check, Review, Repair, Approve,
    Update, Resolve, Merge, Prepare, Prove, Turn, Verify, Branch,
}

/// What a stop opens.
pub enum Opens {
    Nothing,
    Charter(u32),
    /// kind 1 for a rule of scope 0, else kind 2.
    Rule { kind: u32, scope: u32, number: u32 },
    Gate(u32),
    Tier(Box<[u8]>),
    Queue,
    Turn,
}

pub struct Fact { pub label: Box<[u8]>, pub value: Box<[u8]> }

/// A stop's card (8.13, *A stop's card*): at most `FACTS` facts.
pub struct Card { pub who: Box<[u8]>, pub name: Box<[u8]>, pub facts: Box<[Fact]> }

pub struct Stop { pub actor: Actor, pub step: Step, pub words: Box<[u8]>, pub opens: Opens, pub card: Card }

/// `repository` is `OWNER/NAME`, none for a project with no repository.
pub struct Path { pub repository: Option<Box<[u8]>>, pub branch: Box<[u8]>, pub stops: Box<[Stop]> }

pub const STOPS: usize = 32;   // a path's most stops
pub const FACTS: usize = 4;    // a card's most facts
pub const WORDS: usize = 64;   // a stop's words' most bytes
pub const WHO: usize = 128;    // a card's `who`
pub const NAME: usize = 64;    // a card's `name`
pub const LABEL: usize = 32;   // a fact's label
pub const VALUE: usize = 384;  // a fact's value

impl Stop { pub fn same(&self, other: &Stop) -> bool }   // actor, step, words and opens equal; cards ignored
impl Path { pub fn same(&self, other: &Path) -> bool }   // every stop `same`, in order, and as many

/// How work starts on the project (kind 7 and its way's template).
pub enum Start { Chat, Goal { charter: u32 } }

pub struct Tier { pub name: Box<[u8]>, pub follows: temper_engine_domain_forge::Follows }

/// The repository the path is over, as the forge holds it.
pub struct Repository {
    pub host: Box<[u8]>,          // the forge's host, as `covers_branch` takes it
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub ci: bool,                 // adopted with CI
    pub checks: Box<[u32]>,       // its checks, for a repository adopted without CI
    pub gates: Box<[Box<[u8]>]>,  // its test gates' names (landing by turn)
    pub tiers: Box<[Tier]>,
}

/// A role of the project's policy: its name and whether the reader holds it.
pub struct Role { pub number: u32, pub name: Box<[u8]>, pub yours: bool }

/// What an agent's card says of a charter, gathered by the root.
pub struct Agent {
    pub charter: u32,
    pub model: Option<Box<[u8]>>,   // the model's shown name, else its name
    pub effort: Option<Box<[u8]>>,
    pub usable: bool,               // the reader's offer holds an account for it
    pub can: Box<[u8]>,             // `can(...)`'s words
    pub asked: Box<[u8]>,           // `first_sentence(...)` of its instructions
}

pub struct Inputs {
    pub branch: Box<[u8]>,                                        // the landing branch, as `main`
    pub repository: Option<Repository>,
    pub rules: Box<[temper_engine_domain_forge::settings::Held]>, // every rule held for the project, items laid over
    pub start: Start,
    pub producer: u32,                                            // kind 3 number 1
    pub reviewer: u32,                                            // kind 3 number 2
    pub by: temper_engine_domain_forge_change::Way,
    pub roles: Box<[Role]>,
    pub agents: Box<[Agent]>,                                     // one per charter the path names
}

/// domain/forge.md, 8.13: pure, total, at most STOPS stops, ending at a bar.
pub fn derive(inputs: &Inputs) -> Path;

/// A stock way (configuration.md, 13.10).
pub struct StockWay {
    pub name: Box<[u8]>,
    pub line: Box<[u8]>,
    /// The project's rules it sets, by number, each of 1 to `RULES`; every other number of 1 to `RULES` it removes.
    pub rules: Box<[(u32, temper_engine_domain_forge::settings::LandingRule)]>,
    pub ways: Option<Laid>,
    /// Whether the way needs charter 5, role 0 and job 3 in the project's policy's delegation.
    pub delegation: bool,
}
/// The members of kind 7 a way lays over the project's own.
pub struct Laid { pub drafts: bool, pub goals: bool, pub way: u32 }
pub const RULES: u32 = 4;

/// An item of temper's kinds, as jig's `ChangeSettings` carries one.
pub struct Item { pub kind: u32, pub scope: u32, pub number: u32, pub edit: Edit }
pub enum Edit { Set(temper_engine_domain_forge::settings::Value), Remove }

/// The way's items for `project`: rule items in number order 1 to RULES, then kind 7's where the way lays it.
/// `ways` is the project's kind 7, none for the stock's (`settings::project_stock(7)`).
pub fn items(way: &StockWay, project: u32, ways: Option<&temper_engine_domain_forge::settings::ProjectWays>) -> Box<[Item]>;

/// The project's held rules with `items` laid over, as the forge would hold them after the decision:
/// a set rule replaces or joins that number; a removed one leaves it, the stock's `every_project` rule of that number
/// (held with the project's scope) taking its place where there is one. Deployment rules (scope 0) stay.
pub fn laid(rules: &[temper_engine_domain_forge::settings::Held], stock: &[temper_engine_domain_forge::settings::Held], project: u32, items: &[Item]) -> Box<[temper_engine_domain_forge::settings::Held]>;
/// Kind 7 with `items` laid over; the stock's where none is set.
pub fn laid_ways(ways: Option<&temper_engine_domain_forge::settings::ProjectWays>, project: u32, items: &[Item]) -> temper_engine_domain_forge::settings::ProjectWays;

/// The first way whose path is `same` as `own`.
pub fn current(ways: &[Path], own: &Path) -> Option<u32>;

/// The words of a charter's tool families and of its instructions' first sentence, for an agent's card.
pub fn can(inspect: bool, modify: bool, shell: bool, agents: bool) -> &'static [u8];
pub fn first_sentence(instructions: &[u8]) -> &[u8];   // to the first ". " or ".\n", trimmed, at most VALUE bytes at a char's end

/// The watched subject of a project's ways: outside every other subject's bits.
pub const fn subject(project: u32) -> u64 { (1 << 62) | (1 << 38) | project as u64 }  // written without `as` in the code

pub struct WayEntry { pub name: Box<[u8]>, pub line: Box<[u8]>, pub path: Path, pub items: Box<[Item]>, pub delegation: bool }
pub struct WaysDocument { pub project: u32, pub ways: Box<[WayEntry]>, pub current: Option<u32>, pub own: Path, pub may_change: bool }

/// A view's answer (client/README.md, 5.16).
pub enum View {
    Way { name: Box<[u8]>, line: Box<[u8]>, path: Path, current: bool },
    Path(Path),
    Proposal { before: Path, after: Path },
    Charter { charter: u32, card: Card },
    NotDrawn(Box<[u8]>),   // one line, at most 512 bytes
}
```

`can`'s words, binding: the families a charter names joined by "; ", from
"read the code and the forge" (inspect), "change and push its branch"
(modify), "run commands" (shell), "start sub-agents" (agents), then
"never merge or update", always last; with none, "finish alone; never
merge or update".

## 2. The words of a path

Binding for `derive` (task `ways-derive`); the look request's pages
(`origin/look-requests`, `c342a9f113`, `ask-temper-ways/01.html` to
`03.html`) use them.

| Step | Actor | Words | Opens | Card `who` | Card `name` | Facts |
|---|---|---|---|---|---|---|
| Ask | Person | "you ask" | Nothing | "a person · asks for the change" | "you" | "Starts": "a chat" |
| Draft | Person | "draft" | Nothing | "a person · writes the draft with an agent" | "you" | "Starts": "a goal for a draft" |
| Plan | Agent | "plan" | Charter(the template's) | "an agent · cuts the draft into changes" | "the coordinator" | Model, Can, Asked to |
| Produce | Agent | "produce" | Charter(producer) | "an agent · makes the change" | "the coding agent" | Model, Can, Asked to |
| ProduceMany | Agent | "produce ×n" | Charter(producer) | "agents · make the goal's changes, in parallel" | "the coding agent" | Model, Can, Asked to |
| Check (CI) | Code | "check" | Rule(the first covering rule asking CI) | "code · holds a change until CI passes" | "CI" | "Checks": "the repository's CI, at the change's head" |
| Check (a repository's check gate) | Agent | "check" | Gate(n) | "an agent · runs the repository's check N" | "the coding agent" | Model, Can, Asked to |
| Review | Agent | "review" | Gate(n) | "an agent · holds a change until it approves", or "an agent · advises; holds nothing" for an advisory gate | "the reviewer" | Model, Can, Asked to |
| Repair (a change's) | Agent | "repair" | Charter(producer) | "an agent · repairs what a check or a review found, then goes back through them" | "the coding agent" | Model, Can, Asked to |
| Repair (a goal tier's) | Agent | "repair" | Charter(the template's) | "an agent · repairs a red on the goal's branch" | "the coordinator" | Model, Can, Asked to |
| Approve | Person | "you approve", or "a ROLE approves" / "an ROLE approves" by the role name's first letter; then " ×N" where N > 1 | Rule(the rule asking it) | "a person · decides whether it lands" | the role's name | "Role": the role's name; "Asks": "N of them" |
| Update | Code | "update" | Queue | "code · brings the change up to date with its branch" | "the queue" | "Does": "merges the branch's tip into the change, never by an agent" |
| Resolve | Agent | "resolve" | Charter(producer) | "an agent · settles a conflict with its branch" | "the coding agent" | Model, Can, Asked to |
| Merge | Code | "merge" | Queue | "code · lands the change" | "the queue" | "Does": "merges exactly the commit its checks saw" |
| Prepare | Code | "prepare" | Turn | "code · puts the change on its branch's tip" | "the turn" | "Does": "rebases the change onto its branch's tip" |
| Prove | Code | "prove" | Turn | "code · runs the test gates at the candidate" | "the turn" | "Gates": the gates' names joined by ", " |
| Turn | Code | "turn" | Turn | "code · lands the change" | "the turn" | "Does": "moves the branch to exactly the proved candidate" |
| Check (a goal's tier) | Code | "check" | Tier(name) | "code · checks the goal's branch" | the tier's name | "Runs": "the tier NAME on the goal's branch" |
| Verify | Code | "verify" | Tier(name) | "code · checks what landed" | the tier's name | "Runs": "the tier NAME after each landing" |
| Branch (a goal's) | Bar | "goal branch" | Nothing | "a branch · holds the goal's changes until it lands whole" | "goal branch" | none |
| Branch (the landing branch) | Bar | the branch's name, as "main" | Nothing | "a branch · where the change lands" | the branch's name | none |

An agent's facts: "Model": "MODEL · EFFORT" (either alone where the
other is none; "no model" with neither), then " · no account runs it now"
where `usable` is false; "Can": the agent's `can`; "Asked to": its
`asked` in “ and ”. An agent the inputs give no `Agent` for has the facts
"Charter": "charter N" alone.

A goal's segment is judged against the branch `temper/0/g0`, as a goal's
own branch is named (`forge.md`, 11.1), and always lands by queue; the
landing branch's segment by the project's `by`.

## 3. The wire (task `ways-wire`)

In `temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema`,
`CLIENT_VERSION` unchanged (it moves at a release: client/README.md, 4.6),
new variants last in their enums:

```
# A branch's path (temper/docs/design/domain/forge.md, 8.13; client/README.md, 4.6).
enum Actor { person, agent, code, bar }
enum Step { ask, draft, plan, produce, produce_many, check, review, repair, approve, update, resolve, merge, prepare, prove, turn, verify, branch }
record OpensCharter { charter: u32 }
record OpensRule { kind: u32 scope: u32 number: u32 }
record OpensGate { number: u32 }
record OpensTier { name: bytes 32 }
enum Opens { nothing, charter: OpensCharter, rule: OpensRule, gate: OpensGate, tier: OpensTier, queue, turn }
record Fact { label: text 32 value: text 384 }
record Card { who: text 128 name: text 64 facts: list 4 Fact }
record Stop { actor: Actor step: Step words: text 64 opens: Opens card: Card }
record Path { repository: option bytes 2048 branch: bytes 1024 stops: list 32 Stop }
# A project's stock ways, watched (client/README.md, 4.6; configuration.md, 13.10).
record Way { name: text 64 line: text 256 path: Path items: list 8 ProposalItem delegation: bool }
record WaysSubject { project: u32 }
versioned record WaysPanel { project: u32 ways: list 4 Way current: option u32 own: Path may_change: bool }
# A view, read when shown (client/README.md, 4.6 and 5.16).
record ViewRead { project: u32 address: text 512 }
record WayView { name: text 64 line: text 256 path: Path current: bool }
record ProposalView { before: Path after: Path }
record CharterView { charter: u32 card: Card }
record NotDrawn { words: text 512 }
enum View { way: WayView, path: Path, proposal: ProposalView, charter: CharterView, not_drawn: NotDrawn }
versioned record ViewAnswer { view: View }
```

`enum Watch` gains `ways: WaysSubject`, last; `enum Read` gains `view:
ViewRead`, last. `lib.rs` gains `ways_panel` / `decode_ways_panel` and
`view_answer` / `decode_view_answer`, as `changes_panel` /
`decode_changes_panel` are. A `Way`'s `items` are what "Use this" sends:
each `ProposalItem`'s `edit` `set` with the kind's value encoded as
`temper_engine_smith::kinds::encode` encodes it, or `removed`.

## 4. The root's boundary (task `ways-surface`)

In `temper-engine-domain`:

- `translate::Config` gains `ways_stock: Box<[temper_engine_domain_ways::StockWay]>`,
  beside `kinds_stock`: the stock's ways, in their order.
- `ForgeRead` gains `View { project: u32, address: Box<[u8]> }`;
  `ForgeReadAnswer` gains `View(Box<temper_engine_domain_ways::View>)`.
  A party with a role in `project` may read it, as kinds 2, 6 and 7 are
  read in a project's scope.
- `decision::Delivery` gains `Ways { watcher: Token, document: Box<temper_engine_domain_ways::WaysDocument> }`,
  as `Approvals` is.
- `route::see::SeeAnswer` gains `Ways(Box<temper_engine_domain_ways::WaysDocument>)`.
- The root re-exports the crate as `temper_engine_domain::ways`.

In `temper-service`: `stock::Given::Own` gains `ways: Box<[temper_engine_domain_ways::StockWay]>`
(task `stock-ways`), and `stock::ways() -> Box<[temper_engine_domain_ways::StockWay]>`
gives the three of 13.10.

## 5. The client's view blocks (task `client-views`)

In `temper-client-domain`, module `views`:

```rust
/// One part of a text: Markdown to read, or a view block's address.
pub enum Part<'a> { Text(&'a str), View(&'a [u8]) }
/// Split at top-level fenced blocks whose info string is exactly `view`
/// and whose body is one line (client/README.md, 5.16).
pub fn parts(text: &str) -> List<Part<'_>>;   // at most `limits.view_parts` parts; the rest is one Text
```

The chat page (`pages::chat`) keeps each address it shows in a
`Views` map, address to `Shown`: `Asking`, `Drawn(Box<v1::View>)`,
`Failed`. `Chat::view(&self, address: &[u8]) -> Option<&Shown>` is what
`temper-client-view` draws from.

## 6. The client's ways (task `client-ways`)

In `temper-client-domain`, module `forge::stock_ways` (the name
`forge::ways` is kind 7's): the Ask page's watch of `ways` while
`Address::Ask { project }` is open; `pages::Act::UseWay { project, way:
u32 }` and `pages::Act::KeepWay { project }`; a refusal kept on the page
until the next act or a newer document. `Pages::stock_ways(&self) ->
Option<&StockWays>` where `StockWays` gives `tabs()`: each way with its
index, words, path and `current`, then `Yours` where `current` is none;
`act(index) -> Option<WayAct>`, `Use`, `Keep` or none (the rules of
`client/README.md`, 5.13).
