# ui-drafter: the contracts

The run's binding calls, for every task of this plan. Each is built by the task
named, to these shapes, and read by the others as written here.

## Numbers

- **Charter 12, the drafter's; template 6, a design session**
  (`temper/docs/design/protocol/configuration.md`, 13.4;
  `temper/docs/design/domain/agent.md`, 8.5).

## The charter (`drafter-stock`, `temper/crates/temper-service/src/stock.rs`)

- **Its instructions:** `include_bytes!("stock/drafter.txt")`, the file a byte copy
  of `temper/docs/plan/ui-drafter/inputs/drafter.txt` (`cp`, then `cmp`): 2,874
  bytes, 13 lines, ASCII, ending in a newline, sha256
  `1f447743f67f321bac9516e472173e9f88e05b4067c4e1c7bec3d1457c5ed900`.
- **Its policy:** `charter(12, CHAT_TURNS, CHAT_TIME, <instructions>, models,
  FIRST_CHOICES)`, then `inspect`, `modify` and `shell` false and `agents` true;
  the rest is charter 1's: hosts `Workers`, waiting 10 s, resume, 64 turns and
  3,600 s, a reserve of 1 turn and 60 s, a call timeout of 10 s, the default wake,
  no parts, no documents, a spend of K.
- **Its tools:** a constant `DRAFTER_TOOLS: jig_domain_tasks::Tools =
  jig_domain_tasks::Tools(1022)`: every family a chat's names but delegate (1);
  so no delegate, settings (1,024) or asking (2,048).
- **Its place:** a function `drafter(&models)` beside `ask_charter`, called after
  it in `checked_with`'s charters, so the stock's charter numbers read
  `[1, 2, 4, 5, 6, 9, 10, 11, 12]`.

## The template (`drafter-stock`)

- `Template { number: 6, name: b"design session", executor:
  TemplateExecutor::Charter(12), ..chat(result_bytes) }`, a function `design(..)`
  beside `ask`, after it in `checked_with`'s templates: `[1, 2, 4, 5, 6]`.

## The delegation (`drafter-stock`)

- `authority::Executor::Charter(12)` right after `Charter(11)` in `ceiling`'s
  kinds, so the rules' ceiling, policy 1's ceiling and each role with spend name
  it: fifteen kinds, of the box's sixteen.

## The declarations (`delegate-held`, `temper/crates/temper-engine-smith/src/calls.rs`)

- `tools_for(timeout, charter, authority)` declares `goal`, `change` (its plan
  form included) and `delegate` only where `charter.0 &
  ToolFamilies::JIG.delegate.0 != 0`, whatever `authority` holds. The asking
  branch, the other families' gates and `tools(timeout)`, the whole registry,
  stay as they are.

## The workspace (`no-workspace`, `temper/crates/temper-engine-domain`)

- In `src/boundary.rs`, beside `asking`, exported as it is:

  ```rust
  /// A run of this policy is given no repository: an asking charter's, or one that
  /// names none of inspect, modify and shell (temper/docs/design/domain/agent.md, 4.2).
  #[must_use]
  pub fn no_workspace(policy: &RunPolicy) -> bool
  ```

  `asking(policy.tools) || !(policy.inspect || policy.modify || policy.shell)`.
- `forge_route.rs`'s `run_workspace` calls `crate::no_workspace(&charter.policy)`
  where it calls `crate::asking(charter.policy.tools)`; nothing else there moves.

## The client (`design-session-web`, `temper/crates/temper-client-domain/src/pages/chats.rs`)

- `pub const DRAFTING_TEMPLATE: u32 = 6;` beside `ASKING_TEMPLATE`, its doc line
  citing `temper/docs/design/protocol/configuration.md, 13.4`.
- Beside `asking`, as it is over its template:

  ```rust
  /// A design session: a chat of template 6's charter (temper/docs/design/client/README.md, 3.5).
  #[must_use]
  pub fn drafting(me: &Me, project: u32, executor: &c::ChipExecutor) -> bool
  ```
- `Chats::start(.., draft: true, ..)` sends `DRAFTING_TEMPLATE` where
  `jig_client::shapes::template(me, project, DRAFTING_TEMPLATE)` is some, and
  otherwise `jig_client::shapes::CHAT`, as today; its words are today's either
  way, the person's, a blank line and `WRITE_THIS_UP`, so that the fake model's
  scripted draft (`temper-fake/src/backends.rs`, its cue "Write this up as a
  draft beside us.") and the journey `chat-draft` hold. Either opens in
  `Mode::Split`.
- The terminal's `/draft` (`temper/docs/design/draft/surfaces.md`, 11.2, the TUI
  owner's T2) starts from these two names.
