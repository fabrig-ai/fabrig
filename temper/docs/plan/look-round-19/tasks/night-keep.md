+++
id = "night-keep"
part = "night"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/terminal.rs",
  "temper/crates/temper-client-domain/src/tests/pages/terminal.rs",
  "temper/crates/temper-client-shell-web/src/records.rs",
  "temper/crates/temper-client-shell-web/src/host.rs",
  "temper/crates/temper-client-shell-web/src/lib.rs",
  "temper/crates/temper-client-shell-web/src/host_tests.rs",
]
effort = "high"
minutes = 90
+++

# The night look, kept per browser, and `html[data-night]`

## Goal

The request's task 3 (`temper/docs/plan/look-round-19/inputs/request.md`), its state: the look request `night-mode` (`3c5333485a`) needs a switch whose choice holds across reloads. This task builds what the switch turns, not the switch: the kept look gains `night`, `terminal::Act::Night` turns it and keeps the record, and the web shell sets `data-night` on the document's `html` while it is on, from the record before the first page is drawn and in the step a press turns it. `night-switch` draws the button over it; `night-tokens` draws the night under `html[data-night]`.

Acceptance:

1. `temper-client-domain`, `pages/terminal.rs`:
   - `Look { terminal, theme, font, night: bool }`; `Look::default()` has `night: false` (a first visit is day, whatever the system's scheme).
   - `Act::Night`, documented "The night switch: the other of day and night, on any page." `Look::act` turns `night` and pushes `Request::KeepLook { look }` as `Theme` does, and asks no full screen. Every `match` over `Act` takes the arm (compiler-forced arms are allowed anywhere in the tree).
   - The act resolves on any page: `Pages::terminal_act` (and `prepare`, which reaches it) must not drop `Night` where the terminal doesn't draw (`terminal::draws` false: Home, Settings, Needs you). Confirm by reading, and by a test row on Home.
2. `temper-client-shell-web`:
   - `records.rs`: `encode_look(look) -> [u8; 4]`, the fourth byte `0` or `1`, `1` night; `decode_look` reads four bytes, and three, the form before `night`, with `night` off; any other form is `None`. `host.rs`'s `keep_look: Option<[u8; 4]>` and its duty follow.
   - The host says the look's night to the shell: `pub fn night(&self) -> bool` on `Host`, from the domain's `pages().look().night`.
   - `lib.rs` (wasm alone): after `host.start(..)` and before the first page is drawn, and after every step whose look's night differs from what the document's `html` holds, `document.document_element()`'s attribute `data-night` is set (empty value) while night is on and removed otherwise; a step that leaves it alone touches nothing. A refused write of the record changes nothing on the page (the attribute follows the domain's look, not the write).
3. Tests, rows of existing tables where one holds the case, each rule broken once by hand:
   - Domain (`src/tests/pages/terminal.rs`, the test that holds `Theme`'s keeping, or the nearest table): `Night` on Home turns `night` and keeps `{ .., night: true }`; a second press turns it back; `Enter` and `Leave` keep `night` as it is.
   - Shell (`src/host_tests.rs`, the test that holds the look's record at line ~663, and `records.rs`'s own tests if it has them): the record's four bytes both ways; a three-byte record reads with night off; a five-byte record is none; a host started with a night record says `night()` true before any step; a press's `KeepLook` duty carries the fourth byte.
4. `temper/docs/design/client/README.status.md`'s **Not yet** entry naming `night-keep` and `night-switch` loses `night-keep`'s half (the look, its record, the act, `data-night`), keeping the switch's; no other design line.
5. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-domain`, `-p temper-client-shell-web`, `-p temper-client-view`, `-p temper-client-shell-terminal`, `-p temper-client-shell-tui`; `cargo check -p temper-client-shell-web --target wasm32-unknown-unknown` (the attribute's code is wasm's alone; with no wasm target on the builder, say so); clippy on every package changed.

One commit or more, each `Look-request: night-mode`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-19/notes/common.md`.
- `temper/docs/plan/look-round-19/inputs/request.md`, task 3: "Where the switch's choice is kept, and whether a first visit follows the system's scheme, are this task's to decide, as the smallest thing that holds across reloads." This cut decided: the kept store's `look` record, which already holds across reloads, and a first visit is day.
- `temper/docs/design/client/README.md`, 3.5, *A chat's terminal*, *Its look, kept per browser* (this cut's); 5.3, the night switch.
- `temper/docs/design/client/browser.md`, 2.5, *Day and night*, and section 5's *Night following the system* and *Night before the first paint* (both Later).
- The request: `git show 3c5333485a`, its message and `night-mode/night.js` there (its `localStorage` key and `#night` hash are the hand drawing's, not the client's).

## Owns and may change

Owns the header's files. Reader and neighbour files, compiler-forced arms and fields only: every `match` over `terminal::Act` and every `Look { .. }` literal in the tree (`temper-client-view`'s terminal commands and tests, `temper-client-shell-terminal`'s and the view's fixtures that list `Request::KeepLook`, the gallery's samples); `temper/docs/design/client/README.status.md`, acceptance 4.

Options, each decided:

- **A terminal command for night** (`/night`): none; the switch is the browser's rail's (`night-switch`), and the chat terminal keeps its own theme.
- **`terminal_act` returns before `Look::act` off a chat page:** move `Night` (with nothing else) ahead of that return, and say so.
- **`lib.rs` has no step hook after events** where the attribute can be set: set it where `duties.keep_look` is handled and once after `host.start`, reading `host.night()` each time; the same effect.
- **A test that pins the record's three bytes** (`host_tests.rs`): it takes four, keeping what it checks.

## Steps

1. Read the design's *Its look, kept per browser* and the request's message.
2. The domain's `Look` and `Act::Night` (acceptance 1), its tests; `heavy cargo nextest run -p temper-client-domain`.
3. The shell's record, `Host::night`, the attribute (acceptance 2), its tests; `heavy cargo nextest run -p temper-client-shell-web`.
4. The status entry (acceptance 4); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; clippy; the runs of acceptance 5; `python3 -I -B tools/citations/citations.py --base origin/main`.
5. Commit with `Look-request: night-mode` and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| `night.js`: the switch's state kept in the browser across reloads | landed as the kept store's `look` record, four bytes, `night` its fourth (`records.rs`) |
| `night.js`: `html[data-night]` toggled | landed: the shell sets and removes `data-night` on `html` from the look (`lib.rs`) |
| A first visit | day, whatever the system's scheme (`Look::default`); following the system is Later (`browser.md`, 5) |
| The switch's button and its `aria-pressed` | deferred to `look-round-19`'s `night-switch` |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4); compiling and focused tests establish no rendered equality |

## Report

The implementer's report: where `Night` resolves, where the shell sets the attribute, the tests and each rule broken by hand, the wasm check's result, the runs' counts, and the account as committed.
