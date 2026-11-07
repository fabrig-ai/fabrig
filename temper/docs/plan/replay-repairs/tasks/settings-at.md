+++
id = "settings-at"
part = "refusal-at"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/routing/settings/dispatch.rs",
  "jig/crates/jig-domain-core/src/routing/settings/dispatch/project_policy.rs",
  "jig/crates/jig-domain-core/src/routing/settings/restore.rs",
  "jig/crates/jig-domain-core/src/settings/application.rs",
  "jig/crates/jig-domain-core/src/lib.rs",
  "jig/tests/core/tests/suite/settings.rs",
  "jig/docs/design/domain/engine.status.md",
]
effort = "high"
minutes = 45
+++

# A settings refusal names the failing item's position

## Goal

`malformed_refusal` in `jig/crates/jig-domain-core/src/routing/settings/dispatch.rs` builds every out-of-shape settings refusal with `at: 0`, so a change whose third item, a charter, fails reads as refusing item 0. Make `at` the failing item's position among the change's items, as `jig/docs/design/domain/engine.md`, 17.2 (*out of shape*) now says: counted from 0, as a batch's members are (`routing/party/create.rs`); the last item naming the failing setting, so a repeated setting names its repeat; 0 when no item names it (17.1's whole-set check failing on a setting the change leaves as it was, or a restore, which names no items).

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 4), since the design already says what this task builds.

Acceptance:

1. A new row of `refused_changes_preserve_all_settings_rows_and_the_newest` (`jig/tests/core/tests/suite/settings.rs`): a change of three items whose first two pass and whose third, a charter, fails 17.1's check, refused `NotWellFormed { at: 2, why: Malformed::Settings { setting: Setting::Charter(..), .. } }`, with the settings rows and the newest unchanged as the other rows hold. It fails before the fix with `at: 0`.
2. Every path that refuses a change out of shape names the position: a change's admission (the shape check held in `Core::settings_input`, `preflight`, `decide_with`), a proposal's check (`proposal_check`), a project's policy (`dispatch/project_policy.rs`) and an application's items (`settings/application.rs`). A restore (`restore.rs`) keeps 0.
3. Every existing test still passes; an assertion whose failing item isn't a change's first moves to that item's position, and the report lists each one moved.
4. The entry *17.2: an out-of-shape refusal's `at` is 0...* is gone from `jig/docs/design/domain/engine.status.md`.
5. Focused tests pass in `jig-domain-core` (`--lib`), `jig-core-world` and `jig-conformance-world`, and the fuzzy settings stories: `heavy cargo nextest run --profile fuzzy -p jig-core-world -E 'test(/^fuzzy::settings/)'`. This changes what `at` means, not only a signature, so its readers' tests run (the readers below).

## Inputs

- `temper/docs/plan/replay-repairs/notes/common.md`.
- `jig/docs/design/domain/engine.md`, 17.2 (*Refused whole*, *out of shape*) and 17.1 (the check and its faults).
- `jig/docs/design/client/domain/contract.md`, 4's *not well formed* (what a client reads from the refusal).

## Owns and may change

**Owned:** the `files` above.

**Readers** (assertions only, each `at:` moved to its item's position where the failing item isn't first; no other edit): `jig/crates/jig-domain-core/src/tests/settings/*.rs`, `jig/crates/jig-domain-core/src/settings/tests.rs`, and in `jig/tests/core/tests/suite/`: `settings_proposals.rs`, `settings_proposal_edges.rs`, `settings_project_policy.rs`, `application_settings.rs`, `settings_restore.rs`, `model_act.rs`, `fuzzy/settings.rs`; `jig/tests/conformance/tests/suite/client_settings_proposals.rs`. The survey found single-item or first-item failures in most of them, so expect few moves.

**Not touched:** `temper/crates/temper-service/src/clients/translation.rs` numbers its own refusal of an application item from 1 (`index + 1`, held by `settings_entrances_refuse_cut_or_trailing_values_naming_item_one` in `temper-service/src/tests/clients.rs`). It is temper's, another tree; leave it, and say in the report that it disagrees with jig's 0.

**Options, each with its default:**

- The position needs the change's items where the refusal is made. Default: `malformed_refusal` takes them, so the compiler names every caller:

  ```rust
  /// An out-of-shape refusal (engine.md, 17.2): `at` is the position in
  /// `items` of the last item naming `failure.setting`, or 0 when none does.
  pub(crate) fn malformed_refusal(items: &[s::Item], failure: s::Failure) -> people::Refusal
  ```

  If a private `fn position(items: &[s::Item], setting: s::Setting) -> u32` beside it reads better, use it.
- `admitted`'s `Some((Err(failure), _, _))` arm has lost the change's items. Default: `start_at` builds the refusal while it has them, and `Core::settings_input` (`lib.rs`) holds `Result<Box<s::Change>, people::Refusal>`; `malformed(failure)` then goes or takes the refusal. If holding the position beside the failure reads better, do that.
- `settings/application.rs`'s `proposal_change` builds a change from a proposal's jig items, then its application items: an application item's position is its place in that built list (`set.items.len()` plus its index). `accepted` has the change's items in reach: pass them.
- `restore.rs`'s `count_failure` and `shape_refusal` pass `&[]`: a restore names no items, and 17.2 gives it 0.
- A test whose expected refusal moves because its failing item is not first: move the number, and list it.

## Steps

1. The row of acceptance 1 first. The table's tuple is `(against, items, refusal)`; grow its array to four. Its items: `template_edit(&world, 7)` and one more setting's value that passes (charter 1 with changed instructions, as `charter_and_template` in `jig/tests/core/tests/settings/support.rs` makes one), then a charter other than 1 whose value fails 17.1's check: whichever fault the fixture reaches most simply (a template or model it names that isn't configured, a reference fault; or a value out of shape). Run it, see `at: 0`:

   `heavy cargo nextest run -p jig-core-world --test suite -E 'test(=settings::refused_changes_preserve_all_settings_rows_and_the_newest)'`

2. `malformed_refusal` and its callers per the options.
3. The row passes; break the position once (answer 0), see the row fail, restore.
4. The readers' assertions (acceptance 3), then the status entry (acceptance 4).
5. `cargo fmt`, `python3 -I -B jig/scripts/old_forms.py`, `heavy cargo clippy -p jig-domain-core -p jig-core-world -p jig-conformance-world --all-targets -- -D warnings`, acceptance 5, commit and push.

## Report

As the implementer's text says; also the shape you chose for `settings_input`, each assertion moved, and the fault the new row's charter meets.
