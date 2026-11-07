+++
id = "tui-print"
part = "tui-journeys"
kind = "feature"
needs = ["tui-redact", "approval-lines"]
files = ["temper/tools/journeys/Cargo.toml", "temper/tools/journeys/src/lib.rs", "temper/tools/journeys/src/main.rs", "temper/tools/journeys/src/tui.rs", "temper/tools/journeys/src/tui/**", "temper/tools/journeys/tui-journey.js", "temper/tools/journeys/src/tests.rs", "temper/tools/journeys/tests/suite/refusals.rs", "temper/tools/journeys/tests/suite/arguments.rs", "temper/crates/temper-client-tui/src/theme.rs", "temper/crates/temper-client-tui/src/tests/theme.rs", "temper/docs/design/client/tui/journeys/overlay.css", "temper/docs/design/client/tui/journeys/theme.css", "temper/docs/design/client/tui/journeys/tui-journey.js", ".config/gate/read-by-tests.txt"]
effort = "high"
minutes = 210
+++

## Goal

`temper-client-journeys` takes, proves, scans and prints the TUI's journeys beside the browser's (`temper/docs/design/client/tui/journeys.md`, section 6 and 7): `add` reads a recording's client by its first fact after `clocks`, and a TUI's is redacted, proved, scanned and printed under `docs/design/client/tui/journeys/NAME/` as its marks' texts (`NN.txt`) and pages (`NN.html`), with `theme.css`, `tui-journey.js` and an empty `overlay.css` beside them; the reprint prints both directories; the journeys' test holds the TUI's as it holds the browser's.

Acceptance:

- `add [--redact TEXT]... NAME FILE`: a TUI recording (`screen` after `clocks`) goes to the TUI's directory, a browser's as today; a NAME held by the other directory is refused; the TUI's steps are `journey::redact`, `journey::prove`, `journey::scan` and the scan of each printed text and page (`scan_bytes` with the recording's `codes`), all in memory, nothing written until they pass; its counts print as the browser's, with `approval`, `credential` and `bearer` beside `link` and `given`; a refusal names the line, its record's kind and the place, never the bytes, and writes nothing.
- The reprint (`cargo run -p temper-client-journeys`) prints every TUI journey from its recording beside the browser's; a stale one is named with its turn and line, its print left as it was, and the program exits 1 once the others are printed.
- The print, `tui/journeys.md`, 6.2 to 6.4 exactly: `NN.txt` as `Frame::text`; `NN.html` with its head, its `pre` of `tui` and the scheme's class, a row per line, runs of one paint as plain text (the base's) or a `span` of palette classes and attributes, a colour off the palette as its `style`, the cursor's `span`, the escapes, the key's link (`data-journey-key`, `data-journey-next`) on a mark taken before a key, and its neighbours; `theme.css` from `temper-client-tui`'s `theme::palette` for both schemes; `tui-journey.js` the program's (`tools/journeys/tui-journey.js`), following a page's key and `n` and `p`; `overlay.css` made empty when missing and never written otherwise. The generated `theme.css` and `tui-journey.js`, and the empty `overlay.css`, are committed under `temper/docs/design/client/tui/journeys/`.
- `theme::palette(scheme) -> [(&'static str, Rgb); 7]` in `temper-client-tui` (contracts, section 9), from which `theme` takes its colours; the TUI's goldens unchanged.
- Tests:
  - in the print's unit tests: a row's runs, a palette class and an off-palette `style`, each attribute, the cursor, the escapes, a key's link for `enter`, a character and `control c`, and a mark key's page with neighbours alone;
  - in `src/tests.rs`, beside the browser's: every committed TUI journey reprinted into a scratch directory equals its directory and the files beside it, byte for byte, failing with the command that reprints; the TUI's `overlay.css` holding a byte fails; a TUI journey whose `README.md` is missing or empty fails; `add` of a TUI recording built in the test (the recorder over the host, reports given by the test, fake wire bytes, as `temper-client-shell-tui`'s journey tests build theirs) writes its directory and reprints byte for byte; a name across both directories refused;
  - in `tests/suite/refusals.rs`, as a process: a TUI `add` refused for an original left in a frame and for a proof that differs prints no original's bytes on stdout or stderr and leaves its root as it was.
  - The reprint of both directories together within the browser's 1 s of the default suite (name the time).
- `heavy cargo nextest run -p temper-client-journeys -p temper-client-tui` passes, every committed browser journey reprinting byte for byte; clippy clean for both.

## Inputs

- `temper/docs/design/client/tui/journeys.md`, sections 5 to 7.
- `temper/docs/design/client/journeys.md`, 5.1, 5.2 and section 6 (the browser's program and tests, the model).
- `temper/docs/design/client/README.md`, 6.15 (the palette's names and colours).
- `temper/docs/plan/tui-recorded/notes/contracts.md`, sections 6, 7 and 9 (binding).
- `temper/docs/plan/tui-recorded/notes/common.md`.
- The code: `tools/journeys/src/lib.rs` (`add`, `reprint`, `names`, `print_one`, `Added`, `Refusal`), `src/print.rs`, `src/main.rs` (`refusal` and its messages), `src/tests.rs` and `tests/suite/`; `temper-client-shell-tui`'s `journey` (merged); `temper-client-tui/src/theme.rs`; skein's `Frame` (`row`, `text`, `print`) and `Paint`.

## Owns and may change

- Owned: the files above. The TUI's print code in `src/tui.rs` (and `src/tui/` if it grows), the browser's `print.rs` left as it is.
- **A departure from the implementer's text, on purpose:** the generated files and the empty overlay under `temper/docs/design/client/tui/journeys/` are this task's, made by the program's reprint, as the browser's prints are their tasks'; nothing else of `docs/design/` is.
- `.config/gate/read-by-tests.txt` gains one line, `temper/docs/design/client/tui/journeys/**`, since the journeys' test reads its READMEs and overlay.
- Readers: the browser's journeys and their tests stay as they are; a helper both print paths need moves only if neither's output changes.
- Options:
  - `temper-client-shell-tui` as a dependency with `features = ["record"]` (default).
  - `Refusal` gains variants for the TUI's refusals (default), the browser's unchanged.
  - With no committed TUI journey yet, the reprint test reprints `theme.css`, `tui-journey.js` and the overlay alone; `first-journey` adds the first.

## Steps

1. Read the inputs. `palette`; the TUI's `add` and print; the reprint; `main.rs`'s lines; `tui-journey.js`; the committed generated files (by the reprint, never by hand).
2. The tests above. Break each rule once by hand (a page without its key link; `add` writing before its scan; `overlay.css` rewritten), see a test fail, restore; name them.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-journeys -p temper-client-tui --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-journeys -p temper-client-tui`.
4. Commit and push.

## Report

The implementer's report, with: the tests and the reprint's time; each broken rule's failure; how a page was checked by eye in its text (never a browser).
