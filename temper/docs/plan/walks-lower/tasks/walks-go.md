+++
id = "walks-go"
part = "prune"
kind = "migration"
needs = ["status-stories", "away-story", "goal-story", "model-story", "issuer-sockets", "issuer-log", "note-parts", "chat-document", "copy-ctl", "copy-ready"]
files = ["temper/crates/temper/tests/fake_backends/**", "temper/crates/temper/tests/support/**", ".config/nextest.toml", "temper/docs/design/domain/testing.status.md", "temper/docs/design/protocol/setup.status.md", "temper/docs/design/draft/repository-copies.status.md", "temper/docs/design/domain/agent.status.md", "temper/docs/design/client/README.status.md", "temper/docs/design/STATUS.md"]
effort = "medium"
minutes = 75
+++

# The fake box keeps two walks: the ten whose stories run lower go

## Goal

Every walk under `temper/crates/temper/tests/fake_backends/walk/` but two
now has its lower story on the trunk (`temper/docs/plan/walks-lower/plan.md`,
the table). For each, run that story by name, check it holds the
request's keeps, and only then delete the walk, its helpers that nothing
else reads, and every line that names it; the status files then say
where each behaviour is proved. The two that stay are
`here_key::forgejo_here_and_a_key_reach_a_first_chat_on_the_fake_box`
and `sign_in::a_chatgpt_browser_tab_on_the_fake_box_leaves_ready_able_to_create`.

Acceptance:

- **Each row, in turn:** its lower story (the table below) passes, run
  alone by name on your base; you read it and find an assertion for each
  keep the request's ruling names for that walk
  (`temper/docs/plan/walks-lower/inputs/request.md`); then the walk goes.
  A row whose story is missing, fails, or lacks a keep's assertion keeps
  its walk: leave it, and say which and why under `## Left` (that is no
  failure of yours).

  | Walk (`fake_backends::walk::…`) | Its lower story |
  |---|---|
  | `status::a_plans_project_has_its_brief_on_a_return_past_the_sitting` | `temper-client-world` `suite` `status::a_plans_project_has_its_brief_on_a_return_past_the_sitting` |
  | `status::home_on_the_fake_box_keeps_a_landing_count_after_box_and_client_restarts` | `temper-client-world` `suite` `status::a_restart_keeps_the_rows` |
  | `away::a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off` | `temper-client-world` `suite` `away::a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off` |
  | `goal::a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list` | `temper-client-world` `suite` `task::a_chats_goal_shows_its_report_from_its_goals_row` |
  | `sign_in::a_chatgpt_sign_in_on_the_fake_box_leaves_a_usable_model_and_ready_can_create` | `temper-client-world` `suite` `connect_model::a_held_welcome_sign_in_moves_on_once_when_the_watch_makes_its_account` (or `connect_model::a_welcome_sign_in_makes_its_account_usable`, whichever `model-story` left) |
  | `sign_in::a_chatgpt_sign_in_beside_a_browsers_idle_issuer_sockets_leaves_ready_able_to_create` | `temper-real-loop-world` `suite` `sign_in::a_sign_in_beside_a_browsers_idle_issuer_sockets_completes_and_past_the_cap_is_turned_away` and `temper` `suite` `fake::the_oauth_fake_logs_each_connection_it_turns_away_at_its_cap` |
  | `note_parts::a_long_document_is_built_in_parts_on_the_fake_box` | `temper-real-loop-world` `suite` `notes::a_long_document_is_built_in_parts_and_revised_by_a_fresh_run` |
  | `chat_document::a_chat_document_is_written_and_revised_on_the_fake_box` | `temper-real-loop-world` `suite` `notes::a_chats_document_is_revised_whole_after_a_follow_up` |
  | `copy::a_copy_is_made_and_adopted_through_ctl_on_the_fake_box` | `temper-one-process-world` `suite` `copy::a_copy_beside_a_repository_joins_its_project_keeping_the_sources_workflow` |
  | `copy::readys_two_copies_are_adopted_in_order_with_the_first_as_home` | `temper-one-process-world` `suite` `ready_copies::readys_two_copies_are_made_in_order_and_adopted_with_the_first_as_home` |

- **What goes with them:** the files `away.rs`, `chat_document.rs`,
  `copy.rs`, `goal.rs`, `note_parts.rs`, `status.rs` and their `mod`
  lines in `walk.rs`; in `sign_in.rs`, the two moving tests, `browser_hops`
  and the imports only they used; in `walk.rs`, `Act::Visit` (its arm and
  its exclusion from the count), `Walk::short_paces` and `Walk::run`, if
  nothing left reads them. Under `temper/crates/temper/tests/support/`,
  only what the compiler or clippy then calls unused in the `process`
  binary; `ctl::Step::Visit` and what it uses stay, since the focused
  `suite::support_tests` story `a_returned_visit_drains_without_another_process_or_a_poll`
  reads them. `open_default.rs`, `program_here.rs`, `here_key.rs`,
  `fake_backends/mod.rs` and `box_link.rs` stay whole: the two walks
  read all of them.
- **`.config/nextest.toml`,** the `process-temper` override: the status
  walk's arm goes, and the filter names the browser-tab walk exactly
  (`test(=fake_backends::walk::sign_in::a_chatgpt_browser_tab_on_the_fake_box_leaves_ready_able_to_create)`),
  its comment saying the one walk that signs in to ChatGPT on the fake
  box, where the callback's port is fixed. The group, the profile's
  limits and `tools/tiers/budgets.toml` stay as they are.
- **The status files say where each behaviour is proved now,** as
  `DESIGN.md`, section 2 asks (gaps as they stand, no history):
  - `temper/docs/design/domain/testing.status.md`: the **Not yet** entry
    of 2.6 *The fake box keeps two walks* goes; a walk that stayed gets
    a **Departs** entry in its place naming it and the keep its lower
    world lacks. The 2.6 entry on the serialized fixed-callback walks
    says one walk; the 2.6 paragraph on Goal's, Away's and the status
    walks' waits loses what it says of those walks, keeping what it says
    of the walks that stay and its dated figures.
  - `temper/docs/design/protocol/setup.status.md` (about 108): the
    idle-socket walk's clause cites the two issuer stories instead.
  - `temper/docs/design/draft/repository-copies.status.md` (about 32):
    the walk named as the proof becomes the one-process stories; the
    limits it states (no real public host reached) stay.
  - `temper/docs/design/domain/agent.status.md` (about 161): the
    note-parts walk becomes the real loop's story; the real-model and
    drawn-pane gaps stay.
  - `temper/docs/design/client/README.status.md` (about 545 and 888): the
    goal walk and the plan's return become the joint world's stories,
    the return past the sitting on the world's clock (31 minutes) rather
    than a test's seconds.
  - `temper/docs/design/STATUS.md`: only where its summary names a walk
    that went.
- `python3 -I -B temper/scripts/old_forms.py --write` lowers the counts
  for what went, and nothing rises.
- The `process` binary compiles (`heavy cargo check -p temper --tests`);
  clippy is clean on `temper` with all targets; the `temper` package's
  focused `suite` binary passes; the citations check is clean.

## Inputs

- `temper/docs/plan/walks-lower/notes/common.md` (read first).
- `temper/docs/plan/walks-lower/inputs/request.md` (the keeps per walk;
  *What's wanted*, items 2 and 3).
- `temper/docs/plan/walks-lower/plan.md` (the table: where each walk
  went and why).
- `temper/docs/design/domain/testing.md`, 2.6, the bullet **The fake box
  keeps two walks**.
- `DESIGN.md`, sections 2 and 4 (status files; citations).

Departures from the implementer's text, and why:

- **Design files:** the status files above are yours to edit as stated:
  they name the walks that go, and the plan's cut designed their
  replacement. Touch no other `docs/design/` text; `testing.md` itself
  already says the target.
- **The `process` binary is compiled, never run** (the user, 11-06);
  the lower stories are run, by name, alone.

## Owns and may change

Owns:

- `temper/crates/temper/tests/fake_backends/**`.
- `temper/crates/temper/tests/support/**`: removals of what goes unused
  alone.
- `.config/nextest.toml`: the `process-temper` override's filter and
  comment.
- The status files above, the lines stated.

Pre-approved options:

- **Shared support another process story reads:** keep it.
- **A status line another draft has changed since:** edit only the
  clause naming the walk, keeping theirs.

## Steps

1. Read the inputs, `walk.rs` and `sign_in.rs`.
2. For each row: run its story alone,
   `heavy cargo nextest run -p PKG --test suite -E 'test(=NAME)'`
   (check the count is 1), read it against the request's keeps, then
   delete the walk and its `mod` line; after each deletion
   `heavy cargo check -p temper --tests`.
3. Remove what goes unused; `heavy cargo clippy -p temper --all-targets
   -- -D warnings`.
4. Edit `.config/nextest.toml` and the status files; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
5. Once, at the end: `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py --write`; `heavy cargo nextest run -p
   temper --test suite` (check the count). Commit (`walks-lower: the fake
   box keeps two walks`, the body listing each walk gone and the story
   that holds it) and push.

## Report

As the implementer's text says: the first line; the commit pushed; per
row, the story's run (count, pass) and the walk gone or kept with why;
what went from `support/`; the status lines changed; the old-forms
counts; the minutes.
