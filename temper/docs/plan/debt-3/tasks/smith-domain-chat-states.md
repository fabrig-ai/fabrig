+++
id = "smith-domain-chat-states"
part = "smith-lifecycles"
kind = "feature"
needs = ["smith-domain-delivery-save"]
files = [
  "smith/crates/smith-local-domain/src/chat.rs",
  "smith/crates/smith-local-domain/src/chat/machine.rs",
  "smith/crates/smith-local-domain/src/person.rs",
  "smith/crates/smith-local-domain/src/domain.rs",
  "smith/crates/smith-local-domain/src/tests.rs",
]
effort = "high"
minutes = 70
+++

# smith D002, the local Domain: what the chat's states hold

## Goal

Two fields left on `Domain` by `smith-domain-delivery-save` mean something in some chat states
only. `closing_last` means something only while input is closed, and `transcript` only from
`Loaded` until a run starts. Move each into the state that holds it, so no field beside `Domain`'s
machines is meaningful in some states only (`skein/docs/foundation/programming-model.md`, 5.4).
This ends the local `Domain`'s share of
`smith/docs/development/debt/D002-lifecycles-in-flags.md`; the item stays open: leave its file.

Behaviour stays unchanged.

Acceptance:

- `rg -n 'closing_last|domain\.transcript' smith/crates/smith-local-domain/src` prints nothing.
- `Domain` has 22 fields (24 after the task before) and 3 `Option` fields: `effort`, `choice` and
  `answer_wall`. `interactive` stays the one `bool`.
- `smith-local-domain --lib` runs 88 tests (85 after the task before): three new, the rest
  unchanged but for the arms the compiler asks for.
- `smith-local-world --test suite` and `smith-local-service --lib` pass unchanged.
- No public item, wire shape, stored format or design text moves.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`.
- `smith/AGENTS.md`.
- `smith/docs/development/debt/D002-lifecycles-in-flags.md`, whole.
- `skein/docs/foundation/programming-model.md`, 5.4.
- `smith/docs/design/domain/host.md`, 8: "Closing input drains", "Transcripts in files".
- The form: `smith/crates/smith-local-domain/src/chat.rs` (`Metadata`, `Input`, `Load`) and
  `chat/machine.rs`, `state_saved` and `start_progress`.

## Owns and may change

Owned: `chat.rs`, whole, and `chat/machine.rs`. Edits elsewhere, and no others:

- `domain.rs`: the two fields and their `Domain::new` lines; the `State::Idle` arm in `gone`.
- `person.rs`: `finish_answered` reads `domain.chat.drained()`; the `State::Idle` arms.
- `tests.rs`: two new tests and any helper they need. `chat.rs`'s tests module takes the third.

Not yours: `delivery/machine.rs`, `reconcile.rs`, `boundary.rs`, `smith/docs/design/**`, any status
file, the debt item. No design text changes: both sections already say what the code does.

Options, each with its default:

- `chat::State` trips `clippy::large_enum_variant` once `Idle` holds its history: a scoped
  `#[expect]` with a reason, as in `boundary.rs`.
- `Input` needs a move instead of a copy in `selected`: make it `Clone, Copy`, as sketched; it
  holds one `u64`.

These stay as they are, each for its reason, and are not to be converted: `choice` (a pending
command, held in any phase, independent of it), `effort` (the selection, always meaningful),
`answer_wall` (the last answer's clock, read after settlement through `Domain::answer_wall`),
`interactive` (set once by `line_mode`, configuration), and `Starting`'s own fields (they live and
die with that state).

## Steps

1. **`closing_last` into `Input`,** in `chat.rs`. It is `Some` only after an interactive close, so
   a variant holds it. `Input::Closed` keeps meaning closed with no bound (an interrupt, a store
   failure, a headless or quiet close):

   ```rust
   #[derive(Clone, Copy, Debug)]
   enum Input {
       Open,
       Closed,
       /// The person closed interactive input: lines drain through activation `last`.
       Draining { last: u64 },
   }

   impl Chat {
       pub(crate) fn closed(&self) -> bool;              // Closed or Draining
       pub(crate) fn close_input(&mut self);             // Open to Closed; the others stay
       pub(crate) fn drain_until(&mut self, last: u64);  // Open to Draining; the others abort
       pub(crate) fn drained(&self) -> bool;             // Draining with activation >= last
       /// The chat a selection opens, with this invocation's input as it stands.
       pub(crate) fn selected(&self, seed: u64) -> Chat;
   }
   ```

   - `closed` in `chat/machine.rs`: `domain.chat.drain_until(activation.saturating_add(extra))`
     under the same condition, then `close_input` as now.
   - `finish_answered`: `let drained = ..; if domain.chat.closed() && drained` becomes
     `if domain.chat.drained()`. They agree: `closing_last` is `Some` only with input closed.
   - `chat_selected`, `Ok` arm: `domain.chat = domain.chat.selected(domain.chat_seed);` replaces
     `let closed`, `Chat::new` and `close_input`. `last` survives a selection, as it does at base,
     and is compared with the new chat's activation, as at base.

2. **`transcript` into the chat's phase,** in `chat.rs`. `Option` here is the store's own answer: a
   chat may have no saved transcript.

   ```rust
   pub(crate) enum State {
       /// Ready for a person line or exit, holding what the next start hands over.
       Idle { history: Option<Transcript> },
       // Loading(Load), Starting(Starting) and the rest stand.
   }

   pub(crate) struct Starting {
       pub(crate) history: Option<Transcript>,
       // the rest stand
   }

   pub(crate) enum Load {
       Ready,
       Waiting,
       /// Loaded: saved intents settle before the chat is idle.
       Settling { history: Option<Transcript> },
       /// A selection runs; a refusal returns to this history.
       Selecting { history: Option<Transcript> },
   }
   ```

   `load_requested` is true for `Waiting`, `Settling` and `Selecting`, so `is_ready` and `resume`
   read as at base. `request_load` stays `Ready` to `Waiting`; every other source aborts.

   The table: every transition that moves it. The history is never cloned.

   | from | event | to | history |
   |---|---|---|---|
   | `Loading(Waiting)` | `Loaded` | `Loading(Settling)`, then `resume_loading` | from the event |
   | `Loading(Settling)` | `resume_loading`, no intent left | `Idle` | moved |
   | `Idle` | `select_chat` admitted | `Loading(Selecting)` | moved |
   | `Loading(Selecting)` | `ChatSelected`, `Err` | `Idle` | moved |
   | `Loading(Waiting)` | `ChatSelected`, `Err` (the initial chat, `Service::initial_chat`) | `Idle` | `None` |
   | `Loading(Selecting)` or `Loading(Waiting)` | `ChatSelected`, `Ok` | `Loading(Waiting)` | dropped with the old chat |
   | `Idle` | `dispatch_line` starts | `Starting` | moved |
   | `Starting` | a chat's refused grant or unavailable head | `Idle` | moved |
   | `Starting` | start | `Running` | into `RunStart` under `config.resume`, otherwise dropped |

   `Loading(Ready)` and `Loading(Settling)` × `ChatSelected` abort with "selection has one
   terminal". In `start_progress`, `let history = if domain.config.resume { starting.history } else
   { None };`. Every other `State::Idle` arm takes `{ .. }`: about 20, the compiler lists them.
   `select_chat` and `dispatch_line` move the state out with `mem::replace`; `chat.rs` may give
   them small methods, in the form of `request_load`.

3. **Tests.**
   - `tests.rs`, `a_refused_selection_keeps_the_loaded_history_for_the_next_start`: `Loaded` with a
     transcript, `SelectChat`, `ChatSelected` with `Err`: `State::Idle { history: Some(_) }`.
   - `tests.rs`, `a_refused_grant_keeps_the_loaded_history_for_the_next_start`: `Loaded` with a
     transcript, a line, `NoCredential`, `StateSaved` in a chat:
     `State::Idle { history: Some(_) }`.
   - `chat.rs`, `a_selected_chat_keeps_the_invocations_drain`: `drain_until(2)`, then `selected`:
     `closed()`, and `drained()` by the new chat's activation.

   ```
   heavy cargo nextest run -p smith-local-domain --lib
   heavy cargo nextest run -p smith-local-world --test suite
   heavy cargo nextest run -p smith-local-service --lib
   ```

   The world's `messages.rs` holds the close and drain stories, and `simulation.rs` the resumed
   transcripts; the service's `tests.rs` holds the initial chat's selection.

4. `cargo fmt -p smith-local-domain`; `python3 -I -B scripts/old_forms.py` from `smith/`;
   `heavy cargo clippy -p smith-local-domain --all-targets -- -D warnings`. Commit and push.

## Report

- `Domain`'s fields and `Option`s before and after, by count.
- Each `Option` and `bool` left on `Domain`, and that the Steps' reason for it held.
- Any `Input` or history cell you placed otherwise than the table.
