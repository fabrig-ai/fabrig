+++
id = "agent-stories"
part = "configuring-agent"
kind = "feature"
needs = ["stock-configuring", "agent-kinds"]
files = ["temper/tests/engine/domain/tests/suite/configuring.rs", "temper/tests/engine/domain/tests/suite/configuring/**", "temper/tests/engine/domain/tests/suite/main.rs", "temper/tests/engine/domain/src/host_tool.rs", "temper/tests/engine/domain/src/lib.rs", "temper/tests/engine/domain/tests/suite/settings.rs", "temper/tests/engine/domain/tests/suite/agent_reads.rs"]
effort = "high"
minutes = 150
+++

# agent-stories: the configuring agent's flows in the engine's domain world

## Goal

temper's `domain/testing.md`, 6.4's row "the engine's domain world, the configuring agent":
scripted runs of stock charter 3 (template 3) through the root and the core on jig's store,
each flow of temper's `domain/agent.md`, 8.1 that the engine decides. The story plays the
run's host calls, as `suite/settings.rs`' `host_tool` does; a fake model says only what it is
scripted to, so the stories hold the mechanisms, never the agent's judgement.

Acceptance, one story each, in `temper/tests/engine/domain/tests/suite/configuring.rs`
(submodules under `configuring/` where it passes about 600 lines), on the stock's own
defaults (`stock::Given::Stock`, or the stock's defaults as the world's own):

- `an_owners_configuring_chat_proposes_and_hears_it_accepted`: an owner's chat from template
  3 runs under charter 3 and is offered `settings` and `propose_settings`; it reads the
  inventory and a charter, proposes against the newest revision with a reason whose lines
  carry "temper enforces" and "your agents follow"; the owner accepts, running work staying;
  the chat's next run hears `accepted: revision 2 made`.
- `a_members_proposal_waits_for_an_owner`: a member (a role in the project, not the
  deployment's owner) starts the configuring chat; its proposal is answered as proposed to the
  deployment's owners, waits in an owner's inbox, and nothing changes until the owner decides.
- `a_stale_proposal_is_withdrawn_read_again_and_proposed_afresh`: after the run proposes, an
  owner's own change is accepted; the proposal is withdrawn as stale and the run hears it with
  the newest revision; it reads the settings again and its fresh proposal against the newest is
  accepted.
- `a_rejected_proposal_is_revised`: the owner rejects with a reason; the run hears
  `rejected: <reason>`, and its revised proposal is accepted.
- `an_accepted_proposal_moves_running_work`: a task running under revision 1 (a chat of
  template 1 parked mid-work) moves to the new revision at its next run when the owner accepts
  with running work moving; a charter the change removed leaves its task where it is.
- `the_configuring_agent_reads_each_kind`: `settings` of the inventory, a charter, a template,
  a policy, the rules, a landing rule, the forge's charter, a held charter at an older
  revision, a revision's record and a history page, each answered within the answer's bound
  (`host_reply_bytes`; a long one goes on from `from`).
- `propose_settings_refuses_a_stale_revision_and_an_item_out_of_shape`: against an older
  revision, answered stale naming the newest; an item out of shape (an unknown member, a
  charter with no `instructions`), answered as an error naming it; neither commits anything.
- The story `a_run_without_the_settings_family_is_denied` (`suite/settings.rs`) stays, and a
  new one, `a_chat_is_offered_no_settings_tools`, holds that a stock chat (charter 1) is not
  offered them although policy 1 holds the family.
- The world's stories together stay about 0.5 s (6.4's row): measure the new file's wall time
  in the focused profile and report it.
- The focused tests of the engine's domain world pass, all of them.

## Inputs

- `temper/docs/design/domain/testing.md`, 6.4 (the row) and 6.1 (the world's budget).
- `temper/docs/design/domain/agent.md`, 4.3 and 8.1.
- `jig/docs/design/domain/engine.md`, 17.3, 17.4 (moving running work), 17.6 (a proposal of
  settings and its outcomes).
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: the files in this spec's `files`; in `suite/main.rs` the `mod configuring;` line.
- Options, pre-approved:
  - Move `suite/settings.rs`' `host_tool` loop (and `agent_reads.rs`' copy) into the world's
    library as `temper_engine_domain_world::host_tool` (`src/host_tool.rs`), the two files then
    calling it; nothing else of them changes.
  - If the member's proposal cannot be made because no stock role lets a member start a chat
    of template 3, give the story's project a policy of its own (a world's defaults, as
    `settings.rs`' stories do) with a member's role that requests it, and say so.
  - If moving running work at acceptance needs a revision the stock's settings do not have
    (a charter the change removes), make the change remove a world-own charter, and say so.
  - A story past about 0.1 s alone: find why before adding to it; report the figure.

## Steps

1. The helper, if moved.
2. The stories, one at a time, each run alone (`heavy cargo nextest run -p
   temper-engine-domain-world --test <the suite binary> -E 'test(=configuring::NAME)'`,
   names as the workspace has them) before the next.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-world --all-targets -- -D warnings`; the world's focused tests.
4. Commit ("the configuring agent's flows in the engine's domain world (testing.md, 6.4)")
   and push.

## Report

As the implementer's text says, and each story's wall time.
