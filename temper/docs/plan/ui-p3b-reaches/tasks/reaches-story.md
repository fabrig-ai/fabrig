+++
id = "reaches-story"
part = "story"
kind = "feature"
needs = ["reaches-page"]
files = ["temper/tests/client/tests/suite/away.rs"]
effort = "high"
minutes = 50
+++

## Goal

Show, through temper's engine in the client world, that How work runs'
What reaches you is drawn from the project's settings and follows their
change: the request's "the classes drawn from the project's settings"
(`temper/docs/design/client/README.md`, 5.13, *What reaches you*). It
grows the joint world's existing story of N1's classes rather than adding
a test.

Acceptance:

- In `temper/tests/client/tests/suite/away.rs`,
  `what_reaches_a_person_follows_the_classes_away_and_stop_asking` gains
  two visits of How work runs (`Address::Settings { project: PROJECT }`),
  each followed by the story's own way back to the need it was on:
  - after the first "Reached you because it is an implementation choice…"
    and before pressing "Away": on Agents review's stock classes, the
    group "Asks you" holds "implementation choices a spec leaves open",
    "a new dependency" and "a departure from a landed design", in that
    order, and the small line "While you’re away, they are decided with
    the recommended choice and listed."; the group "Agents decide, then
    tell you" holds "wording and names inside a spec", "retrying or
    splitting a task" and "test-only changes"; "Always waits for you"
    holds the four fixed kinds' lines; and the heading "What reaches you"
    with "on PROJECT" at its right;
  - after "Applied: the next one is decided and listed. This one still
    waits for you.": the line "implementation choices a spec leaves open"
    now stands in "Agents decide, then tell you", after "retrying or
    splitting a task" and before "test-only changes", and "Asks you"
    holds the other two alone, with the same small line: the project's
    classes as Stop asking's Apply changed them, read again by the page.
- Two helpers on the story beside `needs` and `view`: one visiting How
  work runs, one reading a column's lines by its group's name from
  `view()`'s tree (`notes/contracts.md`, section 2's shape).
- The story's other assertions stay as they are, and it still passes.

## Inputs

- `temper/docs/plan/ui-p3b-reaches/notes/common.md`; `notes/contracts.md`,
  section 2.
- `temper/docs/design/client/README.md`, 5.13, *What reaches you* and
  *What the client holds* (kind 8 read as the project opens and again
  once changed); 3.5, *Needs you*, *Stop asking*'s *Its act*.
- The story and its helpers: `temper/tests/client/tests/suite/away.rs`,
  `Story` (`visit_home`, `needs`, `open_need`, `view`, `sees`), and its
  `keep_settings` view, the browser's words.

## Owns and may change

- **Owns:** the file in this spec's header.
- **Options, pre-approved:**
  - if visiting How work runs leaves the story elsewhere than its next
    step expects, return it with the story's own helpers (`needs`,
    `open_need`) before that step, and say so;
  - if the page's read of kind 8 after Apply needs the story's clock to
    move, move it with `story.after` as the story does elsewhere;
  - if a drawn word differs from this spec's but matches
    `notes/contracts.md`, section 2, the note wins: assert the note's,
    and say so.

## Steps

1. The helpers and the two visits.
2. Break the rule once by hand (assert the moved line still in "Asks
   you") and see the story fail there; restore.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-world --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-client-world -E
   'test(=away::what_reaches_a_person_follows_the_classes_away_and_stop_asking)'`,
   one test selected, then `heavy cargo nextest run -p temper-client-world
   -E 'test(/^away::/)'`. Commit and push.

## Report

As the implementer's text asks, with the story's CPU time before and
after.
