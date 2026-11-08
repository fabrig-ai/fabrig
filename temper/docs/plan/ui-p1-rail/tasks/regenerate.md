+++
id = "regenerate"
part = "checks"
kind = "migration"
needs = ["rail", "work-page", "deployment"]
files = ["temper/crates/temper-client-shell-terminal/src/tests/golden/**", "temper/docs/design/client/journeys/**"]
effort = "medium"
minutes = 120
+++

# The terminal's goldens and the journeys, regenerated

## Goal

The rail, Work's head on Goals and Changes, and Deployment's frame change what the
line terminal prints and what every web journey's pages hold. Regenerate the line
terminal shell's goldens with their own tool, and re-record the journeys these
changes make stale with `temper-client-journeys again`, which is generation, not a
walk (the request's rules). Their recordings' presses of entries the rail no longer
draws, and their READMEs' words about the rail, follow the new rail.

Departure from the implementer's text, and why: **this task edits
`temper/docs/design/client/journeys/**`** (each journey's `README.md`, `recording`
and `setup` where they name the old rail), since a journey's recording and its
description are what the generation reads, and the tests read them
(`.config/gate/read-by-tests.txt`).

Acceptance:

1. `temper-client-shell-terminal`'s goldens are its print of the trunk's view,
   written by `SKEIN_CODEGEN_WRITE=1` through its own tests, and its focused tests
   pass without it. Each golden's diff is the rail's, Work's or Deployment's
   change, nothing else; a diff you can't explain so is a red, not stale.
2. Each web journey whose pages the changes make stale (`chat-draft`,
   `project-landing`, `welcome-playground`, `welcome-set-up`, as the replay tests
   find them) is re-recorded with `cargo run -p temper-client-journeys -- again
   NAME` (through heavy where it builds), after its `recording`'s presses of "New
   chat", "Settings", "Goals" or "Chats" in the rail are rewritten to the new names
   (`temper/docs/design/client/README.md`, 5.3). A re-record stopped on "a key mark
   caught a moving page" is retried up to three times, each on a fresh fake box;
   then the mark's wait is widened in the recording driver, and said.
3. Each journey's `README.md` says the rail's entries and the tab bar as 5.3 has
   them where it listed them (`project-landing`'s, `welcome-playground`'s,
   `chat-draft`'s).
4. The replay tests pass: `temper-client-journeys`' (`the_committed_journeys_are_their_print`,
   `the_committed_journey_redacted_prints_the_same_pages`,
   `every_journey_has_its_description`, `every_committed_setup_reads_and_joins`) and
   `temper-client-shell-web`'s `a_committed_journey_redacted_replays`.
5. The TUI's journeys are untouched: the TUI draws no web rail.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md`, and the request's rules on
  recordings and generation (`temper/docs/plan/ui-p1-rail/inputs/request.md`).
- `temper/docs/design/client/journeys.md`: how a journey is recorded and replayed.
- `temper/docs/design/client/README.md`, 5.3; 3.5, *Work* and *Settings and
  Deployment*.
- The survey's leads: `chat-draft/recording` (a press named "New chat", around
  line 150) and `chat-draft/setup` (the Settings address it reloads, which still
  reads); `project-landing/README.md` (18-20), `welcome-playground/README.md`
  (12-15), `chat-draft/README.md` (10, 28).

## Owns and may change

Owns: the goldens and the journeys' directories in `files`.

Not this task's: any code, the view's or the gallery's tests, the stories
(`readers`). A journey that won't record for a reason other than a moving mark is a
question with its log.

Pre-approved options:

- **If a journey's recording reaches a page through "Settings":** reach it through
  "How work runs" or "Deployment", whichever holds that page now.
- **If a golden changes only by a control's number** after a removed rail line:
  that is the rail's change.

## Steps

1. Regenerate the goldens; read each diff by `git diff --stat` and its hunks.
2. For each stale journey: rewrite its recording's presses, then `again NAME`;
   check its pages by the replay tests.
3. Rewrite the READMEs' rail words.
4. Run acceptance 4's tests and `temper-client-shell-terminal`'s.
5. Commit and push: the message says the prints follow the rail, Work and
   Deployment.

## Report

Follow the implementer's text. Also list each golden and journey regenerated, each
re-record's tries, and any mark widened.

## Answers

2026-11-08. *May chat-draft start on Chats (`/chats`) and use its page's
"+ New chat" link, or should a repair teach the recorder the rail's menu?*
Start on Chats; no code repair. The rail drops Chats, but `Chats` keeps
its address (`temper/docs/design/draft/surfaces.md`, 2), and its page's
"+ New chat" goes to the same `NewChat { draft: false }` the recording
already presses. An open menu is browser-local state the journeys don't
record (`temper/docs/design/client/journeys.md`, 3.4), and teaching them
is code outside this migration. So, in `journeys/chat-draft/` only:
`setup` reloads `/chats` in place of `/projects/1/settings`; the
recording's press is named as the Chats page's link reads; the `README.md`
starts the recording at Chats and says so for `01.html` and `02.html`
(marked, then before opening "+ New chat"), keeping the rest of the
story. Then `again chat-draft`, with the moving-mark retries as before,
and acceptance 4's tests. If Chats under `/chats` shows no project's
"+ New chat" or the draft's "+ Draft" needs the How work runs start, that
is a question with its log.
