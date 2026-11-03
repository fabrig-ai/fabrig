+++
id = "mine-title"
part = "mine"
kind = "feature"
needs = ["mine-start"]
files = ["temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs", "temper/tests/engine/domain/tests/suite/forge_routes/goal_landing_title.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs"]
effort = "medium"
minutes = 45
+++

# mine-title: the goal's landing's pull request is titled by its words' first line

## Goal

A change on its requester's held branch carries its branch as parameter 3, as a handed
branch does, so its pull request's title is parameter 10 or, without it, "Update
repository" (`initial_change_title`). The goal's landing's title is its words' first line
(`temper/docs/design/domain/forge.md`, 8.1, "A change on its requester's held branch"):
the root supplies it as parameter 10 when it builds the delegate.

Acceptance:
- `change` with `{"words":"Land the reference goal\nWhat it built: ...","branch":"mine"}`
  builds a delegate whose spec carries `Bytes { name: 10, value: b"Land the reference
  goal" }` after parameter 11; its pull request is opened with that title.
- Words whose first line is empty (they begin with `\n`) carry no parameter 10, so the
  title is the handed path's default; words with no newline carry them whole as the title,
  `initial_change_title` fitting them to 256 bytes as it does.
- A change with its own branch, or handed one, is unchanged.
- Every `forge_routes` story and `temper-engine-domain`'s lib tests pass.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.1 (the title bullet: parameter 10, its fitting; "A
  change on its requester's held branch", its first bullet).
- `temper/docs/plan/reference-m2/notes/contracts.md`, C4.
- `temper/docs/plan/reference-m2/notes/common.md`.

## Owns and may change

Owns the files in the header; in `forge_route.rs`, only the lines `mine-start` added to
build a `mine` change's parameters (in `chat_change` or its caller). Other plans change this
file: keep the hunk there.

- The first line is the words up to their first `\n` (a `\r` before it dropped), not
  trimmed otherwise; the bound is `initial_change_title`'s, not this code's.
- The test: a unit test in `route/forge_route/tests.rs` beside `mine-start`'s
  `a_change_on_its_requesters_branch_asks_no_write_holding`, if that module can build the
  delegate (`change_batch`) from a fixture; otherwise a story in a new
  `forge_routes/goal_landing_title.rs`, its `mod goal_landing_title;` line placed after
  `mod documents;` in `forge_routes.rs` (not beside `mod goal_landing;`, where another task
  adds a line), built from `goal_branch.rs`'s helpers (`pub(super)` since `mine-start`;
  this task changes no line of `goal_landing.rs`, which another task appends to):
  `a_goals_landing_is_titled_by_its_words_first_line`, asserting the delegate's parameters
  and the fake forge's pull request title.

## Steps

1. The test, then the code.
2. Narrow: the new test alone (`-E 'test(=...)'`, check it selected 1); then `heavy cargo
   nextest run -p temper-engine-domain-world --test suite -E 'test(/forge_routes::goal_landing/)'`
   and `heavy cargo nextest run -p temper-engine-domain --lib`.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`. Commit
   ("change: the goal's landing is titled by its words' first line") and push.

## Report

As the implementer's text says.
