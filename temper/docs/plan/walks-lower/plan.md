# walks-lower: the plan

The request (`inputs/request.md`) moves the fake box's walks to where what
they check belongs. The design is `temper/docs/design/domain/testing.md`,
2.6, the bullet **The fake box keeps two walks**, written by this cut, with
its **Not yet** entry in `testing.status.md`. Every task follows
`notes/common.md`.

## The walks, one row each

Every walk under `temper/crates/temper/tests/fake_backends/walk/` at the
cut (main `98ba9009ab`): twelve tests in ten modules; `open_default` and
`program_here` hold helpers and no test. **Goes** means: its lower story
is written first, failing first where it guards a fix, and run there by
its task; `walks-go` runs each named story again, by name, on the trunk,
and only then deletes the walk.

| Walk | Where it goes | The story that takes it | Why |
|---|---|---|---|
| `here_key::forgejo_here_and_a_key_reach_a_first_chat_on_the_fake_box` | **stays** | itself | the user's one short end-to-end test through `main` (11-01) |
| `sign_in::a_chatgpt_browser_tab_on_the_fake_box_leaves_ready_able_to_create` | **stays** | itself | the plan's path: the box starts the fake issuer as a process, and `--fake-backends` journeys stand on it |
| `status::a_plans_project_has_its_brief_on_a_return_past_the_sitting` | joint world, `status-stories` | `temper/tests/client` `suite::status::a_plans_project_has_its_brief_on_a_return_past_the_sitting` | Home, the forge's sitting and the plan's funding are the client's and the engine's; the sitting is the world's clock there. Red with the stock's period at 80 × 24K, the fix it guards. Adds "no brief before a landing", which the walk never looked at |
| `status::home_on_the_fake_box_keeps_a_landing_count_after_box_and_client_restarts` | joint world, `status-stories` | `suite::status::a_restart_keeps_the_rows`, grown: Skip pressed before the landing, kept after the engine's restart and a fresh client, with the walk's words | the count after a real landing and the restart are already there; the skip was not |
| `away::a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off` | joint world, `away-story` | `suite::away::a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off` | the five Away stories hold its parts apart; the sequence through Overturn and off is new. The goal's run is played from the worker side through the real host-tool path, as the walk's was by a scripted model; the two 5 s holds are the clock's |
| `goal::a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list` | joint world, `goal-story` | `suite::task::a_chats_goal_shows_its_report_from_its_goals_row` | `a_finished_goals_page_shows_its_report` starts at the page, with no chat and no report note; the route is new |
| `sign_in::a_chatgpt_sign_in_on_the_fake_box_leaves_a_usable_model_and_ready_can_create` | joint world, `model-story` | `suite::connect_model::a_held_welcome_sign_in_moves_on_once_when_the_watch_makes_its_account` and its reloaded twin, through their shared `welcome_moves_on` | the browser-tab walk walks the same path to Ready; the one check only this one makes is the account read usable after the sign-in |
| `sign_in::a_chatgpt_sign_in_beside_a_browsers_idle_issuer_sockets_leaves_ready_able_to_create` | real loop, `issuer-sockets`; the fake's stories, `issuer-log` | `temper/tests/real-loop` `suite::sign_in::a_sign_in_beside_a_browsers_idle_issuer_sockets_completes_and_past_the_cap_is_turned_away`; `temper` `suite::fake::the_oauth_fake_logs_each_connection_it_turns_away_at_its_cap` | the fix has two halves (`8cba1c07a3`): the capacity is `temper-fake`'s, met by the real engine on real sockets in the real loop; the log line is `temper fake oauth`'s own, so it is the fake process's story beside its others. Each is red without its half |
| `note_parts::a_long_document_is_built_in_parts_on_the_fake_box` | real loop, `note-parts` (next batch) | `suite::notes::a_long_document_is_built_in_parts_and_revised_by_a_fresh_run` | the real loop's hosted agents have the real tools |
| `chat_document::a_chat_document_is_written_and_revised_on_the_fake_box` | real loop, `chat-document` (next batch) | `suite::notes::a_chats_document_is_revised_whole_after_a_follow_up` | not covered by `note_parts`: its follow-up `task message` and whole-body revision are paths `note_parts` never takes, so it goes beside it, short |
| `copy::readys_two_copies_are_adopted_in_order_with_the_first_as_home` | **one-process world**, not the joint world, `copy-ready` (next batch) | `temper/tests/one-process` `suite::ready_copies::readys_two_copies_are_made_in_order_and_adopted_with_the_first_as_home` | the joint world has no setup driver: its copies stop at Waiting, and making them there would inject `MakingMade`, a stub of what the walk proves. The one-process world runs the service's copies (`forge.md`, 4.5) and a native person who presses Ready |
| `copy::a_copy_is_made_and_adopted_through_ctl_on_the_fake_box` | **one-process world**, `copy-ctl` (next batch) | `suite::copy::a_copy_beside_a_repository_joins_its_project_keeping_the_sources_workflow` | not covered: 4.5's stories hold the copy's rows, act and recovery, but none a copy joining a project that holds a repository, its provider identity, `home` false on the finished resource, or the source's own workflow kept beside the starter |

None goes to the browser tier (`temper/docs/design/client/browser.md`,
section 4). A keep a lower world can't hold without a stub is its task's
`## Left`; `walks-go` then keeps that walk and writes why in the status.

## Parts and tasks

Batch 1, released with the design:

| Part | Task | Needs | Minutes |
|---|---|---|---|
| `status` | `status-stories` | — | 120 |
| `away` | `away-story` | — | 120 |
| `goal` | `goal-story` | — | 90 |
| `model` | `model-story` | — | 45 |
| `issuer` | `issuer-sockets` | — | 120 |
| `issuer` | `issuer-log` | — | 50 |

Next batch (being written): `notes` (`note-parts`, then `chat-document`),
`copies` (`copy-ctl`, then `copy-ready`), and `prune` (`walks-go`, after
every other task).

## Critical path, as it stands

`copy-ctl` (90) → `copy-ready` (150) → `walks-go` (60): about five hours
of implementer time; `note-parts` → `chat-document` runs beside it.

## Overlaps with other drafts

- `ui-ask-m3a/ways-stories` owns `temper/tests/client/src/**`, which
  `status-stories` and `away-story` may append one constructor to (in
  `front/status.rs` and `front/away.rs`): add, never reshape, and only
  where the suite file can't hold it.
- `walks-go` edits status files other drafts also edit
  (`temper/docs/design/client/README.status.md`,
  `temper/docs/design/domain/agent.status.md`): one line each, last in
  this plan.
- Nothing else of this plan's early work touches a file another draft is
  changing.
