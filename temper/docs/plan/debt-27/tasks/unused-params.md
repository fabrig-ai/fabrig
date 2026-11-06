+++
id = "unused-params"
part = "unused-params"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-people/src/domain/start.rs", "jig/crates/jig-client-protocol-link/src/translate/facts/chips.rs", "jig/crates/jig-client-protocol-link/src/translate/facts.rs", "jig/crates/jig-client-protocol-link/src/translate/documents/task.rs", "jig/crates/jig-client-protocol-link/src/translate/documents/runs.rs", "jig/docs/development/debt/D041-unused-parameters.md"]
effort = "medium"
minutes = 20
+++

# The unread parameters outside `jig-client-view` go

## Goal

Lower jig's D041 as far as this plan may reach. The measure reads 31: 27 are in `jig-client-view`, which this plan stays out of, and four aren't. Delete those four. The item stays open, and says the rest wait for that crate.

Acceptance:

- The measure reads 27, each in `jig/crates/jig-client-view`: `rg -o '\b_(limits|env|out): &' jig/crates -g '!**/tests/**' -g '!**/tests.rs' -g '!**/generated/**' | wc -l`.
- `git diff --stat` shows nothing under `jig/crates/jig-client-view/`.
- `jig/docs/development/debt/D041-unused-parameters.md` stays, status `open`. Its **Where** is `jig/crates/jig-client-view`; its **What** says 27 remain, all there; its **Measure** gives 27; `health_words` has left it, since `health_words(healthy: bool)` no longer takes the parameter; and a sentence says this plan left the crate alone for fabrig/ui. **Paid when** doesn't change.

## Inputs

- `jig/docs/development/debt/D041-unused-parameters.md`, whole.
- `temper/docs/plan/debt-27/notes/common.md`.

## Owns and may change

Owns the files in the header. May change, in call sites only: every caller of the four functions. The compiler finds them; the cut's survey found `jig-domain-people`'s `start_link` called from `jig-domain-core` (`src/start/script.rs`, `src/reads/settings.rs`, its tests), `jig/examples/ops/domain/src/tests.rs`, `jig/tests/people/**` and `jig/tests/core/**`. temper calls none of them. These removals are agreed: a shape no sibling shares (`jig-domain-authority`'s `start_link` takes no `Env`, and each `*_piece` is called by name).

Options, each pre-approved:

- A local that a removed argument leaves unread goes with it. If that leaves a further function's parameter unread and it's private to the file, remove that too; otherwise report it under `## Left`.
- `_bound` on the two `*_piece` functions is as unread as their `_limits`, and goes with it.

## Steps

- `jig-domain-people/src/domain/start.rs`: `pub fn start_link(&mut self) -> bool`.
- `translate/facts/chips.rs`: `chip_phase_fields(value, index, children)`.
- `translate/documents/task.rs`: `application_part_piece(bytes)` and `writer_slot_piece(bytes)`.

`heavy cargo check -p jig-domain-people -p jig-client-protocol-link --tests` until it's quiet, then `heavy cargo check --tests` on each package whose call sites you changed. `heavy cargo nextest run -p jig-domain-people -p jig-client-protocol-link -p jig-domain-core`, and the worlds in `jig/tests/people` and `jig/tests/core` by the names in their manifests. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy` naming each changed package, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says: the measure's reading, the net lines, and anything under `## Left`.
