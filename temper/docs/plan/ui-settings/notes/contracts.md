# ui-settings: the shapes that cross tasks

Binding for every task of the plan. Where a spec and this note differ, this note wins and the report
says so.

## Kinds 6 and 7 (`temper/docs/design/protocol/configuration.md`, 13.2 and 13.4)

The wire, `client-wire-v1.schema`, under *Values of temper's settings kinds*:

```
enum QuestionsRoute { none, foreman, up }
record ProjectQuestions { route: QuestionsRoute }
record ProjectWays { drafts: bool goals: bool changes: bool away: bool way: u32 }
```

The store, `store-v11.schema`, family 20, reusing its `Route`:

```
record ProjectQuestions { route: Route }
record ProjectWays { drafts: bool goals: bool changes: bool away: bool way: u32 }
enum SettingValue { rule: LandingRule, charter: CharterValue, questions: ProjectQuestions, ways: ProjectWays }
```

The forge, `temper-engine-domain-forge/src/settings.rs`:

```rust
pub enum Kind { DeploymentRules, ProjectRules, Charters, ProjectQuestions, ProjectWays } // 1, 2, 3, 6, 7

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ProjectWays { pub drafts: bool, pub goals: bool, pub changes: bool, pub away: bool, pub way: u32 }

pub enum Value { Rule(LandingRule), Charter(u32), Questions(temper_engine_domain_forge_change::Route), Ways(ProjectWays) }

pub enum Refusal { /* as today */ Way } // "a way that is not a template the project offers"
```

And on the forge's `Domain`:

```rust
/// Kind 6 of `project`, `Route::None` with none stored (configuration.md, 13.2 and 13.4).
#[must_use]
pub fn project_questions(&self, project: u32) -> temper_engine_domain_forge_change::Route;
```

`Setting::Application { kind, scope, number }`, kind 6 or 7, `scope` the project, `number` 1.

## Their names (`temper/docs/design/client/tools.md`, 4 and 5.3)

| Kind | Text | Selector in a settings file | Value in a settings file and `--json` |
|---|---|---|---|
| 6 | `project-questions P` | `{"kind": "project-questions", "project": P}` | `"none"`, `"foreman"` or `"up"` |
| 7 | `project-ways P` | `{"kind": "project-ways", "project": P}` | `{"drafts": true, "goals": true, "changes": true, "away": true, "way": 0}` |
