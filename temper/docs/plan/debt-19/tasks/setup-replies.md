+++
id = "setup-replies"
part = "setup-vocabulary"
kind = "feature"
needs = ["setup-forgejo"]
files = ["skein/crates/skein-forge-setup/src/plan.rs", "skein/crates/skein-forge-setup/src/forgejo.rs", "skein/crates/skein-forge-setup/src/forgejo/**", "skein/crates/skein-forge-setup/src/github.rs", "skein/crates/skein-forge-setup/src/github/**", "skein/crates/skein-forge-setup/src/host/**", "skein/crates/skein-forge-setup/src/tests/**", "skein/docs/development/debt/D037-*.md"]
effort = "high"
minutes = 100
+++

# skein: each dialect's plans read a `Reply` of their own dialect

## Goal

The third of four steps of `skein/docs/development/debt/D014-forge-setup-vocabulary.md`. A plan matches its own dialect's reply, so the 27 arms that reject the other dialect's bodies go, and `reachable`, the one plan that speaks both, becomes two plans.
What the host hands a plan stays one value, so the tests that pin a wrong dialect's reply keep pinning it.

Acceptance:

- The item's **Measure** reads 0: `rg -c 'GithubRows\(' skein/crates/skein-forge-setup/src | awk -F: '{s+=$2} END{print s}'`. A rename alone would make it so, so also:
- `rg -n '\bReply::(GitHub|Unsigned)' skein/crates/skein-forge-setup/src/forgejo skein/crates/skein-forge-setup/src/forgejo.rs -g '!*tests*'` lists only Forgejo's gate; `rg -n 'plan::Reply|Event::Reply' skein/crates/skein-forge-setup/src -g '!tests/**' -g '!*tests.rs'` lists only `plan.rs`, the two gates and `host/`.
- `heavy cargo nextest run -p skein-forge-setup -p skein-forge-setup-world -p temper-fake` passes. A test's assertions stay: the three that pin a wrong dialect's reply hold (`tests/reconcile.rs`, `Unsigned` to a Forgejo plan is `KeyForm`, a GitHub reply is `Unexpected`; `tests/github_installations.rs`, Forgejo's replies to a GitHub child). A test's edits are the rename and `Reachable::github`'s call sites alone.
- `temper/crates/temper-fake/src/backends/tests.rs` doesn't change.
- By hand: Forgejo's gate lets `Unsigned` through as `Status(0)` (`tests/reconcile.rs`'s test goes red); GitHub's gate passes a Forgejo `Document` as `Empty` (`tests/github_installations.rs`'s goes red). Restore each.
- Net production falls or holds (estimate -20 to 0). `skein/docs/design/forge-setup.md`, 4.2 is written; don't change it.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `skein/docs/development/debt/D014-forge-setup-vocabulary.md`
- `skein/docs/design/forge-setup.md`, 4.2 (a plan reads its own dialect's reply): written, and binding

## Owns and may change

- **Owns:** the files in `files`.
- **The shape:**

  ```rust
  // plan.rs: what the host hands any plan. Plans never match it.
  pub enum Reply { Document(..), Rows(..), Empty, Status(u16), Down, GitHubDocument(..), GitHubRows(..), Unsigned }

  // forgejo.rs (or forgejo/reply.rs)
  pub(crate) enum Reply { Document(Document), Rows(Box<[Row]>), Empty, Status(u16), Down }
  /// A plan's reply, or how it ends: `Unsigned` as `KeyForm`, GitHub's bodies as `Unexpected`.
  pub(crate) fn reply(event: Event) -> Result<Reply, Why>;

  // github.rs (or github/reply.rs)
  pub(crate) enum Reply { Document(Document), Rows(Box<[Row]>), Empty, Status(u16), Down, Unsigned }
  /// A plan's reply, or how it ends: Forgejo's bodies as `Unexpected`.
  pub(crate) fn reply(event: Event) -> Result<Reply, Why>;
  ```

  `plan::Reply`'s `Document` and `Rows` keep their names, so `temper-fake`'s three sites compile. **The cut decided a flat outer enum:** nesting it (`Reply::Forgejo(..)`) breaks those sites for nothing, and a `Reply` and an `Event` per dialect would make the three tests above unrepresentable.
- **The rename:** `GithubDocument` and `GithubRows` become `GitHubDocument` and `GitHubRows` in `plan::Reply`, and in `connection.rs` and `run.rs`; the tests' 22 spellings follow.
- **`reachable`:** its GitHub half (`Reachable::github`, `github_advance` and the six `Github*` states, about lines 187 to 362) moves to `github/reachable.rs`, as a plan of its own with states named `GitHub*`, and `host/run.rs`'s `Machine` gains an arm for it. `Reachable::github` has no outside reader, so `github::reachable::Reachable::new` or the like is yours to name; the test call sites follow (`tests/github_memory.rs`, `tests/github_picker.rs`, `host/run.rs`).
- **`skein/docs/development/debt/D037-*.md`:** its forge-setup `Github*` bullet is true of nothing now, so it goes. D037 isn't paid here.
- **The gates' `Err`s take the plans' stop paths,** so a stop's stage and subject hold. If a plan's `other` arm did something other than what its gate now does, keep the plan's behaviour by an arm of its own and say so. If a stage or a subject must change, or `Why` or `Stage` must gain a variant (temper matches them), stop and ask.

## Steps

1. The enums and gates; the rename.
2. Forgejo's plans read their gate, and their arms for the other dialect go. `reply` and `account` are the two doors: about 32 call sites, with inline `Event::Reply` only in `github/accept.rs` and `access.rs`.
3. GitHub's plans, likewise.
4. `reachable`'s split, the `Machine` arm.
5. Break the gates by hand, as the acceptance says.
6. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-setup --all-targets -- -D warnings`; the tests above; `heavy cargo check --tests -p temper-service -p temper-init -p temper-forge-setup -p temper-github-app -p temper-protocol-records -p temper-codecs-world -p temper-engine-domain-forge`. The citations check. Commit and push.

## Report

The common note's items, anything a gate couldn't do for a plan, what you broke and what failed, and the net lines.
