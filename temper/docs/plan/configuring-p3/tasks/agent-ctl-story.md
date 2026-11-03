+++
id = "agent-ctl-story"
part = "configuring-agent"
kind = "feature"
needs = ["stock-configuring", "agent-kinds"]
files = ["temper/tests/client/tests/suite/configuring_chat.rs", "temper/tests/client/tests/suite/main.rs", "temper/tests/client/src/front.rs", "temper/tests/client/src/fixtures.rs"]
effort = "high"
minutes = 120
+++

# agent-ctl-story: a configuring chat through temper ctl, in the joined client world

## Goal

The story the walk repeats on a real model (temper's `domain/agent.md`, 8.1, "Started from
ctl"; `domain/testing.md`, 6.4's row "the client and the engine, joined", its ctl half; the
pages are the web client's, not built): ctl against the engine root, the run's host calls
scripted by the world, as the world plays its runs.

Acceptance, in `temper/tests/client/tests/suite/configuring_chat.rs`:

- `a_configuring_chat_started_through_ctl_proposes_and_an_owner_accepts`: an owner runs
  `chat start --project 1 --template 3 TEXT`; the chat's task runs under charter 3; the world
  plays its run reading `settings` and proposing a small change (charter 1's `waiting`, for
  one) with a reason under "temper enforces" and "your agents follow"; `inbox --kind
  proposals` lists it; `proposal show PROPOSAL` prints its values now and proposed, the
  revision it was made against, its reason and its effect on running work; `proposal decide
  PROPOSAL accept --running stays` accepts it; `chat show TASK` shows the run's line naming the
  revision after it hears `accepted: revision 2 made`; `settings show` names revision 2.
- `a_members_configuring_proposal_waits_for_an_owner`: the same chat started by a member;
  `proposal show` says it waits for an owner, and the member's `proposal decide` is refused.
- `a_stale_configuring_proposal_is_withdrawn`: an owner applies a settings file
  (`settings propose ... --against 1`, accepted) after the run proposed; `proposal show` of
  the run's proposal says it was withdrawn as stale.
- ctl's stdin is `</dev/null` as the world's other ctl stories give it; the three stories
  together about 0.4 s (6.4's row), measured and reported.
- The focused tests of the client world pass, all of them.

## Inputs

- `temper/docs/design/domain/agent.md`, 8.1.
- `temper/docs/design/client/tools.md`, 5.3 (`chat start --template`, `proposal show`,
  `proposal decide`, `inbox --kind proposals`, `settings show`).
- `temper/docs/design/domain/testing.md`, 6.3 and 6.4.
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: the files in this spec's `files`; in `suite/main.rs` the `mod configuring_chat;`
  line; in `src/front.rs` and `src/fixtures.rs`, what the stories need added (a scripted host
  call from a run, as the engine's domain world's `host_tool` does it; a world that offers
  charter 3 and template 3), no existing story's behaviour changed.
- Options, pre-approved:
  - The world builds its configuration from the engine's domain world's fixtures, not the
    stock: give these stories' world the stock's charter 3, template 3 and the settings family
    in its rules and policy (from `temper_service::stock::defaults`), raising its own
    `tasks.charters` and `core.templates` for them alone.
  - If `proposal show`'s text for a run's proposal lacks a part the acceptance names, assert
    what it prints, and list the gap in the report (ctl is not this task's).
  - If the member cannot start a chat of template 3 under the world's policy, give its project
    a role that requests it, and say so.

## Steps

1. The world's additions.
2. The stories, each run alone first (`heavy cargo nextest run -p temper-client-world --test
   <the suite binary> -E 'test(=configuring_chat::NAME)'`, names as the workspace has them).
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-world --all-targets -- -D warnings`; the world's focused tests.
4. Commit ("a configuring chat through temper ctl, in the joined client world (agent.md,
   8.1)") and push.

## Report

As the implementer's text says, and each story's wall time.
