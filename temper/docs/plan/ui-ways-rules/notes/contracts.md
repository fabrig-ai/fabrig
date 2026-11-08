# ui-ways-rules: contracts

The run's binding calls for the shapes that cross tasks. Design:
`temper/docs/design/protocol/configuration.md`, 13.2 to 13.4 and 13.9;
`temper/docs/design/domain/forge.md`, 8.9. Paths are from the repository
root; `F` is `temper/crates/temper-engine-domain-forge/src`, `R` is
`temper/crates/temper-engine-domain/src`, `S` is
`temper/crates/temper-service/src`.

## 1. A pattern of kind 2 may begin at `branch` (13.2, 13.3)

- **Judged:** `settings::judge` (`F/settings.rs`) admits a rule whose
  first segment is `forge`, or `branch` for kind 2 alone. Kind 1's first
  segment stays `forge`; `Refusal::Pattern` and its words are unchanged.
- **Matched:** one function decides whether a pattern covers a branch of a
  repository, and every reader of held rules goes through it:

  ```rust
  // F/gates.rs, re-exported at the crate root
  /// Whether `pattern` covers `branch` of `repository`: a pattern beginning at
  /// `branch` is judged against the branch's resource path past its
  /// repository's four segments (protocol/configuration.md, 13.2).
  #[must_use]
  pub fn covers_branch(pattern: &GatePattern, repository: &Repository, branch: &[u8]) -> bool;
  ```

  The resource path is `branch_resource_name`'s, `["forge", host, owner,
  name, "branch", part, …]`; a pattern beginning at `branch` is matched
  against its tail from `"branch"` on, by the same exact or open rule as
  `covers` today. A branch with no resource name (an empty part) is
  covered by nothing, as today. `held_requires`, `held_gates`,
  `judge_held_landing` and `panel_approvals` (`F/standing.rs`) and
  `settings::requires` match through it, or through a crate-private
  helper it shares.
- **Covered repositories:** the root's `covers_repository`
  (`R/route/kinds.rs`) answers true for a rule whose first segment is
  `branch`: it covers every repository of its project, and its callers
  already take the project's writable repositories alone.

## 2. Every project's rule, given to the forge (13.3, 13.4)

- **Given:** a `settings::Stock` entry with `kind: 2` and `scope: 0` is
  every project's rule of its `number`, given at each start in the same
  list as the forge's other defaults (`Domain::give_settings_defaults`).
  The forge answers no `DefaultState` for it, so jig hears nothing of it,
  and holds it in no scope's held rules of its own.
- **Read:** `Domain::setting(2, P, N)`, for a project `P` (not 0), answers,
  in order: a stored `Set` change's value; with no change stored, the
  scope's own default, as today; else, with no change stored or a stored
  `Remove` or `Reset`, the every-project rule `N` where one is given;
  else none. Its signature is unchanged. `judge_settings` judges a reset
  against the same value. The client's read (`R/route/kinds.rs`) and a
  run's read take the forge's answer.
- **Held:** a merge into a repository of project `P` meets every rule the
  forge holds for it:

  ```rust
  // F/kinds.rs: replaces `-> (&[Held], &[Held])`
  /// Every rule a merge into a repository of `project` meets: the deployment's,
  /// then the project's by number, its rule N the every-project rule N while
  /// the project has no value of its own set (protocol/configuration.md, 13.4).
  #[must_use]
  pub fn landing_rules(&self, project: u32) -> Box<[settings::Held]>;
  ```

  An every-project rule held for `P` is a `Held { scope: P, number: N, .. }`.
  `keep_settings` wakes the live changes of a project whose rules this
  answer changes, as it does for a project's held rules today.
- **A run's `settings` read** (`R/route/host_route.rs`, `settings`): where
  jig answers `Unknown` for kind 2, number `N`, of an existing project at
  the newest revision, the root answers as it does for kinds 6 and 7, its
  application value the forge's `setting(2, scope, N)` where that is some.
- **A deleted project** keeps nothing: its rows are erased as today, and
  no held entry is made for it.

## 3. Charter 5's first edition and template 4 (13.4)

- **The words:** `S/stock/coordinator.md`, a byte-for-byte copy of
  `temper/docs/plan/ui-ways-rules/inputs/charter5.md`. No task edits its
  words. If a stock bound or test refuses any of it, the task stops with
  `Question:`, naming the bound and the bytes.
- **Read at each start,** `include_bytes!`, split at its level-2
  headings: `## shared` is charter 5's `policy.instructions`; each
  `## part, on: NAME, NAME, …` is one `jig_domain_core::CharterPart`, in
  the file's order, `on` the union of jig's `tasks::Reasons` the names
  give (`first` `FIRST`, `question` `QUESTION`, `question_answered`
  `QUESTION_ANSWERED`, `question_ended` `QUESTION_ENDED`,
  `question_owed` `QUESTION_OWED`, `escalation` `ESCALATION`, `proposal`
  `PROPOSAL`, `person_stalled` `PERSON_STALLED`, `result` `RESULT`,
  `notice` `NOTICE`, `nothing_awaited` `NOTHING_AWAITED`, `news` `NEWS`,
  `timer` `TIMER`, `words` `WORDS`). A section's text is every byte from
  the line after its heading to the line before the next heading, or the
  file's end, with the line breaks that open and close it removed and
  nothing else. An unknown name or heading panics with the stock's
  `expect`, which the stock's test holds.
- **Charter 5:** number 5; the stock's models in `FIRST_CHOICES` order
  and `rest: true`; hosts `Workers`; the run of charter 2 (`WORK_TURNS`,
  `WORK_TIME`, reserve 1 turn and 60 s); `resume: false`; `waiting` 10 s,
  as every stock charter; `inspect`, `modify` and `agents` true, `shell`
  false; `tools` charter 1's, `CHAT_CHARTER_TOOLS` (1,023); no documents;
  `spend` the stock's run, as every stock charter's; `wake`:

  ```rust
  WakePolicy {
      words: WakeRule::Immediate,
      notices: WakeRule::Immediate,
      news: WakeRule::Paced { span: Duration::from_secs(1_800) },
      results: ResultsWake::Each,
      questions: true,
      answers: true,
      timers: true,
  }
  ```
- **Template 4:** `number: 4`, `name: b"goal for a draft"`, `executor:
  TemplateExecutor::Charter(5)`, `contract: Contract::Report { words:
  result_bytes }`, `shallower: 1`, `budget: TemplateBudget::Given`,
  `deadline: TemplateDeadline::Given`, `tracked: Some(1)`, `flags` as
  template 2's (ended by its result, no conversation, listed, kept),
  `label: 2`, `grace: None`, `llm: None`, `resources:
  [jig_domain_people::Symbol { connector: 1, symbol: 1 }]`, the forge's
  own goal branch. The stock's templates are, in order, 1, 2, 4 and 5.
- **The delegation:** the rules' ceiling and policy 1's (and so its roles
  0 to 2, which keep every charter, job and procedure) delegate to, in
  order, `Charter(1)`, `Charter(2)`, `Charter(4)`, `Charter(5)`,
  `Charter(6)`, `Charter(9)`, `Charter(11)`, `Procedure(2)`,
  `Procedure(3)`, `Procedure(4)`, `Job(3)`, `Role(0)`, `Role(1)`,
  `Role(2)`: fourteen kinds, within `authority.executors` 16. Nothing else
  of the rules or policy 1 changes.
- **Counts that follow:** eight stock charters (1, 2, 4, 5, 6, 9, 10 and
  11), four templates and fourteen executor kinds.

## 4. The binary's forge stock (13.4)

`stock::forge` (`S/stock.rs`) gives, in this order:

```rust
Stock { kind: 2, scope: 0, number: 1, value: /* sized as today */ Value::Rule(LandingRule {
    pattern: GatePattern { segments: Box::new([b"branch".as_slice().into()]), last: GateLast::Open(Box::default()) },
    enforce: true,
    ci: true,
    up_to_date: true,
    gates: Box::new([RuleGate { number: 1, blocking: true, freshness: JudgeFreshness::Clean }]),
    approvals: Box::default(),
}) },
Stock { kind: 3, scope: 0, number: 1, value: Value::Charter(2) },
Stock { kind: 3, scope: 0, number: 2, value: Value::Charter(4) },
```

and no entry of kind 1. A world that gives its own forge defaults
(`Given::Own`) gives what it gives: none of this reaches it.

## 5. The reference project's rule 1 (13.9)

`temper/docs/examples/reference/project.json` gains, after its kind 7
item, the project's rule 1, main's, naming a branch:

```json
{
  "setting": {"kind": "landing-rule", "project": 1, "number": 1},
  "set": {
    "pattern": {"segments": ["6272616e6368"], "last": {"exact": "6d61696e"}},
    "enforce": true, "ci": true, "up_to_date": true, "gates": [], "approvals": []
  }
}
```

`settings.json` keeps its kind 1 rules 1 and 2 as they are.
