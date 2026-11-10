# delivery-checks: a delivery checks what the change touched, so agents run no build or test

The owner's decision (2026-11-09). Coding agents run no build or test command. They may use `cargo check` on what they're writing, to find compiler errors. When an agent delivers, the delivery check (`.temper/pre-pr`, run by the host on the agent's warm checkout before anything is pushed; `smith/docs/design/domain/run.md`, 8.1) runs the change's checks mechanically. A pull request into a goal's branch runs no CI. The landing's CI into main stays as it is. Today `.temper/pre-pr` runs only the format check and the old-forms counts, so this plan widens it. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **The check set.** `.temper/pre-pr` finds the packages the delivery changed against its base, and their direct dependents in the workspace. On those it runs, after the format check and the old-forms counts it runs today:
   - `cargo clippy --all-targets -- -D warnings` on the changed packages;
   - the focused tests (`cargo nextest run`, default profile) of the changed packages and their direct dependents.

   A change with no Rust package touched runs what it runs today. The base is the commit the delivery's branch started from, or its goal branch's head when it's newer. Say how the check knows it. If the host doesn't tell it, propose how.
2. **Inherited reds don't block.** A failure the base already has is reported as inherited, with its test name, and doesn't refuse the delivery. Only what this delivery broke refuses it. The refusal's output names each failing check and test with its log's tail, within the host's bounded tail (8.1).
3. **Work in progress before a question.** An agent that delivers to ask a question (the charter's "On a doubt, stop: deliver the work you have") must not be refused for unfinished work. Choose between a delivery check that the agent can mark as work in progress, recorded and never merged until a full check passes, and saving work without a delivery. Say which, and why.
4. **It stays fast on a warm checkout.** No `cargo clean`, no whole-workspace run. The tree-hash cache that skips a tree already passed (`.temper/pre-pr`'s `pre-pr-passed`) keeps working. Report what the checks took on a typical change in this repository, and the longest you saw.
5. **The design says so:** `temper/docs/design/protocol/configuration/reference.md`, and `temper/docs/design/domain/agent/charter.md`, 4, with their status files: what a delivery checks, that a goal branch's pull request runs no CI, and that the landing's CI is the guard. An agent can change what a check runs (8.1, not a security boundary).

## Not in this plan

The deployment's `check_timeout` figure, which the owner raises separately. The charters' words, which are generated elsewhere. Changing fabrig's `AGENTS.md`.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
