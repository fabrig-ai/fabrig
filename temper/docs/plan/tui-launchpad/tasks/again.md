+++
id = "again"
part = "again"
kind = "feature"
needs = ["journey-setup", "tui-launchpad", "tui-foot", "tui-chat", "host-launchpad", "joint"]
files = ["temper/docs/design/client/tui/journeys/sign-in-approved/**", "temper/docs/design/client/tui/journeys/launchpad-chat/**", "temper/docs/design/client/tui/journeys.status.md", "temper/tools/journeys/src/tui_again.rs", "temper/tools/journeys/src/tui_again/**"]
effort = "high"
minutes = 150
+++

# tui-launchpad: the TUI's journeys recorded again, and the request's account

## Goal

The plan's last task. Record `sign-in-approved` again, which now ends on the launchpad of
a box with no project, and record the new TUI journey `launchpad-chat` on a fake box set
up to the project Hello: the launchpad, the commands, a question sent to Ask temper and
the chat's answer. Its last commit carries the request's account
(`temper/docs/design/draft/look-requests.md`, 5), checked row by row against the trunk.

Acceptance:

- `sign-in-approved/walk`'s `see "Connected to"` becomes `see "Setting temper up makes
  your first project"` (the launchpad with no project, `temper/docs/design/client/README.md`,
  6.14, *No project yet*), nothing else of the walk changing; `heavy cargo run -p
  temper-client-journeys -- again sign-in-approved` records it, and its `README.md`'s
  `02` line says the launchpad of a box with no project, in place of "the first screen,
  Connected to 127.0.0.1:PORT as Owner".
- `launchpad-chat/`, new, as `temper/docs/design/client/tui/journeys.md`, 9 has it:
  - `setup`:

    ```
    setup 1
    after "project-landing"
    see "work/hello is in Hello"
    reload "/projects/1"
    ```

  - `walk`:

    ```
    walk 1
    see "then press Approve."
    approve
    see "Ask temper"
    mark
    key char "/"
    see "Start a new chat"
    key escape
    key paste "What can you tell me about Hello?"
    key enter
    see "Hello from the local fake model."
    end
    ```

    The `see` words are the design's: "Ask temper" the launchpad's box placeholder, "Start
    a new chat" `/new`'s words in the commands list, "Hello from the local fake model."
    the fake box's model answering a cue it has no script for
    (`temper/crates/temper-fake/src/backends.rs`, `otherwise`). Escape's and Enter's
    events mark the frames drawn before them (`tui/journeys.md`, 3.3), so the frames are
    the launchpad (the `mark`), the commands (before Escape), the question typed (before
    Enter) and the chat answered (the `end`).
  - `README.md`: what the journey walks, then a line a frame, `01` to `04`, as
    `sign-in-approved/README.md` writes its own; the code, the port and the clocks the
    recording's own.
  - `heavy cargo run -p temper-client-journeys -- again launchpad-chat` records it, `add`
    redacting, proving and printing it as for any TUI journey.
- `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-journeys` passes:
  `the_committed_journeys_are_their_print` holds both journeys and every browser journey
  byte for byte, the overlay empty.
- `temper/docs/design/client/tui/journeys.status.md`: delete the **Not yet** entry "9:
  `sign-in-approved` recorded to the launchpad, and `launchpad-chat`, neither recorded."
  and nothing else (a stated departure from the implementer's text).
- **The account:** the last commit's message is the landing's account
  (`look-requests.md`, 5): a table, one row per row of
  `temper/docs/plan/tui-launchpad/notes/account.md`, each checked against the trunk
  this task starts from (the code, the goldens, the design, the status files) and
  written as it stands: landed; differs, and why; or deferred, to whom and where it is
  tracked. A row whose result is not the note's expected one says so and why. The
  render check's row says it follows landing and names its owner. Each commit carries
  the trailer `Look-request: tui-launchpad`; the last also says, above its trailers,
  "The account of the look request tui-launchpad."

## Inputs

- `temper/docs/design/client/tui/journeys.md`: 3.3, 8.1, 8.2 (*A setup*), 9.
- `temper/docs/design/client/journeys.md`: 8.2 (the setup's format and `after`).
- `temper/docs/design/client/README.md`: 6.14 (*The launchpad*, *No project yet*, *The
  foot*, *The commands*, *A chat*).
- `temper/docs/design/draft/look-requests.md`: 3 (steps 6 and 7) and 5.
- `temper/docs/plan/tui-launchpad/notes/common.md` (*Recording flakes*, generated
  files); `temper/docs/plan/tui-launchpad/notes/account.md`.

## Owns and may change

- Owns the two journeys' directories, the journeys' status, and, only for a mark's or a
  step's wait that a moving screen defeats three times, the TUI's `again` driver
  (`tui_again.rs`): widen that wait and say so (`notes/common.md`).
- **A `see` that never shows:** if the launchpad's words differ from those above on the
  trunk (the design's words moved in a later task), use the trunk's, the nearest the
  design names, and say so. If Ask temper's chat gets no answer on the fake box within
  the walk's bound, end `launchpad-chat`'s walk on the person's words in the chat
  (`see "What can you tell me about Hello?"` after `key enter`), record it, and say why
  in the report and in the account's journey row; that is no question.
- If `project-landing`'s setup fails on the builder (its Forgejo), report the failure
  with its log and record `sign-in-approved` alone; the account's journey row says
  `launchpad-chat` is deferred, to temper/tui, with the reason.
- A browser journey's files are another owner's: none changes.
- Every other task of the plan has landed when this starts: a red it inherits on the
  trunk is reported under `## Left`, never repaired here unless it is a journey's print.

## Steps

1. The walks, the setup and the READMEs.
2. `heavy cargo run -p temper-client-journeys -- again sign-in-approved`, then `again
   launchpad-chat` (each retried up to three times on a moving screen, each on a fresh
   fake box, as the driver does itself); `heavy cargo run -p temper-client-journeys` to
   reprint. If `again` needs Chromium or a build the builder lacks, report `Failed:`
   with what it lacked.
3. `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-journeys`; `cargo fmt`
   if a source changed; `heavy cargo clippy -p temper-client-journeys --all-targets -- -D
   warnings` if a source changed.
4. The account, checked row by row; commit with it as the last commit's message and the
   trailer `Look-request: tui-launchpad`; push.

## Report

As the implementer's text says: each recording's attempts, the frames each journey
printed, and the account as committed.

## Answers

2026-11-08. **Q:** `launchpad-chat/setup` without `reload` is refused ("missing
reload"): should `journey-setup` make the setup reader take it, or may this task add a
`reload`? **A:** add the `reload`; the setup is four lines, as the acceptance now has it
(`reload "/projects/1"` last, as `project-landing`'s own). A TUI setup is "in the
browser's format and with its rules" (`temper/docs/design/client/tui/journeys.md`, 8.2),
and the browser's rules have `reload` once, last of the steps
(`temper/docs/design/client/journeys.md`, 8.2): the three-line setup this spec gave was
the spec's error, not the reader's. The reload loads Hello's Home in the signed-in
browser before the TUI starts, and nothing of it reaches the recording. No code, no
shared reader and no other task changes; the status file still loses only its **Not
yet** entry. Keep the pushed `sign-in-approved` work and go on from step 2's `again
launchpad-chat`.
