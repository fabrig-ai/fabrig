+++
id = "public-walk"
part = "temper-public"
kind = "feature"
needs = ["forge-public", "public-web", "public-ctl", "visibility-core"]
files = ["temper/tests/client-browser/tests/browser_all/browser_public.rs", "temper/tests/client-browser/tests/browser_all/main.rs", "temper/tests/client-browser/src/public.rs", "temper/tests/client-browser/src/lib.rs", "temper/tests/client-browser/src/configuration.rs"]
effort = "high"
minutes = 120
+++

# The walk: a visitor reads a public project in Chromium, and loses it when it is made private

## Goal

The plan's walk slice, in temper's browser tier (`temper/tests/client-browser`,
`temper/docs/design/client/browser.md`, 8.4 and section 4): on the box's demo with its fakes,
an owner's goal makes a change that lands; the owner makes the project public with
`temper ctl project amend`; a visitor with no sign-in reads `/public`, the project's board,
the goal's task page with its change, and the landings, each page's text checked against the
public records' fields alone; then the project is made private and its pages say "Not
public, or not here." at their next read. Every part built by this plan's tasks runs here at
once: jig's index and reads, the component and its `GET`, the forge's projections, the web
client's public mode and ctl.

Acceptance:

- **The deployment,** a tier of its own (`src/public.rs`, `Tier::public_demo(name)` beside
  `Tier::demo`, its documents written by `configuration.rs` for this tier alone): the demo as
  `Tier::demo` writes it, with public reading figured on in `engine.limits.overrides` at
  `temper/docs/design/draft/first-release.status.md`'s proposed figures (`clients.public.rate`
  20, `rate_burst` 20, `bytes` and `bytes_burst` 2 MiB, `answer_bytes` 256 KiB, `reads` 8,
  `clients_http.public_calls` 8, `core.public.goals` 20, `ended` 20, `lines` 50, `depth` 16),
  but `clients.public.max_age` 2, so a private project's pages go within the story's wait; and
  the demo's scripted CSV chat's `delegate` batch member given `"tracked": 1` in this tier's
  copy of `application.demo.llm.asked`, so the change it delegates is a goal under the chat,
  a public goal by jig's rule (`jig/docs/design/domain/people.md`, 13.7). No other tier's
  documents change.
- **The story,** `browser_public.rs`,
  `a_visitor_reads_a_public_project_and_loses_it_when_made_private`:
  1. the owner starts the playground and the CSV chat, as `browser_task_pages.rs` does, and
     waits for the change to land;
  2. `temper ctl project amend Demo --visibility public --key public` (a `Step::Run` of the
     harness, in a ctl directory signed in with the link `temper serve` printed), which prints
     `project N: public` and `page: /public/projects/<16 hex>`; the story keeps the path;
  3. a visitor with no cookie (a new browser profile or `Step::Reopen` after signing out, as
     the harness allows) visits `/public`: the project's link "Project <8 hex>" shows; it
     follows it to the board (the goal's line, "Goal <8 hex>", its phase in words), the goal's
     task page (its line and its change: step, CI's state, gates by number and kind, "its
     change on the forge"), and the landings page; the address bar shows each public path;
  4. on each page, the text the harness reads holds none of: the goal's words ("Export
     widget's table as CSV"), the chat's words, the project's name `Demo`, the repository's
     branch, a gate's name, any party's login or name, and any task's or project's number as
     the signed-in pages print it ("Task 2", "T3"); and no page shows a signed-in banner, a
     stream's status or a person's name;
  5. `temper ctl project amend Demo --visibility private --key private`; within the wait
     (`WITHIN`, past the 2-second `max-age`), the open task page says "Not public, or not
     here." with a link to `/public`, and `/public` lists no project;
  6. a path under `/public` that does not read (an upper-case identifier) shows `/public`.
- The story runs in the `browser-temper` profile (`tools/tiers/run temper browser`, through
  `temper/scripts/browser.sh`), within the tier's budget.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`.
- `temper/docs/design/client/browser.md`, section 4 (the browser tier) and section 8;
  `temper/docs/design/client/tools.md`, 5.3 (`project amend`); jig's
  `jig/docs/design/domain/people.md`, 13.2 and 13.7; `temper/docs/design/protocol/sign-in.md`,
  2.4 (off until figured).
- `tasks/public-ctl.md` (what ctl prints), `tasks/public-web.md` (the pages' words and
  addresses), `tasks/forge-public.md`.
- As they stand: `temper/tests/client-browser`'s `src/lib.rs` (`Tier`, `Tier::demo`, `WITHIN`,
  `ABSENT`), `src/configuration.rs` (`demo`, the overrides `documents` sets),
  `tests/browser_all/browser_task_pages.rs` (the demo's CSV chat to its change),
  `browser_sign_in.rs` (signing out); skein's `skein-fake-person-browser`'s
  `harness/script.rs` (`Step::Run`, `Step::Visit`, `Step::Reopen`, `Step::Address`).

## Owns and may change

- Owns the files in the header; in `lib.rs` and `main.rs` only a module line and
  `Tier::public_demo`'s entry; in `configuration.rs` only a function for this tier.
- Nothing outside `temper/tests/client-browser`. A fault the walk finds in a part another
  task built is reported with the failing step and its log, never repaired here: report it
  under `## Left`, with the test left failing only if the fault is real, and say so in the
  first line.
- Pre-approved:
  - If the harness cannot run `temper ctl` mid-story, run the two scripts on one server in
    turn (the owner's, then ctl, then the visitor's), as the harness allows, and say how.
  - If the demo lands its change without a landing by turn, the landings page shows its
    queues and turn empty: assert that page's empty state and say so.
  - If the tracked delegate is refused by the demo's policy, give the tier's chat template
    the authority it needs in the same documents, and say which field.
  - If `public` cannot be listed as a project link because jig's *projects* is read before
    the project is public, read `/public` again after the `max-age`.

## Steps

1. The tier and its documents; build the server and the bundle as the browser tier says
   (`heavy cargo build -p temper` and `temper/scripts/bundle.sh`, through heavy).
2. The story, step by step; run it alone
   (`heavy python3 -I -B tools/tiers/run temper browser -- -E 'test(=browser_public::a_visitor_reads_a_public_project_and_loses_it_when_made_private)'`,
   or the form `temper/scripts/browser.sh` takes).
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-browser-world --all-targets -- -D warnings`; the citations check.
4. Commit (why: the plan's walk, every part of public reading at once in a real browser);
   push your branch.

## Report

As the implementer's text says, and also: the story's steps as they ran with their times, the
pages' text the checks read (one page in full), and how ctl was run in the story.
