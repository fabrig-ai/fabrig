+++
id = "add-writes-last"
part = "quiet"
kind = "feature"
needs = []
files = ["temper/tools/journeys/src/lib.rs", "temper/tools/journeys/src/main.rs", "temper/tools/journeys/src/tests.rs", "temper/tools/journeys/tests/suite/arguments.rs"]
effort = "high"
minutes = 40
+++

# `add` writes last, and a refusal prints no line of the recording

## Goal

Two things the guard needs that hold before there is any redaction. `add` writes `NAME/recording` first and prints after, so a refusal between them leaves bytes at the output path. And a failed proof prints the stale lines whole, which may hold what the redaction missed: `add`'s refusal names the turn, the line and each record's kind alone.

Acceptance:

- `add` does everything in memory, the pages built, before its first `fs` call. A refused `add` leaves its root as it was: no directory made, a journey of that name byte for byte as it stood, no other file on any path.
- A `Refusal::Proof` and a `Refusal::Save`, stale, print `NAME is stale at turn T, line L: recorded KIND, asked KIND; record it again (client/journeys.md, 8)`, where `KIND` is the first word of the `Stale`'s text. `journey::Stale` keeps its fields as they are.
- The reprint's stale journeys still print whole (`reprint_all`, `stale_message`): their recordings have passed the scan. `journeys.md`, 4.3.
- `print_one`'s reprint keeps what it does: a stale journey's pages are left as they were.

## Inputs

- `temper/docs/design/client/journeys.md`, 3.7 *A refusal*, 4.3 and 5.1.
- `temper/docs/plan/real-journeys/notes/common.md`.

## Owns and may change

Owns the files in the header.

Options, pre-approved:

- Split `print_one` into a pure `printed(name, recording) -> Result<Result<Vec<print::Printed>, Stale>, Refusal>` and a write of those pages, or another split that leaves reprint's order of writes alone.
- A stale `asked` that isn't a line's text (`nothing`, `end`, `deadline`, `no control gives the event`): its first word is what's printed. Say `no control gives the event` whole if that's simple: it holds no recording bytes.

## Steps

In `src/tests.rs`:

```rust
#[test] fn a_refused_add_leaves_its_root_as_it_was()   // empty root stays empty: an unproved recording, and a stale one
#[test] fn a_refused_add_keeps_the_journey_of_its_name() // an added fixture journey; add again, stale; every file byte for byte
```

In `tests/suite/arguments.rs`, as a process, with a recording in a `Scratch` directory:

```rust
#[test]
fn a_stale_recording_prints_no_line_of_its_own() {
    // A fixture recording (Facts, Recorder, as a_reprint_keeps_a_journeys_readme builds one, moved to a helper the
    // suite may build with its dev-dependencies, or the recording's lines written by hand) with `secret: Some(..)`,
    // so that its start asks `/jig/v1/sign-in/link`, whose exchange body is edited to hold a marker of fake bytes: it goes stale. Run `add a-journey FILE`.
    // Exit 1; stderr names the turn and the line and the word `exchange`; neither the marker nor `"/jig` is on stdout
    // or stderr; temper/docs/design/client/journeys/a-journey does not exist.
}
```

Pick a NAME no committed journey has. A refused process writes nothing, so running the real binary is safe; a passing one would write under the checkout, so none does here.

Then fmt, clippy on `temper-client-journeys`, its tests, commit and push.

## Report

As the implementer's text says. Name any `fs` call that still comes before the proof.
