+++
id = "skein-echo-states"
part = "skein-echo"
kind = "migration"
needs = []
files = [
  "skein/examples/echo/protocol/src/layer.rs",
  "skein/examples/echo/protocol/src/conn.rs",
  "skein/docs/design/examples.status.md",
  "skein/docs/development/debt/D024-echo-template-flags.md",
  "skein/docs/development/debt/README.md",
]
effort = "high"
minutes = 75
+++

# skein D024: the echo's stop, its peer's end and its closing are states

## Goal

The echo example's protocol layer is the template every service copies; make it read as
the model's: the layer's stop and the connection's end and closing become states, as
`skein/docs/design/examples.md`, 3.4 now gives them (this cut wrote the tables), following
`skein/docs/foundation/programming-model.md`, 5.4. Behaviour is unchanged: every echo test
passes unchanged. This pays `skein/docs/development/debt/D024-echo-template-flags.md`.

Acceptance:

- `Protocol` holds no `stopping` or `aborting` field: its stop is the two machines of
  `examples.md` 3.4's "The layer's stop" (`Shutdown` and `Stop`).
- `conn.rs` holds no `ended`, `closed` or `call` field and no `Closing` struct: the states are
  3.4's (`AdmittingEnded`, `AnsweringEnded`, `Closing(Settle)`, `Shut(Settle)`).
- No test file of the echo changes (`git diff --stat origin/main -- skein/examples/echo
  skein/tests/echo` lists `layer.rs` and `conn.rs` only), and the tests in Steps pass, each
  count above zero.
- `examples.status.md` keeps its entry for 6 and loses 3.4's; D024's file and its README row
  are gone; `tools/citations/citations.py --all` reads 0; clippy is clean.

## Inputs

- `temper/docs/plan/debt-1/notes/common.md`.
- `skein/AGENTS.md`.
- `skein/docs/development/debt/D024-echo-template-flags.md`, whole.
- `skein/docs/design/examples.md`, 3.3 and 3.4 (3.4 written by this cut: the connection's
  states and table, and "The layer's stop"). The tables bind the states' names and cells.
- `skein/docs/foundation/programming-model.md`, 5.4 (one enum per lifecycle; the table is
  total; matches exhaustive, one enum then the other, never a tuple; one small handler per
  cell; the `mem::replace` idiom; `follow`), and 5.2 (a connection retires once both bindings
  end).
- `skein/docs/design/shell.md`, 13 (the owner's drain and abort).

## Owns and may change

- `skein/examples/echo/protocol/src/layer.rs` and `conn.rs`, including their module and item
  docs (the owner-transition table in `conn.rs`'s module doc follows the new states).
- `skein/docs/design/examples.status.md`: remove the words for 3.4 from its **Not yet** entry
  (from "3.4:" to just before "6:"), keep 6's, and date it today.
- `skein/docs/development/debt/D024-echo-template-flags.md` (delete) and its row in
  `skein/docs/development/debt/README.md`. `D025-request-table-in-lib.md` names "D017 to D024"
  by ID, not by path: leave it.

Nothing else changes: no public item of `skein-echo-protocol` (`Protocol`'s methods,
`resume`, `up`, `fire`, `down`) changes its signature, and no test file is edited.

Pre-approved:

- If a test fails, the change moved behaviour: fix the code, never the test. If a test cannot
  pass without editing it (it reads a private field, say), stop and ask.
- Every panic message stays word for word where a test pins it
  (`rg -n 'should_panic' skein/examples/echo`: `"one reply per call"` in
  `protocol/src/tests/conn.rs`); keep the others too unless a cell's reason changes.
- The `Shut` cells for `Bytes`, `Room`, `End` and `Failed` are ignored, as `Closing`'s are, so
  that behaviour is unchanged; do not make them `unreachable!`.
- A helper taking `abort: bool` as an argument (today's `schedule_stop`, `Conn::stop`) may
  stay: an argument that picks the owner's transition is not a flag beside a state. Taking
  the target `Stop` instead is equally fine.

## Steps

1. Sketch of the outer layer (private types; the names are 3.4's):

   ```rust
   // layer.rs
   pub struct Protocol {
       addr: Addr,
       listener: Listener,
       conns: Slab<Conn>,
       tables: Tables,
       shutdown: Shutdown,
       stop: Stop,
   }

   /// The service's shutdown from io or a lower-tier world (io.md, section 7),
   /// told to the domain once from the ready list.
   enum Shutdown { Running, Asked, Told }

   /// The connections' stop (examples.md, 3.4): drained once the domain asks
   /// `Stop`, aborted by a second shutdown (shell.md, section 13).
   enum Stop { Serving, Draining, Aborting }
   ```

   `Protocol::shutdown` matches `self.shutdown`, then `self.stop` inside each arm (3.4's
   table: Running with Serving goes Asked; Running with Draining goes Told and Aborting, each
   connection aborting; Asked or Told with Serving or Draining goes Aborting, each aborting;
   with Aborting it is ignored; Running with Aborting is `unreachable!`). The domain's `Stop`
   (`fn stop`) matches `self.stop`: Serving or Draining goes Draining, each connection
   draining; Aborting stays, each connection aborting; the listener's own transition is as
   today. `is_ready`, `is_empty` and `resume` read `Shutdown` as today.

   ```rust
   // conn.rs
   enum State {
       Greeting,
       Admitting,
       AdmittingEnded,
       Reading { session: Token },
       Answering { session: Token },
       AnsweringEnded { session: Token },
       Draining { session: Token },
       Closing(Settle),
       Shut(Settle),
       Closed,
   }

   /// What the domain still holds of a closing connection (examples.md, 3.4).
   enum Settle {
       /// Its call is out, whose reply is awaited.
       Call,
       /// The session the domain is still to be told is gone, from the ready list.
       Owed(Token),
       /// Nothing.
       Clear,
   }
   ```

   `owes()` is `Closing(Owed(_))` or `Shut(Owed(_))`; `is_done()` is `Shut(Clear)`; io's
   `Closed` takes `Closing(s)` to `Shut(s)`; an owner abort sends `Abort` from `Closing` and
   not from `Shut`; a reply on `Closing(Call)` or `Shut(Call)` gives `Owed(session)` for
   `Admitted` and `Clear` for `Busy` and `Echo`, the same outer state kept; `resumed` tells
   `Gone` from `Owed` and leaves `Clear`. `follow` gives the two new open states no demand
   and no deadline, as their unended twins.
2. Narrow tests, one at a time, each count above zero:
   - `heavy cargo nextest run -p skein-echo-protocol --lib`
   - `heavy cargo nextest run -p skein-echo-service --lib`
   - `heavy cargo nextest run -p skein-echo-world --test suite`
   - `heavy cargo nextest run --profile fuzzy -p skein-echo-world --test suite`
   - `heavy cargo nextest run -p skein-echo-shell`
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`;
   `heavy cargo clippy -p skein-echo-protocol -p skein-echo-service -p skein-echo-world -p skein-echo-shell --all-targets -- -D warnings`.
4. Status and debt text as in "Owns"; `python3 -I -B tools/citations/citations.py --base
   origin/main` and `--all` from the repository root.
5. Two commits ("the echo's stop, its peer's end and its closing are states
   (programming-model.md, 5.4)"; "pay skein D024: examples.status.md 3.4 and the item"),
   pushed.

## Report

As the implementer's text says, and also: the final `State`, `Settle`, `Shutdown` and `Stop`
variants; any cell whose handler you merged or split beyond the sketch; the test counts of
each command.
