# D022: skein-ui's `Limits` is written out by hand 25 times

- **Kind:** duplication
- **Where:** `jig-client-view/src/sizes/limits.rs`, `jig-client-view/src/tree/fragments/deployment.rs`, `jig-client-view/src/tree/fragments/result.rs`, `jig-client-view/src/tests`, `testing/jig-client-test-view/src/connector_values.rs`, `tests/client/src`, `tests/ops/src/joint/fixture.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Duplicated code": `skein_ui::Limits` written out by hand)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (a new skein-ui field is added in one place)
- **Size:** S
- **Status:** open
- **Paid when:** jig builds a `skein_ui::Limits` without scenes through one constructor, and no literal with `scene_marks: 0` is left.
- **Measure:** `rg -U -c '(skein_ui::)?Limits \{[^{}]*?scene_marks: 0' -g '*.rs' jig | awk -F: '{s+=$2} END {print s}'` (25 at base)

## What

25 struct literals of `skein_ui::Limits` in 22 files each spell every field, with
`scene_marks: 0` and `scene_bytes: 0`: three in the view's production code
(`jig-client-view/src/sizes/limits.rs`, line 121; `jig-client-view/src/tree/fragments/deployment.rs`, line 753;
`jig-client-view/src/tree/fragments/result.rs`, line 133), the rest in the view's tests, the test view kit,
the client world and `ops`'s joint fixture (34 in 31 files on 10-18).

## Why it matters

A field added to skein-ui's `Limits` breaks 22 files; a value meant to be shared is
retyped in each.

## What pays it back

skein-ui gives `Limits::EMPTY`, an empty tree without scenes a caller sets its
fields over (`skein/docs/design/ui.md`, 15). The literals move onto it.
