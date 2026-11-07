# tui-recorded: the plan

2026-11-07. From `inputs/request.md`: the rest of the TUI's M2, its way through the browser's approval, then M3, its journeys recorded from the start (`temper/docs/design/draft/tui.execution.md`, M2 and M3; `tui.md`, 4.2 and section 5). It designs and builds together and lands whole.

## The design

- **The TUI's flow** is `temper/docs/design/client/README.md`, 6.13 (the seam gains `Report::Opened`) and 7.1, *The TUI's flow*, which `tui-approval`'s cut began and this cut finishes: the ask once per opening of the sign-in screen, the tab's address from the TUI's own parts (`approval_address`), the opener chosen by the binary and run with one argument, the opener's report, the lines, a pasted link replacing the take, and asking again.
- **The TUI's journeys** are a document of the TUI's own, `temper/docs/design/client/tui/journeys.md`, with its status. The browser's `journeys.md` gains only the shared lines: the pointer, and 3.6's three TUI kinds of original.
- **Status:** `draft/tui.status.md`'s lag (5) is closed and section 8's format question records the default taken; `tui.execution.md`'s M2 and M3 states are set as they stand at landing; `client/README.status.md` and the two journeys' statuses hold the **Not yet** entries this plan's landing clears.

## Parts and tasks

| Part | Task | Needs | Crates | Minutes |
|---|---|---|---|---|
| `tui-approval-flow` | `tui-surface` | none | `temper-client-tui` (state, `approval_address`, `may_ask_again`, the view's clock) | 60 |
| | `approval-lines` | `tui-surface` | `temper-client-tui` (words, view, tick, goldens) | 150 |
| | `child-alone` | none | `skein-io` (`Request::SpawnAlone`, `Way::Null`), `skein-shell`'s spawn, `skein-sim` | 120 |
| | `approval-host` | `tui-surface`, `child-alone` | `temper-client-shell-tui` (host, `opener`, run), `temper`'s `tui.rs` | 180 |
| | `approval-story` | `approval-host`, `approval-lines` | `temper/tests/client` | 120 |
| `tui-journeys` | `originals` | none | `temper-client-shell-web`'s `journey` (three kinds, `Originals::new`, `redact_field`, `scan::{original, shape}`) | 75 |
| | `tui-lines` | none | `temper-client-shell-tui`'s `journey` (feature `record`; the file's lines) | 120 |
| | `tui-replay` | `tui-lines`, `approval-host` | `temper-client-shell-tui`'s `journey` (`Facts`, `Recorder`, `Replay`, `prove`, `event`) | 180 |
| | `tui-redact` | `originals`, `tui-replay` | `journey`'s TUI originals, fields, `redact`, `codes`, `scan`; `temper-client-shell-web` as `record`'s dependency | 150 |
| | `tui-record` | `tui-replay` | the run's recorder, its keys and saves; `temper`'s feature `record` | 150 |
| | `tui-walk` | `tui-replay` | `journey`'s walk file, `follows`, `shows`, `browsed` | 75 |
| | `tui-print` | `tui-redact`, `approval-lines` | `tools/journeys` (the TUI's `add`, print, reprint, tests); `temper-client-tui`'s `theme::palette`; the generated files and empty overlay | 210 |
| | `tui-again` | `tui-print`, `tui-record`, `tui-walk` | `tools/journeys`' `again` for a TUI journey | 240 |
| | `first-journey` | `tui-again`, `approval-story` | `sign-in-approved`: its walk and README by hand, the rest by `again` | 90 |

## Critical path

`child-alone` (120) → `approval-host` (180) → `tui-replay` (180) → `tui-redact` (150) → `tui-print` (210) → `tui-again` (240) → `first-journey` (90): about 1,170 implementer-minutes. Off it: `tui-surface` (60), before `approval-host`; `originals` (75) and `tui-lines` (120) from the start; `approval-lines` (150) beside `approval-host`; `approval-story` (120) after both; `tui-record` (150) and `tui-walk` (75) beside `tui-redact` and `tui-print`. Thirteen tasks, about 1,920 implementer-minutes in all.

## Decisions

- **Asking again is Enter in the empty Link field,** not `a`: the field has the focus on the sign-in screen and takes every character, as it takes `q` (`tui.status.md`, 4.1's departure); 7.1 says so. While an approval waits, Enter in the empty field is the paste's refusal, as before.
- **The opener is the binary's choice** (SSH, no display, `PATH`), and the run reports its end as `Report::Opened`; the view draws the tab as opened until it hears otherwise. The opener's descriptors are never the terminal's.
- **One journey format, sharing what fits:** the browser's syntax and shared records, the TUI's records beside them; the TUI's codec is its own, the redaction and scan the browser's code, reached through `Originals::new`, `redact_field` and `scan::{original, shape}`. The browser's recordings, prints and refusals don't change.
- **The journey module is `temper-client-shell-tui`'s, under `record`;** `temper`'s own `record` feature, which nothing else enables, is what makes a TUI record, so a workspace build's feature unification never makes the shipped TUI record.
- **A TUI journey has a `walk`,** the steps `again` follows, so that the first journey is made by `again` with no earlier recording, and `again` needs no stillness test: each step waits for a frame's text, read by replaying a save natively.
- **The scan cuts the approval's code** read from the TUI's own `browse` addresses before seeking shapes, since `code=WDJB-MJHT` takes the *code* shape; the browser's scan is untouched.
- **The opener is spawned alone** (`child-alone`, in skein first, `AGENTS.md`, 1): io's close of a child kills its group, which would end the browser an opener started, so skein's io gains `Request::SpawnAlone`, whose close kills the child alone, and `Way::Null` for the opener's 0, 1 and 2 (`skein/docs/design/io.md`, 6). Every exhaustive match on `io::Request` and `kernel::Way` gains an arm, in every tree.
- No stored format or protocol of the engine changes. The seam gains a report (`Opened`); before v0.1.0, with no compatibility.

## Overlaps

- **The browser's journeys** (`temper-client-shell-web`'s `journey`, `tools/journeys`, `docs/design/client/journeys/`): `originals` adds to the shared redaction and scan, additions only; `tui-print` and `tui-again` add the TUI's beside the browser's in `tools/journeys`. The browser's recordings and prints are never touched.
- **skein's io** (`child-alone`): additions only, `SpawnAlone` and `Way::Null`; `Spawn`'s group kill is unchanged, and the other trees' matches gain arms and nothing else.
- **`temper-client-domain`:** no change. The root's `asked`, `Domain::asked()` and `print_approve` already give the TUI what it needs.
- **`tui-lines` and `approval-host`** both add one line to `temper-client-shell-tui/src/lib.rs` (`mod journey`, `mod opener`) at the same time; nothing else of theirs meets.
- **`temper-client-shell-tui`'s `journey` module** is written by `tui-lines`, then `tui-replay`, then `tui-redact`, `tui-record` and `tui-walk` at once: the three meet only in `src/journey.rs`'s `mod` and `pub use` lines (and `tui-redact`'s `Cargo.toml` feature line), each told to keep its hunk there.
- **`.config/gate/read-by-tests.txt`** gains `temper/docs/design/client/tui/journeys/**` (`tui-print`): the journeys' test reads the TUI journeys' READMEs and overlay.
- **Drafts beside this one:** none of their files is this plan's (`ui-stop-coding-row`'s one-process and jig routing, `ui-live-cheap-2`'s `forge_land`, `wake-followups`' engine store).
