+++
id = "walk"
part = "walk"
kind = "docs"
needs = []
files = [
  "temper/docs/plan/ui-final-walk/report.md",
  "temper/docs/plan/ui-final-walk/findings.md",
  "temper/docs/plan/ui-final-walk/driver/**",
  "temper/crates/temper-fake/src/backends.rs",
]
effort = "high"
minutes = 150
+++

# The first walk: the whole client against the mock, producing findings

## Goal

Walk the request's seven steps as a person would, on fresh fake boxes, at 1440 and again at 420, beside the mock, and write down what differs. This task fixes nothing in the client: its product is the table, the findings and a driver that `walk-again` can replay.

Acceptance:

- `temper/docs/plan/ui-final-walk/driver/` walks all seven steps at either width from one command, with the box or boxes it needs, and says how to run it in a short `README.md` there.
- `report.md` has a row for each state each step names, at both widths, as `notes/walk.md`, The table says. A state left out is a failure of this task; a state the fake can't reach is `unreachable`, with how it was shown instead.
- `findings.md` has a row for each difference that no decision explains, as `notes/common.md` says, each with a size and an owner. A `large` one has its reproduction.
- The screenshots and `index.html` are on the builder, as `notes/walk.md`, The screenshots says, and `report.md` is copied beside them.
- Every box and Chromium you started is stopped.

## Inputs

- `temper/docs/plan/ui-final-walk/inputs/request.md`, all of it.
- `temper/docs/plan/ui-final-walk/notes/common.md`, `notes/walk.md`, `notes/decisions.md`.
- `temper/docs/design/client/greenfield/README.md`; the mock's pages and `app.js` as you need them.
- `temper/docs/design/client/README.md`, 3.5, only for the page you're looking at.
- `temper/docs/design/client/journeys/project-landing/README.md` and `chat-draft/README.md`: a walk already done, in words.
- `temper/docs/help/box.md`, *Before you start* and 2.

## Owns and may change

Owns, as `files` lists them.

May also change, for the walk's sake alone:

- `temper/crates/temper-fake/src/tests.rs` and any test that lists `model_document`'s scripts: the lines a script you add needs.
- A `temper-fake` limit that bounds scripts, if you reach it: `cued` and `asked` hold sixteen each, and have seven and three.

Options, each pre-approved, the first named being the default:

- **The fake's literals, `work/hello`, Home's brief, what the fake can't reach:** as `notes/walk.md`, The fake model, says for each.
- **A walk that stops on a fault:** a state you can't get past is a finding with its reproduction. Reach what's beyond it on another fresh box or by `temper ctl`, say so on those rows, and go on. Don't fix it here, unless the fault is in your driver or the fake's scripts.
- **The driver's language:** Python's standard library or Node, whichever the builder has without installing anything. No new dependency in the repository.
- **A word the mock has and the client's data can't:** a listed decision if there is one; a finding if not.

## Steps

1. Build and copy; run one box by hand far enough to see the driver's shape.
2. The driver, step by step, at 1440, taking the pairs as it goes. Then at 420.
3. The table, the findings, `index.html`.
4. If you added scripts: `heavy cargo nextest run -p temper-fake --lib`, and the walks that read them, in the process tier's profile, one at a time: `heavy cargo nextest run --profile process-temper -p temper --test process -E 'test(a_chat_document_is_written_and_revised_on_the_fake_box)'`, then likewise `a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off` and `home_on_the_fake_box_keeps_a_landing_count_after_box_and_client_restarts`. `cargo fmt`, and `heavy cargo clippy -p temper-fake --all-targets -- -D warnings`.
5. `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As `notes/common.md`, Your report, and: how many boxes, how long each walk took, the counts of each verdict, and which of the fake's scripts you added.
