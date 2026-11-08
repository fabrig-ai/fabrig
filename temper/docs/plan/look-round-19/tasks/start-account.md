+++
id = "start-account"
part = "start"
kind = "docs"
needs = []
files = [
  "temper/docs/design/client/journeys.status.md",
]
effort = "medium"
minutes = 45
+++

# What Start opens: the account of `ask-temper-start`

## Goal

The request's task 1 (`temper/docs/plan/look-round-19/inputs/request.md`): the look request `ask-temper-start` (`e9da9f87a8`) draws what "Start" opens on a new project, and its message says it adds nothing beyond `ui-ask-m1`'s move of Configure into Ask temper. Its landing is its account (`temper/docs/design/draft/look-requests.md`, 5): each state its two pages show, matched against what the trunk draws, committed as the account. A state that differs is made to match only where the options below say; otherwise it is a row that says why.

Acceptance:

1. Each page of the request, read from its own commit (`git show e9da9f87a8:temper/docs/design/client/journeys/ask-temper-start/01.html`, `02.html`, and its `README.md`), is compared with the trunk's markup for the same state, element by element in `#app` (roles, names, words, order, classes, states, hrefs), ignoring ids the skein menus number and the journeys' `data-journey-next`:
   - `01.html`, the new project's landing: Home in a project with its repository adopted and "Tell temper how you work" and the chat step left. The trunk's markup is the gallery's `home-two-left` (`temper/tools/gallery/src/samples.rs`), whose deliveries are that state, and `project-landing/01.html` (`temper/docs/design/client/journeys/project-landing/`), recorded on a fresh box. "Start" and the rail's "Ask temper" go to `Ask { project }` with no Ask temper chat (`temper-client-view`'s `pages/home.rs`, `to_begin`, `Act::Go { address: frame::ask_address(..) }`; `frame.rs`, `entries` and `ask_address`).
   - `02.html`, the page they open, before a first message: the gallery's `configure` state (the first run's world at its step 7, `Address::Ask { project: 1 }`, template 5 listed; named `ask-first-run` where `ui-ask-drop` has renamed it on your base).
   The gallery's pages come from `heavy cargo run -p temper-client-gallery -- OUT`, OUT a scratch directory outside the slot's tree, deleted before you push.
2. No overlay rule: the request's commit adds none (`git show e9da9f87a8 --stat`); the account says so in a row.
3. The **Not yet** entry in `temper/docs/design/client/journeys.status.md` naming `look-round-19`'s `start-account` is deleted whole, and no other line.
4. One commit, `Look-request: ask-temper-start`, carrying the account (below) in its message.

## Inputs

- `temper/docs/plan/look-round-19/notes/common.md`.
- `temper/docs/plan/look-round-19/inputs/request.md`, task 1.
- `temper/docs/design/draft/look-requests.md`, 3 and 5.
- `temper/docs/design/client/README.md`, 3.5, *To begin* (the first-step act, "Start") and *Ask temper's page*; 5.3, the rail's "Ask temper"; `temper/docs/design/client/journeys.md`, 9, the paragraph on `ask-temper-start`, which this cut wrote.
- The request: `git show e9da9f87a8`, its whole message.

## Owns and may change

Owns `temper/docs/design/client/journeys.status.md`, its one entry. Nothing else, unless an option below says.

Options, each decided:

- **Both pages match** (the expected case: the message says the request adds nothing): the account's rows say "landed, by `ui-ask-m1`", each naming the code that draws it.
- **A difference the next request changes:** the request `to-begin-two-steps` (`4332a26265`) redraws Home's To begin to two steps and the rail's rows while it shows, and this plan's task `two-steps` builds it. A difference in To begin's steps, notes or the rail's rows is a row "superseded by `to-begin-two-steps`, built by `look-round-19`'s `two-steps`", with no change.
- **A difference in a word, an order or a class of Ask temper's page** (02) that the view draws otherwise: make it match in `temper-client-view`'s `pages/configure.rs` (or `pages/ask_page.rs` where `ui-ask-drop` has renamed it on your base) and its test in `src/tests/configure.rs` (or `tests/ask.rs`), a row of the test that holds the page's words; run `heavy cargo nextest run -p temper-client-view --lib` and `heavy cargo nextest run -p temper-client-gallery --lib`, and reprint the journeys if a print moves (`notes/common.md`, *Prints*). Never `pages/home.rs` or `frame.rs`, which `two-steps` owns: a difference there is a row, deferred to `two-steps` with its words.
- **A difference that is the hand drawing's** (a value written in by hand, a recorded clock, an account's model): a row "differs: the request's page is hand-made" with what differs.

## Steps

1. Read the request's message and its pages from its commit; print the gallery into a scratch directory.
2. Compare each page's `#app` with its trunk markup (acceptance 1), writing down each state and each difference.
3. Delete the status entry (acceptance 3); make a change only where an option says.
4. `python3 -I -B tools/citations/citations.py --base origin/main`. With a Rust change, the runs its option names, `cargo fmt` and `heavy cargo clippy -p temper-client-view --all-targets -- -D warnings`.
5. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| Overlay rules | none: the request adds no rule |
| 01: Home with its repository adopted, "Tell temper how you work" left, "Start" filled at its row's right | landed by `ui-ask-m1` (`pages/home.rs`, `to_begin`), or what differs |
| 01: "Start" opens Ask temper's page with no Ask temper chat | landed (`Act::Go` to `ask_address`) |
| 01: the rail's "Ask temper" after "Chats", the phone's "Ask" tab | landed (`frame.rs`, `entries`), or superseded by `to-begin-two-steps` |
| 02: Ask temper's page before a first message: its question, lede, "Ways to begin", the card "At scale, as temper builds itself", the composer and its model button | landed (the gallery's `configure`), or what differs |
| 02: "Ask temper" current in the rail | landed, or what differs |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4); compiling, prints and focused tests establish no rendered equality |

## Report

The implementer's report: each page's comparison, element by element where it differs; the option taken for each difference; the account as committed; any test run and its count.
