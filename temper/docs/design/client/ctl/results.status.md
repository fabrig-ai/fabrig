# results: status

2026-11-11. Where the tree's code lags or departs from [results.md](results.md).

- **Not yet:** 1: a deletion's outcome prints `removed: project 3, fabrig`, not `deleted: project 3, fabrig`, and its kept form `kept: remove-project, removed: project 3` [in flight: change 634].

- **Not yet:** 3: the accepted `core-after-project` not-ready watch
  refusal and ctl deadline diagnostic are absent. Jig has no
  `WatchRefusal::NotReady`, and a shut service gate still sends `Busy`,
  so the client can claim all watches are in use. The sign-in's distinct
  line does not cover this path (`../../protocol/README.status.md`, 6.2).

Section 1's run-ended text and JSON retain the invalid-result cause.
The renderer tables distinguish each contract kind and unreadable result;
task help explains both. No known gap remains in this diagnostic.

`tools.md`, section 3 has no known implementation gap in settings files, the form,
apply and show carrying ordered charter parts and a template's later
choice. The decoder names malformed parts by their path before sending;
text show names each part's reasons and instruction bytes and the later
model and effort. The settings tables cover round trips and refusals,
and the generated field and ctl help pages carry both shapes.
`tools.md`, section 3 also has the stored-policy refusal's supplied bytes and
`root.journal.run_bytes` maximum, shared with a run's settings tool;
the JSON member is `stored_policy`. The refusal table holds charter 5,
supplied 70112 and maximum 65536. Malformed parts still name no part
in the engine's refusal, as the draft's status records.
`tools.md`, section 3 has no known gap in a goal template's work: `settings show
template-goal N [--revision R] [--json]` prints `design` and `close_out`,
or with none "none kept", a settings file's `template-goal` item is
read, proposed and applied, and a proposal card names it
`template-goal N` (`temper-client-tools`' tables
`forge_setting_commands_join_inventory_state_and_the_forges_value_in_text_and_json`
and `forge_setting_commands_print_removed_and_unset_values_and_preserve_refusals`;
`tests/client`'s `each_kind_through_ctl_proposed_shown_accepted_and_restored`).

`testing.md`, section 2's joined story of a settings file of each kind proposed,
shown, accepted and restored (`tests/client`'s `settings_tools.rs`,
`each_kind_through_ctl_proposed_shown_accepted_and_restored`) runs
through its restart: a project's rule 1 reset to the every-project
default keeps its retained state (`../../protocol/configuration/settings.status.md`, 2 and `../../protocol/configuration/stock.status.md`
).

`tools.md`, section 2, and `results.md`, section 1 have no known gap in next-run bounds explanations,
hello figures, charter-bearing shortfalls or their JSON. The waiting
tables check each named ctl call through the parser, each help page by
lookup and the program-login command against its configured prefix.
Host waits name no command. The settings reference's non-waiting help
pointer remains deferred in `README.md`, section 11.

Unlisted sections have no known gap, including 3's deadline words
apart from the not-ready watch path above, and `README.md`, 6.2's retry spans.
`README.md`, section 5's key refusals, 1's unpinned account wait and failure-log directions, `areas.md`'s
setup ending and `tools.md`, 3's effort check against `me` have no known gap.
`tools.md`, section 3, and `results.md`, section 2's publication words are in `project amend`'s help
and successful answer's stderr, as text or a `may_show` JSON object.
The help walk gives them before its command. Ctl asks nothing before
acting: an owner who skips the help can first meet them after publication.
The preceding acknowledgement remains deferred in `README.md`, section 11.

`README.md`, section 5 has no known gap in distinguishing same-key replay, a retained
decision's `decided first:` at exit 0, and an unknown proposal's exit 2
with "this proposal isn't waiting; it may have been decided or replaced."
The tools' settings table checks both accepted-decision words and the
unknown-proposal explanation; the generated help states the same limit.

- **Not yet:**  the groups `finding`, `qa`, `party` and
  `deployment`, absent from `temper-client-tools`' `table.rs` `GROUPS`.
   `repository receipts`, `finding list`, `finding show`, `qa marks`,
  `qa passes` and `hold list`.  `task move
  --under`, `task succeed --by`, `finding route`, `finding backlog`,
  `finding drop`, `qa choose`, `task hold` and `project hold`.
   area 2's procedure executor choice for `goal start`,
  `proposal withdraw`, `seen`, and `task amend`'s `--budget`; area 3's `party make`, `sign-in --service`, `deployment
  show`, `task move`, and the project's
  `policy`, `role` and `pool`; area 4's `create`, `propose` and
  `compound`; area 6's `forge forget`, with no request on the wire.
  1: a task wait's refusal by a worker's hello
  prints its stop bound as jig's nanoseconds, `allowed grace N ns`, not
  whole seconds (`render/document/llm.rs`, `HelloRefusal::StopBound`).
   `GROUP VERB --help`; `temper ctl` parses a
  leading `help` alone. 
  the joined story of a model's amendment (`tests/client`'s
  `amend_tools.rs` parses `--model` alone); the joined stories of
  `playground end`, `forge remove`, and a set up or a connect without
  FORGE, which `temper-client-tools`' own step tests hold alone; the joint component correlates
  forge setting reads, including historical revision values. 
  the real loop's connected Forgejo, its tokens connect, a replace of
  its git token and another user's tokens stopped.
- **Departs:**  `project` is area 1's (`jig/rows.rs`
  `PROJECT_SHOW`, `PROJECT_FUND`, `PROJECT_MAKE`), not area 3's; the placeholders add
  `HOST:PORT` and `URL`.  `settings show template|policy` takes
  `--revision` too; `landing show` takes `TASK [--project PROJECT]`, not
  `LANDING`; `landing list` takes no `--newer-than` or `--after`, and
  its `--ended` adds the ended to the live.  `task stop` places a
  hold over the task's subtree (`jig/rows.rs` `STOP`), as jig's `Stop`
  does (jig's `domain/people/acts.md`, 1), where its row holds the task;
  `hold lift` is keyed optionally (`Keyed::Optional`). 1: a party's wait
  prints `you: question of task 44 · 1 waiting`, with no question's number,
  where the section's sample and `temper help task` have `question 3`, and
  the `plainer-panels` request has that page say what the line prints: jig's
  `WaitsOn::Party` carries the asked task, its kind and how many wait, and no
  number (`render/document.rs`, `waiting`). A task's spent and budget print
  divided by 100, `spent 1.20 of 5.00` (`render/document.rs`, `amount`), as
  `temper help budget` says, where the section has them never scaled. A task
  pinned to an account that holds no source for this work waits on that account
  with the why `Empty` (jig's `select_source`), and prints `next run: account 43
  source unknown: empty`, then `no account serving this work yet`
  (`render/document/llm.rs`, `wait_explanation`), then `waiting on: Nothing
  awaited`: the `wait-words` request of 2026-11-06 has `waiting on:` name an
  account when the next run has none to run on. Its plan held "no account" to
  the wait that printed the words the request drops, to keep this one's number,
  so these lines can still say opposite things (`README.md`, section 11). A person task's
  `task amend` prints `task amend 41 --key KEY`, where that request has the act
  and its arguments, as an agent task's `--rev 8 CHANGE... REASON` does:
  `person_call` (`render/document.rs`) gives an answer alone its arguments.  an
  authority's entries are 14 (`limits.rs`).
