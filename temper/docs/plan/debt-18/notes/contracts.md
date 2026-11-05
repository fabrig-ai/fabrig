# debt-18: the shapes that cross tasks

The run's binding calls. Inside them the choices are yours. D035's shapes are in `skein/docs/design/codec.md`, 11.6.

## 1. The protocol-world kit (skein D032)

`skein/testing/skein-protocol-kit`, depending on `skein-lib` alone. The name isn't `skein-protocol-world`, which is the
protocol world's own package, and doesn't end in `-world`: a kit isn't a world.

```rust
// skein_protocol_kit::below: whether bytes answer a demand
pub fn meets(read: Read, bytes: &[u8]) -> bool;
pub fn delivers(read: Read, bytes: &[u8]) -> bool;

// skein_protocol_kit::random: each is one Rng call, as every copy makes it
pub fn fault(rng: &mut Rng) -> Fault;                 // rng.below(3) over [Reset, Invalid, Other]
pub fn draw(rng: &mut Rng, low: u32, high: u32) -> u32;

// skein_protocol_kit::seen: how often each thing a sweep injects or reaches fell
#[derive(Default, Debug)]
pub struct Seen(pub BTreeMap<String, u32>);
impl Seen {
    pub fn note(&mut self, what: String);
    pub fn assert_fell<S: AsRef<str>>(&self, wanted: &[S]);   // panics "{what} fell: {:#?}", as the copies do
}

// skein_protocol_kit::transcript: the directory scan; records and `parse` stay each world's
pub struct Kept { pub name: String, pub bytes: Vec<u8>, pub expect: Option<Vec<u8>> }
pub fn kept(directory: &Path, extension: &str) -> Vec<Kept>;  // sorted by name
pub fn quote(bytes: &[u8]) -> String;
pub fn quoted(text: &[u8]) -> Option<(Vec<u8>, &[u8])>;
pub fn unquote(text: &[u8]) -> Option<Vec<u8>>;
pub fn word(text: &[u8]) -> (&[u8], &[u8]);
```

- **Each world's `below_acts` stays its own.** They aren't copies: the order of their draws differs (TLS draws an arrival
  with no room, and checks failure first), and a shared one would change every seeded schedule. Only these functions move.
- **A local `impl Seen` can't stay in a test file** (the orphan rule). `fn record(&mut self, ..)` becomes
  `fn record(tally: &mut Seen, ..)`, and `self.note` becomes `tally.note`.
- **Only the same record moves.** These are other things under one word, and stay: `http/src/client_world.rs`,
  `http/src/server_world.rs`, `protocol/src/client_end.rs`, `protocol/src/server_end.rs`, `skein-browser/src/boundary.rs` (an
  API in `skein/docs/design/browser.md`), `skein-echo-client/src/lib.rs`, `llm/tests/suite/bounded_peer_memory.rs`,
  `fake-peers/tests/suite/llm.rs`, `io/src/joined.rs`.

## 2. One type per operation (jig D012)

In `jig_domain_tasks`, one public type for each operation, and `Operation` their sum. Each entrance carries the one it accepts,
with its own context beside it.

```rust
pub struct Cancel { pub task: u64, pub reason: Box<[u8]>, pub grace: Option<Duration>, pub message: Option<u64> }
pub struct Release { pub task: u64, pub again: Option<bool>, pub message: Option<u64> }
pub struct Amend { pub task: u64, pub message: u64, pub stop_run: bool, pub amendment: Box<Amendment> }
pub struct Move { pub task: u64, pub reason: Box<[u8]> }
pub struct Message { pub task: u64, pub word: Word }

pub enum Operation { Cancel(Cancel), Move(Move), Amend(Amend), Release(Release), Message(Message), Stop { task: u64, hold: u64 } }

// Event: `Control` splits in two, and the others take their part
Cancel { reply_to: ReplyTo, by: Party, operation: Cancel },
Release { reply_to: ReplyTo, by: Party, operation: Release },
Amend { places: Box<[DependencyPlace]>, reply_to: ReplyTo, by: Party, operation: Amend, funder: Option<Funder> },
Move { reply_to: ReplyTo, person: u64, period: u64, pool_budget: u64, period_budget: u64, operation: Move },
Message { reply_to: ReplyTo, project: u32, operation: Message },
AuthorizedMessage { reply_to: ReplyTo, project: u32, operation: Message },

// ProcedureDecision
amendments: Box<[Amend]>, cancels: Box<[Cancel]>, messages: Box<[Message]>,

// jig_domain_core::NamedAction: only a cancel is ever built here
Control { cancel: tasks::Cancel },
```

Each derives `Clone, PartialEq, Eq, Debug`, with a line of doc. The field is `operation` on every `Event` variant.

**The rules are already one path** (`check_*`, then `Plan`, then `apply_*`). What goes is the input types and the argument lists.
**What differs between entrances is behaviour, and stays**, so no entrance becomes a compound of one:

- refusal order: `Reference` before `NotReady` in a compound, `NotReady` first in `Event::Cancel` and `Event::Release`;
- a compound needs a matching project, and refuses `Party::Task` outright; a control has no project check, and allows a strict
  ancestor task (`control::apply`, whose `permitted` stays ahead of `check_cancel`);
- single entrances remember capacity (`moving::refuse`, `capacity`); a compound doesn't;
- answers: `Done` and `Refused`, `Sent`, `Done` and `CompoundRefused { at }`, `Made` and `ProcedureRefused`;
- `word.from != by` is `Reference` only in compound and procedure, since `Event::Message` has no `by`;
- `authorized`: `Message` false, `AuthorizedMessage` true, compound true, procedure false;
- a procedure's amendments, messages, proposals and cancels stay separate lists, in that order, each under `limits.batch`;
- `funder` and `places` stay real on `Event::Amend`, and `None` and empty in compound and procedure;
- `reason` bounds: `result_bytes` for a cancel, `message_bytes` for an amend and a move.

**Not this item, and they stay:** `ProposalAction` (stored), `people::Operation` (stored, another crate),
`routing/party/dispatch.rs`'s `Control` (an admission question), `RoutedCall::Control` and `ToolKind::Control`, `skein_channel::Control`,
`Request::{Release, Stop, Close}` (effects), `ProcedureInput` (selects inbox inputs), `Ask::ProcedureMessage` (a connector's), and
`Event::{Notice, PersonQuestion, PersonQuestionAnswer, DelegateResult}`, which carry a task's word but aren't the message operation.
