# tools: status

2026-11-02. Where temper's code lags or departs from [tools.md](tools.md).

- **Not yet:** 5.1: the groups `goal`, `finding`, `qa`, `party` and
  `deployment`, absent from `temper-client-tools`' `table.rs` `GROUPS`.
  5.2: `repository receipts`, `finding list`, `finding show`, `qa marks`,
  `qa passes`, `hold list`, and `settings show landing-rule` and
  `forge-charter` (`jig/rows.rs` `SETTING` reads charters, templates,
  policies and rules alone). 5.3: `chat start`'s `--template`; `task move
  --under`, `task succeed --by`, `finding route`, `finding backlog`,
  `finding drop`, `qa choose`, `task hold`, `project hold` and `revision
  restore`; a settings file's items of temper's kinds and a proposal's
  (`ProposalItems`). 5.5: area 2's `goal start` with its model flags,
  `proposal withdraw`, `seen`, and `task amend`'s `--budget` and
  `--tasks`; area 3's `party make`, `sign-in --service`, `deployment
  show`, `task move`, `task take`, `task hand-back`, and the project's
  `policy`, `role` and `pool`; area 4's `create`, `propose` and
  `compound`; area 6's `forge replace` and `forge forget`, with no
  request on the wire. 6.1: a task wait's refusal by a worker's hello
  prints its stop bound as jig's nanoseconds, `allowed grace N ns`, not
  whole seconds (`render/document/llm.rs`, `HelloRefusal::StopBound`).
  5.4, 5.5: `change show`'s `cleared` condition, `landed` met by the
  steps after it, and its `--until cleared` following call.
  8.1: `GROUP VERB --help`; `temper ctl` parses a
  leading `help` alone. 11.2:
  the joined story of a model's amendment (`tests/client`'s
  `amend_tools.rs` parses `--model` alone); the joined stories of
  `playground end`, `forge remove`, and a set up or a connect without
  FORGE, which `temper-client-tools`' own step tests hold alone.
- **Departs:** 5.1: `project` is area 1's (`jig/rows.rs`
  `PROJECT_SHOW`, `PROJECT_FUND`), not area 3's; the placeholders add
  `HOST:PORT` and `URL`. 5.2: `settings show template|policy` takes
  `--revision` too; `landing show` takes `TASK [--project PROJECT]`, not
  `LANDING`; `landing list` takes no `--newer-than` or `--after`, and
  its `--ended` adds the ended to the live. 5.3: `task stop` places a
  hold over the task's subtree (`jig/rows.rs` `STOP`), as jig's `Stop`
  does (jig's `domain/people.md`, 5.1), where its row holds the task;
  `hold lift` is keyed optionally (`Keyed::Optional`). 12: an authority's
  entries are 14 (`limits.rs`), and the client's `projects` bound, 2, is
  not in the table. 5.5: `forge setup` refused "not under the box"
  prints the four `temper forge` commands and a restart
  (`temper-client-domain`'s `forge/words.rs` `set_up_commands`), not
  that temper starts with `temper box`, though those commands find no
  group's file for a forge the box did not run
  (`../protocol/setup.md`, 7.4): words for an owned forge set up by
  hand, which the box's group files leave without a way.
