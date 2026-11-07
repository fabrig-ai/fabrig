+++
id = "first-journey"
part = "tui-journeys"
kind = "feature"
needs = ["tui-again", "approval-story"]
files = ["temper/docs/design/client/tui/journeys/sign-in-approved/**", "temper/docs/design/client/tui/journeys/theme.css", "temper/docs/design/client/tui/journeys/tui-journey.js"]
effort = "high"
minutes = 90
+++

## Goal

The TUI's first journey, `sign-in-approved` (`temper/docs/design/client/tui/journeys.md`, section 9): on a fresh fake box, the browser signed in by its printed link, `temper` started at the box's address, the approval's frame, the approval given in the browser, and the first screen. Its `walk` and `README.md` are written by hand; its recording, texts and pages are made by `again`, which is generation, not a walk: this task runs it.

Acceptance:

- `temper/docs/design/client/tui/journeys/sign-in-approved/walk` is exactly:

  ```
  walk 1
  see "then press Approve."
  mark
  approve
  see "Connected to"
  end
  ```

- Its `README.md` says what the journey walks (section 9's row) and what each page shows: `01`, the approval's frame, "A tab opened in your browser", its code and its address, marked by the mark key; `02`, the first screen, "Connected to 127.0.0.1:PORT as" the box's owner, at the recording's end; and that the code, the port and the clocks are the recording's own.
- `cargo run -p temper-client-journeys -- again sign-in-approved` run through heavy, with `SKEIN_TEST_CHROMIUM` as the builders set it, writes the journey's `recording`, `01.txt`, `01.html`, `02.txt`, `02.html` (and the beside files if they changed); a run that stops on a moving screen or page is retried at most three times, each on a fresh fake box; if all fail, widen the `see` step's wait in `tools/journeys/src/tui_again.rs` and say so.
- The committed recording holds no original (its `add` scanned it) and no mark's print; `01.txt` shows the code and `http://127.0.0.1:PORT/terminal?code=` with that code, `02.txt` "Connected to".
- `heavy cargo nextest run -p temper-client-journeys` passes, the journeys' test reprinting `sign-in-approved` byte for byte beside the browser's; the reprint within its 1 s (name the time).

## Inputs

- `temper/docs/design/client/tui/journeys.md`, sections 6, 8 and 9.
- `temper/docs/design/client/README.md`, 7.1, *The TUI's flow* (the frame's lines).
- `temper/docs/plan/tui-recorded/notes/common.md` (recording again is generation; the flake rule).
- The browser's journeys' READMEs (`temper/docs/design/client/journeys/*/README.md`), the model of a journey's README.

## Owns and may change

- Owned: the journey's directory and the generated files beside it. **A departure from the implementer's text, on purpose:** these are under `docs/design/`, and are this task's: the `walk` and `README.md` written by hand, the rest by the program.
- May change, if `again`'s first run shows it: a fault in `tools/journeys/src/tui_again.rs` or the TUI's recorder that stops the walk (a key's bytes, a save not told, a bound), fixed in its own commit with a test where one can be written natively, and named in the report. A fault in the TUI's view or host is a question.
- Options:
  - If the frame's lines put "then press Approve." elsewhere than the walk's `see` finds it, the `see` text changes to a line the approval's frame holds, and the README says so.

## Steps

1. Write the `walk` and the `README.md`.
2. Run `again sign-in-approved` (retrying as above), then the journeys' tests.
3. Read `01.txt` and `02.txt` against 7.1's lines; read the recording's line kinds (never its bytes into a log).
4. `cargo fmt` and clippy if any code changed; commit the journey and push.

## Report

The implementer's report, with: each `again` run, its outcome and what it named when it failed; the journey's line count and pages; what the frames show against the design's lines; any fault fixed on the way.
