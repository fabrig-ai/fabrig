+++
id = "goals-gallery"
part = "goals-gallery"
kind = "feature"
needs = ["goals-page"]
files = [
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 90
+++

# Goals in the gallery

## Goal

The gallery prints Goals in its states, and the Solo and Reference worlds gain goals, notes and an inbox selection.

Acceptance:

- States `goals-solo` (rows in each state), `goals-reference`, `goals-none`, `goals-needs`, `goals-under-way`, `goals-ended`,
  `goals-filter-empty`, `goals-more`, as `temper/docs/design/client/browser.md`, 7.2 says, in `names.txt` (sorted as it is), `samples.rs`'s
  table with `N` and `tests.rs`'s `HOLDS`, each holding what its name says.
- Solo's and Reference's `me` lists goal templates. `goals-reference`'s lede names four ways in number order, as `goals::ways` gives them: templates `goal`, number 2 as the stock's, `goal for a draft`, 4 as the
  reference's, then "Plan, then wait for me" and "One agent, one branch". So its first two stand the other way round from the mock's, whose ways aren't numbered: name it in your report. Words from `temper/docs/design/client/greenfield/data.js`, where the client holds the fact.
- **The trap:** `c::Subject::Tasks { window, .. }` (`worlds.rs:241`) and `c::Read::Tasks { .. }` ignore the query and answer chat rows. Split on the
  query's `tracked: Some(true)`, so that no chat page shows a goal and no goals page a chat; `home_domain` (`samples.rs`) has its own loop: the same care.
- A goal's `Read::Notes` and the needs' `Read::Inbox` answer from the world's data, its `selection` naming the goals.
- Every other state's print is unmoved, by the gallery's tests.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/browser.md`, 7, 7.2.
- `temper/tools/gallery/src/samples.rs`, `home_domain`; `worlds.rs`, `facts`, `task_line`, `read_value`; `tests.rs`.

## Owns and may change

Owns, as `files` lists them. May also change `temper/docs/design/client/browser.md`'s rows for these states if a row's words mislead, and `temper/docs/design/client/journeys*`
only if a test reads them for a gallery name.

Options, each pre-approved:

- **`goals_domain(name)`,** dispatched as `home-` is, where a state is simpler by hand; or the world's own `then`. Your call per state.
- The worlds' `data.js` ways aren't templates: Reference's `me` gains templates of those names, marked `tracked`.
- A goal of a hundred tasks in Reference: `delegates_done` 96, `delegates_total` 112, with no hundred rows.

## Steps

1. **Worlds:** goals, notes and selection, as data beside `facts`.
2. **Samples,** the three lists in step.
3. **Print once and look:** `heavy cargo run -p temper-client-gallery` as `temper/docs/design/client/browser.md`, 7.1 says, a throwaway look at each state beside `goals.html`.
4. **Check:** `heavy cargo nextest run -p temper-client-gallery`, the implementer's text, then push.

## Report

As `notes/common.md`, Your report, naming any state that differs from the mock and why.
