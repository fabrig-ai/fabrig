+++
id = "pg-models"
part = "playground"
kind = "feature"
needs = ["pg-start", "fr-init"]
files = [
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/stock/**",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/welcome_policy.rs",
  "temper/crates/temper-init/template/temper.json",
  "temper/crates/temper-init/src/**",
]
effort = "high"
minutes = 70
+++

## Goal

The scripted model is the playground's alone (draft 6.1): the
playground's project allows only the scripted provider's models, and no
other project allows them, so a person's own account never runs the
playground's scripts and the scripted account never pays for a real
project's work. Built in temper's stock policies: the stock policy 1
(every project but the playground's) names every configured model but
the playground provider's; a new stock policy for the playground's
project names the playground provider's models alone; `temper init`'s
document names that policy as `application.playground.project.policy`.

Acceptance:

- `temper-service`'s `src/tests/stock.rs`:
  - `the_stock_policy_names_every_model_but_the_scripted_ones`: with
    the init template's `engine.llm` and its playground, policy 1's
    ceiling and roles name `Models::Named` bits of every configured
    model but `demo`'s;
  - `the_playgrounds_policy_names_the_scripted_models_alone`;
  - `without_a_playground_the_stock_is_as_before`: a document whose
    `application.playground` is null gets today's stock (`Models::ALL`,
    one policy);
  - the existing stock floor tests pass with both policies.
- `temper-engine-domain-world` (stock fixture from the template):
  `a_chat_in_a_real_project_may_not_name_the_scripted_model` and
  `the_playgrounds_chat_runs_on_the_scripted_model` (new, in
  `temper/tests/engine/domain/tests/suite/roles.rs` or a new
  `suite/playground_models.rs`, your choice; the world's stock fixture
  `tests/fixtures/stock.rs` reads the template and may change to pass
  the playground's provider).
- `temper-init`'s tests: the written document's playground names the
  playground's policy, and the document checks.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 4
  (`application.playground`: `provider`, the scripted model's members,
  `project.name` and `project.policy`).
- `temper/docs/design/draft/playground-reset.md`, 6.1 ("The scripted
  model, the playground's alone"), 6.3 (a real account alone does not
  end the playground).
- `jig/docs/design/domain/authority.md`, the Models bullet (around line
  114: every configured model unless the rules, the policy or a role
  names fewer) and section 6.
- `jig/docs/design/domain/accounts.md`, 4.2 and 4.3 (uses; which
  providers may pay for a task's runs).
- `temper/docs/design/protocol/configuration.md`, 13.4 (the stock).
- The code: `temper-service/src/stock.rs` (`checked`, `charter`,
  `ceiling`, the policy literal), `jig-protocol-config/src/projects.rs`
  (`models`: bits in configuration order), `jig-domain-authority`'s
  `Models`.

Assumed: fr-'s parsed playground settings in `temper-service`
(`settings.rs`, the rename of today's `settings::Demo`: read its name
there) carrying the playground's provider and project policy; pg-start's
playground project made with that policy.

## Owns and may change

Owns the files in the header. May change: the callers of
`stock::checked`/`stock::defaults`/`Given::defaults` (startup and test
fixtures across `temper-service`, `temper/tests/engine/domain`,
`temper/tests/system`, `temper/tests/one-process`, `temper-fake`) to
pass the playground's provider, a call-site change each;
`temper/tests/engine/domain/tests/fixtures/stock.rs` and the new world
test file; `temper-service/src/startup/checks.rs` where it checks the
playground's `project.policy` names a policy (it must accept the new
stock number). `notes/scope.md` applies.

Pre-approved options:

- If a chat's charter default model (the first configured model) is
  refused in the playground's project instead of the task choosing the
  scripted model as today's demo client asks, give the stock a
  playground chat charter whose model is the playground provider's
  first model and make it the playground policy's chat template's
  charter; say so in the report.
- If the policy number 2 is taken by a configured policy in a document,
  take the next free stock number and write it in the template.

## Steps

1. `stock::checked(llm, playground: Option<u32>)` (the playground's
   provider number): the scripted models are that provider's models in
   `llm.models`, as bits in configuration order (`Models::Named`, the
   order `jig-protocol-config` gives). With a playground: policy 1's
   ceiling, role and projections take every other model's bits; policy
   `PLAYGROUND_POLICY` (`pub const PLAYGROUND_POLICY: u32 = 2`) is policy
   1's shape with the scripted bits alone. Without: today's stock.
2. The template: `application.playground.project.policy` is
   `PLAYGROUND_POLICY`'s value; `temper-init`'s tests follow.
3. The world tests; the stock tests; the floors (`floors`) for both
   policies.
4. Then, one at a time:
   - `heavy cargo nextest run -p temper-service --lib -E 'test(/stock::/)'`;
   - `heavy cargo nextest run -p temper-init`;
   - `heavy cargo nextest run -p temper-engine-domain-world`;
   - `heavy cargo nextest run -p temper-service -p temper-system-world -p temper-one-process-world -p temper-fake`
     (readers of the stock: a change of meaning; if a package name
     differs, take the crate's `Cargo.toml` name);
   - `cargo fmt`; `heavy cargo clippy` over every package you changed,
     `--all-targets -- -D warnings`;
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
5. Commit and push.

## Report

The standard report, plus: how a playground chat picks the scripted
model (charter default or the client's choice) and whether the
pre-approved charter was needed; every caller of the stock you changed.
