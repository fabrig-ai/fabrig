+++
id = "models-switch"
part = "page"
kind = "feature"
needs = ["models-tab", "model-act"]
files = ["temper/crates/temper-client-domain/src/pages/models.rs", "temper/crates/temper-client-domain/src/tests/pages/models.rs", "temper/crates/temper-client-view/src/pages/models.rs", "temper/crates/temper-client-view/src/tests/models.rs"]
effort = "high"
minutes = 110
+++

# An owner's switch on each row, and what the row says of where it came from

## Goal

Each model's row takes its switch for an owner in the deployment, which asks `ChangeModel`, and says "added by you" and "estimated prices" from `me`; a provider's section ends with how many models it left out, where it left any. A model turned off stays in All models and off the picker: the picker already filters on `picker`.

Acceptance:

- An owner presses "Sonnet 5.5 in the picker": the request is `Picker { number, on: false }`; the switch holds its state until the answer; then `me` redraws it "Off", and the picker's menu has no Sonnet 5.5, while All models has it.
- A refusal shows under its row, in the words of a change's refusal; a press while one is pending does nothing.
- A member sees "On" or "Off" and no switch.
- A row reads " · added by you" and " · estimated prices", in that order after the default, from `source` and `estimated`.
- `left_out` of 3: `.facts`: "3 more models don't fit the catalogue."; of 0, nothing.
- Stays with a chat's picker: a model the task holds, turned off, still names itself in its chat (`temper/docs/design/draft/model-catalogue.md`, 3).
- New gallery states `models-off`, `models-added`, `models-member`. Focused tests pass.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *The `Models` tab*.
- `jig/docs/design/client/domain/contract.md`, 3.2, *`ChangeModel`*, and 5.1.
- `temper/docs/plan/model-catalogue-2/notes/contracts.md`, 3 and 4.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files. May change: `temper-client-domain/src/pages.rs` (`Pages`' new page and its act, beside `away`'s or `builders`'), `temper-client-domain/src/acts.rs`; `temper-client-view/src/words.rs`; `temper-client-shell-web/temper.css`, under the marker; `temper/tools/gallery/`; `temper-client-shell-terminal/src/tests/golden/*`, regenerated; `temper/tests/client/tests/suite/models.rs`.

Options, each pre-approved:

- The closest act is `forge/ways.rs`, its `Set`, its pending and its refusal, without its stale retry: `ChangeModel` is never stale.
- The switch is `settings.rs:338`'s (`look::SETTINGS_SWITCH`, `temper-settings-toggle-row`), reused by a `pub(crate)` if that's its only edit.

## Steps

```rust
// temper-client-domain/src/pages/models.rs
pub enum Act { Picker { number: u32, on: bool } }    // `models-add` adds Add
pub struct Models { pending: Option<Pending>, refused: Option<(u32, Outcome)> }
impl Models {
    pub fn refused_at(&self, number: u32) -> Option<&Outcome>;
    pub fn pending(&self, number: u32) -> bool;
}
```

Tests, each breaking its rule once by hand: in the domain, `a_press_asks_the_act_once`, `a_refusal_stays_with_its_row`; in the view, `an_owner_has_the_switch_and_a_member_the_word`, `a_row_says_where_it_came_from`, `left_out_says_how_many`; in `temper/tests/client/tests/suite/models.rs`, `an_owner_turns_a_model_off_and_it_leaves_the_picker`.

`cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents. Commit and push, with the trailer `Look-request: model-picker`.

## Report

What the picker's filter did when the switch went off; whether `look-round-11` had landed.
