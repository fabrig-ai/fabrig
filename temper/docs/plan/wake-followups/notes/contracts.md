# wake-followups: the shapes the tasks build to

The run's binding calls. A name, member, figure or word here is spelled as given.

## 1. A charter's stored policy in jig's refusal detail (task `size-refusal`)

The answer to `per-wake-temper`'s question `charter-size-refusal`: a member added in place,
no version bump, the refusal kinds kept (jig's `jig/docs/design/domain/engine.md`, 17.9;
`temper/docs/design/domain/root.md`, section 10).

- **The member:** `SettingMember::StoredPolicy`, appended after `Connector` in every copy:
  `jig-domain-people` (`src/settings.rs`), `jig-client-domain` (`src/contract/settings.rs`),
  `temper-settings-form` (`src/view.rs`); and `stored_policy`, appended last to
  `enum SettingMember` in `jig/crates/jig-client-protocol-wire/schema/client-v19-part1.schema`
  and `jig/crates/jig-protocol-store/schema/people-v14.schema`, their code and goldens
  regenerated. The existing members keep their tags 0 to 5.
- **The facts:** `SettingDetail { member: StoredPolicy, document: 0, segment: 0, rule:
  SettingRule::Maximum, supplied, bound }`, where `supplied` is what the core's size check
  measures (`policy_owned_bytes` of the charter's policy plus `parts_owned_bytes` of its
  parts, `jig-domain-core/src/limits.rs`), `u32::MAX` where that passes `u32` or doesn't
  measure, and `bound` is `Limits::run_bytes`.
- **The refusal:** the charter's item out of shape as today (`Fault::Shape`, the charter's
  setting, the item's position), its `detail` now `Some` of those facts, wherever the
  core's check refuses a charter for its stored size on a change of settings or a proposal.
- **The words** (`temper-settings-form`, `render::setting_detail`, shared by ctl and a run's
  settings tool): for `StoredPolicy`, `; stored policy: supplied <supplied>; maximum
  root.journal.run_bytes <bound>`, with no `; documents` before it. So ctl prints
  `settings apply: out of shape; item 1; charter 5: shape; stored policy: supplied 70112; maximum root.journal.run_bytes 65536`.
  The other members' words stay as they are.
- **The client's JSON** (`temper-client-tools`, `render/document/me.rs`): the member
  `"stored_policy"`, the rule `"maximum"`, the other fields as for any detail.

## 2. A task read refused as unknown (task `task-read`)

`temper/docs/design/protocol/llm.md`, section 4. jig refuses a run's `read_task` as
`Refusal::Unknown` for a task it has no row of, an offset outside the text, and a `result`
no ended task kept, a running task's among them (`jig-domain-core/src/reads/run_task.rs`,
`value` and `loaded`). jig stays as it is; temper's words for that refusal of that call
alone (`temper-engine-smith/src/answers.rs`, `task_read`) are, exactly:

```
Task read refused: unknown: no such task, no result yet, or no text at that offset. A task has a result only once it has ended, and its requester hears it then as a result message; while it runs, read its specification (leave out field) for its state, holds and questions.
```

Every other refusal of `read_task`, and `unknown` in every other answer, keeps its words.

## 3. A host call's backoff (task `busy`)

`smith/docs/design/domain/run.md`, 5.2. In `smith-domain-run`'s relay
(`src/run/relay.rs`, `host_recover`), the wait before relay attempt `n + 1`, after attempt
`n` was answered busy or went unanswered, is `host_backoff` times `2^(n - 1)`, at most `64`
times `host_backoff`: with the standard 50 ms, 50, 100, 200, 400, 800, 1600, then 3200 ms
for every later ask. An ask whose start would be at or past the caller's deadline is not
made and the call returns as today (`Returned::Busy`, or `Returned::HostUnknown` once an
ask went unanswered). `host_backoff` keeps its value and meaning (the first wait); no
limit is added; the cap `64` is a constant of the relay, named
`HOST_BACKOFF_GROWTH`.

## 4. A charter's selected effort (task `foreman`)

jig's `jig/docs/design/domain/tasks.md`, section 3, and `engine.md`, 17.11. In
`jig-domain-core/src/settings/charters.rs`, `resolution`, the record a model the charter
names resolves to is the catalogue row with its `effort` replaced by the charter's model's
`effort` where that is `Some(e)` and the row's `efforts` contains `e`; otherwise the row
whole, as today. A model that follows by `rest` takes the row whole. Nothing else in the
resolution changes: its order, its omissions, its `run_bytes` fit (measured on the record
it keeps). So a task given no choice takes, at each run's check, the effort its charter
selects (`runs/llm.rs`, `defaults`, already reads the resolved policy's model), and a
given choice keeps its own.

## 5. Charter 5's cut (task `subscribe`)

`temper/docs/design/protocol/configuration.md`, 13.9, the cut. In
`temper/docs/examples/reference/settings.json`, charter 5's part `on` `["first"]`, the
sentence

```
On your first run, once your first batch is made, subscribe with `subscribe_forge` to the landings on main in your repository.
```

is replaced, in place, by

```
The core subscribes you to the landings on each branch your changes target as the first change into it is made; don't subscribe to them with `subscribe_forge`, which refuses a duplicate.
```

and the sentence after it (`The cut never waits for or reacts to its changes.`) stays.
No other charter text changes, and no figure (`forge.paths_per_subscription` stays 4).
