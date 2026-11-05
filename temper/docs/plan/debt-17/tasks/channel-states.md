+++
id = "channel-states"
part = "channel-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-channel/src/machine.rs", "skein/crates/skein-channel/src/tests.rs", "skein/docs/design/channel.status.md", "skein/docs/development/debt/D019-channel-machine-flags.md"]
effort = "high"
minutes = 120
+++

# skein: the channel machine's lifecycle is states that hold their data

## Goal

Pay `skein/docs/development/debt/D019-channel-machine-flags.md`. `Machine` keeps eleven bools beside its `Phase` and sets
`Phase::Closed` in eleven places. It becomes a few enums, each one lifecycle (`skein/docs/foundation/programming-model.md`,
5.4: one enum per lifecycle, never one for their product), and going closed is written once.

This is a recast: **every event and every bound stays.** Nothing above `machine.rs` changes.

Acceptance:

- `grep -c ': bool' skein/crates/skein-channel/src/machine.rs` reads 0, which takes in `Refusing`'s `framing`, and so does the
  item's `': bool,'` measure. `rg -n ': bool\b' skein/crates/skein-channel/src` finds no new field elsewhere.
- `rg -n 'struct_excessive_bools' skein/crates/skein-channel/src` finds nothing.
- `rg -n 'terminal_why|closed_sent|finish_sent|cancel_sent|cancel_requested|output_granted|peer_terms|write_failed' skein/crates/skein-channel/src/machine.rs`
  finds nothing (two test names in `tests.rs` say `peer_terms` about something else, and stay), and `rg -c 'self\.phase = Phase::Clos' skein/crates/skein-channel/src/machine.rs` reads at most 3: `Closing`
  in the one close path, `Closed` when the terminal goes up, and cell 1 below.
- `git diff` of `skein/tests/channel` is empty, every test already in `skein-channel/src/tests.rs` is unchanged, and the worlds
  and readers below pass.
- `MAX_UP` and `MAX_DOWN` stay, and so do the public `Machine` items (`new`, `with_mode`, `down`, `up`, `poll`, `waiting`, `version`,
  `room`).
- Cap: production lines of `machine.rs` rise by at most 60. This is the one item of the round that may grow.
- The item is deleted, and `channel.status.md`'s `Departs: 5, 8.3` entry leaves; the file then says it has no gap, as `DESIGN.md`,
  2 asks.

## Inputs

- `temper/docs/plan/debt-17/notes/common.md`
- `skein/docs/development/debt/D019-channel-machine-flags.md`
- `skein/docs/design/channel.md`, 5, 6, 7, 8.1, 8.2, 8.3, 9; `skein/docs/foundation/programming-model.md`, 5.2 and 5.4

## Owns and may change

- **Owns:** the files in this spec's header. `boundary.rs`, `opening.rs`, `write.rs` and `read.rs` should need nothing; a
  private helper's signature may change for what the recast forces.
- **Inside the sketch the choices are yours,** and where a cell can't be held by the sketch's shape, change the shape and say
  so in the report. If an event would have to change, stop and ask.

## Steps

1. **First commit: five pins, before any recast.** What the machine does today in cells a recast is likely to move, as step
   tests in `src/tests.rs` that assert today's events. They pass on the base, and each stays through the recast:

   1. A credential past `limits.credential` in `open`: the phase goes closed, with no event, and a later `close` is a no-op:
      the owner hears nothing.
   2. A framing error in `StreamMode::Two` after `Finish` went down: no terminal until the owner closes.
   3. `StreamMode::One`, `Failed` then `Settled { Granted }` on the output right: no terminal.
   4. `Up::End` with a demand out: the `Demand { Nothing }` still goes down.
   5. `StreamMode::Two`, truncation: the queue keeps draining while closing, and `close` is ignored meanwhile; an output
      failure while `Refusing` reports `OutputFailed` then `Closed`.

   **The first three look like faults against `channel.md`, 8.3. They stay as they are here:** the request holds behaviour
   still. Name each plainly, and list them under `## Left` in your report for the plan's owner. If one can't be reached through
   `down`, `up` and `poll`, say so rather than pin it.

2. The shape, `machine.rs`'s private types:

   ```rust
   enum Phase {
       Idle,
       WaitOpen,
       WaitOwner { lowest: u16, highest: u16 },
       WaitAccept,
       WaitTerms { version: u16, own: OwnTerms, peer: Option<List<Term>> },
       Ready { version: u16, peer: List<Term> },
       Finished { version: u16, finish: Finish },
       Refusing { why: Closed, finish: Finish }, // for `reason` and `framing`
       Closing { why: Closed },                  // active, then closing, then closed (channel.md, 8.3)
       Closed,
   }
   enum OwnTerms { Due, Sent }
   enum Finish { Due, Sent }

   enum Output {                                 // for four fields and `write_failed`
       Free,
       Asked { right: Token },
       Granted { right: Token },
       CancelDue { right: Token },
       Cancelling { right: Token },
       ReleaseDue { right: Token },              // `granted` and `requested`, however they came
       Failed { owed: Option<Token> },
   }

   enum ReadState {
       Header { first: Option<u8> },
       Body { header: Header, body: Box<[u8]>, filled: u32 },
       Skip { kind: u16, remaining: u32 },
       Skipped { kind: u16 },                    // for `pending_unsupported`
       Ended,
       Idle,
   }
   enum Demand { None, Out }                     // for `read_outstanding`
   ```

   - `read_credit` is what `ReadState` says: `Header`, `Body`, `Skip` or `Skipped` in `Ready` or `Finished`. Read through a small
     method, not a field.
   - `ping_queued` and `unsupported_queued` are a scan of the bounded queue by `QueueTag`.
   - A grant can beat a cancel, leaving `granted`, `requested` and `sent` all true (`tests.rs` holds that race). `ReleaseDue`
     takes it.
   - `write_failed` goes with any active phase in `StreamMode::Two`, never with `Refusing`, and it can go with a held right.
     `Failed { owed }` holds that.

3. The one close path. `maybe_terminal` becomes the only writer of `Closed` but for cell 1:

   ```rust
   enum OutputFate { Drop, Keep }
   /// Active to closing (channel.md, 8.3): sets `Closing`, ends the read unless it has `Ended`, and on `Drop`
   /// clears the queue and gives up the right.
   fn closing(&mut self, why: Closed, output: OutputFate);
   ```

   Sites at this plan's base, which keep what differs between them: `refuse` after a write failure, `close` (which adds its read
   withdrawal, `cancel_output` and `maybe_terminal` around it), both arms of `output_failure`, end of stream with a failed write
   (the read stays `Ended`), truncation, a failed read, a peer's `Refuse`, and `maybe_finish`. Update the state table at the
   top of `machine.rs`.

4. `heavy cargo nextest run -p skein-channel -p skein-channel-world`. Readers, whose behaviour rests on this: `heavy cargo
   nextest run -p jig-protocol-worker-link -p jig-protocol-engine-workers -p smith-protocol-channel -p smith-host-protocol`, then
   `-p jig-channel-world`. No public signature changes, so no other dependent is run.

5. Delete the item and the status entry. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-channel --all-targets -- -D warnings`; the citations check. Commit and push.

## Report

The common note's items, where the sketch's shape moved and why, the five pins and what each found, and each under `## Left`.
Each pin's bite: break the rule by hand on the recast, and say what failed.
