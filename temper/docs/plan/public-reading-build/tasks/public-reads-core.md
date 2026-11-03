+++
id = "public-reads-core"
part = "jig-public"
kind = "feature"
needs = ["public-index"]
files = ["jig/crates/jig-domain-core/src/public/reads.rs", "jig/crates/jig-domain-core/src/public/reads/**", "jig/crates/jig-domain-core/src/tests/public/reads.rs", "jig/crates/jig-domain-core/src/tests/public/reads/**", "jig/testing/jig-test-domain/src/admission.rs", "jig/testing/jig-test-domain/src/client_documents.rs", "jig/testing/jig-test-domain/src/routes.rs", "jig/testing/jig-test-domain/src/tests/public.rs"]
effort = "high"
minutes = 150
+++

# The core answers public reads from the index; jig's root admits them behind signed-in work

## Goal

A public read admitted by the root is decided by the core on the public index alone: jig's
three documents projected from it, an application's asked of its connector with the project's
public tasks, every answer rechecked in the step that completes it and given once durable,
and every miss one answer by one path (`jig/docs/design/domain/engine.md`, section 11;
`jig/docs/design/domain/people.md`, 13.5, 13.7, 13.9 and 13.10;
`jig/docs/design/domain/root.md`, sections 3 and 10).

Acceptance:

- `src/public/reads.rs` replaces `public-vocabulary`'s "every public read unknown":
  `Event::PublicRead { reply, read }` is decided in this order, the same work whatever the
  identifiers name:
  1. the project (`read.project`, `u32::try_from` it, a failed conversion taking the miss's
     path) looked up once, `public_index.project()`: `None` -> `Unknown`; `Busy` -> `Busy`;
  2. the task, if the kind takes one, `goal_of(task)`: `None` -> `Unknown`;
  3. owner 0 (jig): kind 1 *projects* (no ids; the public projects whose entries are
     `Ready`, ascending), 2 *board*, 3 *card*, any other kind `Unknown`, each projected from
     the index and `public/line.rs` alone (no `Request::Load`, no store read, no scan of the
     live tasks), within the bounds and folded by rule (contract.md, section 12's table: live
     goals by priority then ended most recent first; a goal's tree in tree order; a card's
     lineage nearest first ending at its public goal, its public delegates and their folded
     count);
  4. owner `n > 0`: `Request::Ask { connector: n - 1, ask: Ask::Public { reply, kind, version,
     project, task, public } }`, `public` the project's `public_tasks()`, ascending; a
     connector the core does not know -> `Unknown`. By the time the core sees it, `owner` names
     a connector, not a family: the root maps a family's owner to the connector that serves it,
     as `Work::ReadApplication` takes its `number` (`jig-test-domain/src/lib.rs`), and hands
     the core `owner` = that connector's number plus one (`notes/contracts.md`, section 4); a
     family no connector serves -> the root answers `Unknown` itself, after the same admission.
- **Completion (13.10):** each admitted read waits in the core (a bounded table of its own)
  until it is durable: after its document is made (jig's) or its connector's
  `Event::PublicAnswered` arrives (an application's: `None` -> `Unknown`; a `tasks` entry not
  among the `public` it was handed -> `Unknown`), the core waits for every earlier commit to
  be durable, then, in the step that completes it, checks again that the project is public
  and not being deleted (`Core::public_project`) and that every task the answer names is
  still public (`goal_of`), and answers `Held::PublicRead { reply, answer }`, `Unknown` if any
  check fails, else `PublicAnswer::Document { decided, document }` with `decided` that step's
  `env.now`. A read commits, marks and loads nothing, and opens no watch.
- **jig's root** (`jig/testing/jig-test-domain`): `Event::PublicRead` is new work admitted
  only while the backlog is under `limits.public_watermark` and the core holds fewer than
  `limits.public_reads` public reads (the root counts those it handed in less those whose
  `Held::PublicRead` it released); otherwise it is handed back to the clients' component at
  once as `Held::PublicRead { reply, answer: PublicAnswer::Busy }`'s answer, never queued;
  `Ask::Public` reaches the numbered test connector and its `Event::PublicAnswered` returns to
  the core (the test connectors answer `None`, as `public-vocabulary` made them).
- Unit tests in `jig-domain-core`'s `src/tests/public/reads.rs` (a module line in
  `src/tests/public.rs`), every output of each step compared whole:
  - `a_board_and_a_card_are_projected_from_the_index_alone`: the expected records built
    independently from contract.md 12's source table; no load, write or mark leaves the core;
  - `a_private_a_deleting_and_a_never_made_project_answer_alike` and
    `a_card_of_a_task_that_is_not_public_answers_as_a_missing_project` (a conversation, a
    task past the bounds, another project's task, a number never made, an identifier past
    `u32`): each pair's outputs identical but for the reply, and the same requests out;
  - `projects_lists_the_public_projects_alone`;
  - `an_application_read_asks_its_connector_with_the_public_tasks_ascending`;
  - `an_application_answer_naming_a_task_not_handed_is_unknown`,
    `a_connector_answering_none_is_unknown`;
  - the races of 13.6, each both ways round (the change decided before the read is admitted,
    and while it waits): `made_private_while_under_way_ends_unknown`,
    `deletion_admitted_while_under_way_ends_unknown`,
    `untracked_while_under_way_ends_unknown`, `moved_out_while_under_way_ends_unknown`, each
    once while the connector answers and once while waiting to be durable;
  - `a_read_of_a_project_not_yet_built_is_busy`;
  - `decided_is_the_completing_steps_time`;
  - `a_public_read_commits_and_marks_nothing`.
- Root tests in `jig-test-domain`'s `src/tests/public.rs` (a module line in `src/tests.rs`):
  `a_public_read_at_the_public_watermark_is_busy_and_a_signed_in_read_is_admitted`,
  `the_core_holds_at_most_public_reads`, `the_count_falls_as_answers_are_released`,
  `an_application_kind_reaches_the_test_connector_and_back`.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, sections 1
  and 4.
- `jig/docs/design/domain/engine.md`, section 11 (public reads); `jig/docs/design/domain/root.md`,
  sections 3 ("Public reads come behind") and 10 (the public watermark);
  `jig/docs/design/domain/people.md`, 13.5, 13.6 (its races), 13.7, 13.9, 13.10;
  `jig/docs/design/client/domain/contract.md`, section 12.
- As they stand: `public-index`'s module and report; `public-vocabulary`'s unknown answer and
  its tests; `jig-domain-core/src/reads.rs` (how a read's answer is held until durable);
  `jig-test-domain/src/admission.rs` (`try_step`, `backlog`, `admits`), `routes.rs`,
  `client_documents.rs`.

## Owns and may change

- Owns the files in the header.
- `jig-domain-core`: `src/public.rs` (the module line), `src/tests/public.rs` (the module
  line), `src/lib.rs` (the `Core` field for the pending table), and the site where
  `public-vocabulary` answers a public read unknown and ignores an unasked answer (replaced by
  calls into `public/reads.rs`).
- `jig/testing/jig-test-domain/src/lib.rs` and `src/tests.rs`: the root's count field and a
  module line, nothing else.
- **The durability wait:** if the core has no output that returns to it once every earlier
  commit is durable, add the pair `Held::PublicWait { read: Token }` and
  `Event::PublicDurable { read: Token }`, appended last in `vocabulary.rs`, the root stepping
  the event back as it steps a store answer (not new work); its forced arms in jig's root,
  `jig-test-service`, `jig/examples/ops/domain`, `temper-engine-domain` and
  `temper/tests/one-process` are allowed, each only for the arm (temper's root routes it the
  same way: a required arm). Say which you used.
- The pending table's bound: the core's figures as they stand; if none bounds it, size it by
  the figure that bounds the core's other reads in flight and say which.
- `temper-engine-domain`'s root admission is `forge-public`'s, not yours.

## Steps

1. `public/reads.rs`: the order of lookups, jig's projections, the application's ask; the
   pending table and completion; `heavy cargo check -p jig-domain-core --tests`.
2. jig's root: admission and routing; `heavy cargo check -p jig-test-domain -p jig-test-service --tests`.
3. The unit tests above, each run alone first
   (`heavy cargo nextest run -p jig-domain-core --lib -E 'test(/public::reads::/)'`,
   `heavy cargo nextest run -p jig-test-domain --lib -E 'test(/public/)'`).
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` (and temper's if you touched it);
   clippy on every package you changed; the focused tests of `jig-domain-core`,
   `jig-test-domain`, `jig-test-service`, `jig-core-world`; `heavy cargo check -p temper-engine-domain --tests`;
   the citations check.
5. Commit (why: public reads decided on the index alone, rechecked at completion, admitted
   behind signed-in work); push your branch.

## Report

As the implementer's text says, and also: the durability wait you used, the pending table's
bound, and for each paired test the outputs compared.

## Answers

2026-11-03. **The user's decision on public titles.** User, 2026-11-03 15:40Z: public reading shows titles. A goal's and a task's title (the first line of its spec, as jig's client contract names a task's title) crosses into its public line, and a public project's name crosses into its public record. There is no author flag and no human publication capability for these. Everything else in people.md 13.3's "No text at all" list still never crosses: spec beyond its first line, instructions, inputs, results, words, questions and answers, transcripts, notes, reasons, labels' meanings, proposals, escalations, people's and parties' names, charters', procedures' and roles' names. Review finding 1 is settled by the user's decision.

For this task: Fill the title and the name from their sources. A spec amendment changes the title, so the public line's last-changed covers it; the recheck after asynchronous work reads the current title.

2026-11-03. **A public project's name in the wire** (answered for `public-wire`): jig's
*projects* answer is a list of `PublicProject { project, name }`, and `PublicBoard` carries
`name` after `project`, each the project's name within the client wire's project-name bound
(64 bytes).
