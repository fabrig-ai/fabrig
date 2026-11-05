+++
id = "goal-browser"
part = "goal-browser"
kind = "feature"
needs = ["plan-filter-out"]
files = [
  "temper/tests/client-browser/tests/browser_all/browser_task_pages.rs",
  "temper/tests/client-browser/src/lib.rs",
  "temper/crates/temper/tests/fake_backends/walk/goal.rs",
  "temper/crates/temper-fake/src/backends.rs",
]
effort = "high"
minutes = 75
+++

# Start goal, walked in a browser and on the fake box

## Goal

Both real sides of the page, in the worlds that hold them: a browser tier story from Goals' sheet to the goal's page, and the fake box's walk reaching its report from there.

Acceptance:

- **Browser:** `start_goal_opens_on_its_goals_page`, in the existing harness (`browser_task_pages.rs`, `Tier`, `Step::Person`): Goals, "Start one from a draft", the sheet, the Draft and Name fields, "Start goal", `Step::Address` of the goal's `/tasks/N`, its `<h1>`.
- **The fake box's walk,** `fake_backends/walk/goal.rs`: `a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list` still passes, and goes on to open its row: the goal's page, its headline in `<h1>`, "Report", "Landing".
- **A stated gap, if any:** what the fake model's script can't show. See Options.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.8, 5.14, 10; `browser.md`, 4.3 on the tier's budget.
- `temper/crates/temper-fake/src/backends.rs`, `model_document`: the cue "You are a goal".

## Owns and may change

Owns, as `files` lists them.

Options, each pre-approved:

- **A person's goal on the fake box:** `model_document`'s goal cue names `"goal":2`, the first chat's goal, and refuses others its scope: its comment says so. A goal started from the sheet on a fresh box is task 1, so it ends without a report: the browser story asserts its page, and that's enough. If a small cue
  (a goal whose words hold `Build docs/design/draft/`, scope `"goal":1`, one report note, then waiting on a delegate, or idle) lets `goal-proof` show planning, a report and Pause, add it, and say so; `first-run-walks` and the fake-box tests that read this document must still pass. If it can't, list what the script can't show.
- **Tier's budget:** this is one story. If it takes the tier past its budget, say so with measurements: it isn't yours to raise.
- A browser run needs `TEMPER_BUNDLE` unset for the tier, as the tier's own documentation says.

## Steps

1. The browser story, then the walk's extension. Read one existing story through for the harness's way of filling a field.
2. `heavy cargo nextest run -p temper-client-browser-world --test browser_all -E 'test(start_goal)'`; the walk through the package's own profile, as `temper/docs/development/qa.md` or `tools/tiers/` says.
3. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report: the run's minutes, and any state the script can't show.

## Answers

2026-11-05 (fabrig/ui). Replace it: this task's browser acceptance goes to Goals, Start one from a draft, Chats' Drafts filter; and the sheet in the gallery or a world's state. The full chat-draft-to-goal walk is ui-chat-draft's draft-proof, which I'll add. The rulings are coord/ui/report/rulings.md on hel8, not in the repository. Finish without it.
