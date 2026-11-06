+++
id = "models-add"
part = "page"
kind = "feature"
needs = ["models-switch"]
files = ["temper/crates/temper-client-domain/src/pages/models.rs", "temper/crates/temper-client-domain/src/tests/pages/models.rs", "temper/crates/temper-client-view/src/pages/models.rs", "temper/crates/temper-client-view/src/tests/models.rs"]
effort = "high"
minutes = 80
+++

# "Add a model by name"

## Goal

The form under the sections: a provider and a name, the name's shape checked alone, and the model on the picker reading "added by you". It's the way to a new Claude model on a subscription before a release.

Acceptance:

- An owner picks "Claude", types `claude-haiku-5-5` and presses "Add": the request is `Add { provider, name }`, the field empties, and the row appears, on, reading "claude-haiku-5-5 · added by you · estimated prices", and on the picker with no effort row.
- "Add" is off while the name is out of shape or empty. A name that provider's offer holds says "Claude already has that model." and asks nothing.
- The provider's select holds what `me` offers but the playground's, in `me`'s order, a provider with no usable account among them: a name added there shows once it has one.
- A refusal shows under the form. A member has no form.
- The terminal's form works the same.
- `.temper-models-add`'s overlay rules land under the marker, from the look request's overlay.
- The name's check agrees with the engine's: a table of names against `jig_domain_core`'s `valid_name` isn't available to the client's domain, so `name_shape` and `valid_name` each have the same rows in their tests.
- New gallery state `models-add`. Focused tests pass.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *The `Models` tab*, *"Add a model by name"*.
- `temper/docs/design/draft/model-catalogue.md`, 6.
- `jig/docs/design/domain/engine.md`, 17.11, *An owner's act*; `temper/docs/plan/model-catalogue-2/notes/contracts.md`, 3.
- `git show 47c23ff6ed:temper/docs/design/client/journeys/model-picker/07.html`.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files. May change: `temper-client-domain/src/pages.rs`, `temper-client-view/src/words.rs`, `temper-client-shell-web/temper.css`, `temper/tools/gallery/`, the terminal's goldens, `temper/tests/client/tests/suite/models.rs`.

Options, each pre-approved:

- The field's draft lives in `Models`, as the composer's and the Adopt form's do; `Text` fields in the view take whichever of theirs is closest.
- A provider with no account selected and added to: the row isn't drawn until an account can run it; the field empties and a small "Added. It shows when an account of PROVIDER can run it." stands under the form until the next press.

## Steps

```rust
pub enum Act { Picker { .. }, Name(Box<[u8]>), Provider(u32), Add }
#[must_use] pub fn name_shape(name: &[u8]) -> bool;
```

Tests, each breaking its rule once by hand: `add_asks_once_with_the_provider_and_the_name`, `a_name_out_of_shape_keeps_add_off`, `a_name_the_provider_holds_says_so`, `a_member_has_no_form`, and in `temper/tests/client/tests/suite/models.rs`, `an_owner_adds_a_model_and_it_is_on_the_picker`.

`cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents. Commit and push, with the trailer `Look-request: model-picker`.

## Report

The placeholder and words as landed, where they differ from 07's.
