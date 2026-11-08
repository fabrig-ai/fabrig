+++
id = "account"
part = "account"
kind = "docs"
needs = ["old-look-out"]
files = ["temper/docs/design/client/README.status.md", "temper/docs/design/client/browser.status.md", "temper/docs/design/client/forge.status.md", "temper/docs/design/protocol/sign-in.status.md", "temper/docs/design/STATUS.md", "temper/docs/design/client/mockups/STATUS.md", "temper/docs/design/client/journeys/welcome-set-up/README.md", "temper/docs/design/client/journeys/welcome-playground/README.md"]
effort = "medium"
minutes = 45
+++

# look-outside-pages: the status entries close, and the landing's account

## Goal

The plan's last task. Check every row of the plan's account note against the trunk,
close the design's **Not yet** entries this plan opened, bring the journeys' and mockups'
words in line (`temper/docs/design/draft/look-requests.md`, 3, step 7), and write the
landing's account in the last commit's message (`look-requests.md`, 5).

Acceptance:

- Each row of `temper/docs/plan/look-outside-pages/notes/account.md` checked against the
  trunk this task starts from (the code, `temper.css`, the tests, the design): the commit
  message's table has one row per row of the note, written as it stands: landed; differs,
  and why; or deferred, to whom and where tracked. A row whose result isn't the note's
  says so and why. The render check's row says it follows landing and names its owner.
- Departing from the implementer's text (which keeps `docs/design/` closed), this task
  edits design status files: it deletes the **Not yet** entries the plan opened, those
  naming the look request `outside-pages` in `client/README.status.md`,
  `client/browser.status.md`, `client/forge.status.md` and
  `protocol/sign-in.status.md`, where the trunk shows them built; an entry the trunk does
  not show built is narrowed to what stays, and its row says deferred to temper/ui. It
  updates each file's date line to the day it runs (`date -u`), and
  `temper/docs/design/STATUS.md`'s summary if it names these gaps.
- `client/mockups/STATUS.md`: one line saying the look request `outside-pages` draws every
  page outside a project in the welcome's look (`../README.md`, 5.1; `../browser.md`,
  2.5), so the mockups' banner over those pages is not the client's. The two journeys'
  `README.md` files: a line describing a frame that is now the top bar is brought in line,
  nothing else.
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new;
  `heavy cargo nextest run -p temper-client-journeys` passes (its `README.md` files are
  read by it).

## Inputs

- `temper/docs/design/draft/look-requests.md`, 3 (step 7) and 5.
- `temper/docs/plan/look-outside-pages/notes/account.md`;
  `temper/docs/plan/look-outside-pages/notes/common.md`.
- `DESIGN.md`, 2 (status files: gaps as they stand, no history).

## Owns and may change

- Owns: the files above, only as Acceptance says.
- Read, never change: any design document's text (`README.md`, `browser.md`, `forge.md`,
  `sign-in.md`); code. A gap you find in the code is a row, not a fix.

## Steps

1. Read each row's place on the trunk; note its result.
2. The status edits, the mockups' line, the journeys' lines.
3. Citations; the journeys' focused tests.
4. Commit with the account in the message and the trailer (common.md), push.

## Report

As the implementer's text says, with the account's table.
