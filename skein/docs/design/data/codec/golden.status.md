# golden: status

2026-11-23. Where the tree's code lags or departs from [golden.md](golden.md).

- **Departs:**
  - 1: skein-codegen's construct tests read `every-v3` from its file, because its owner is a private world whose library no other package may name (`fabrig-boundary`'s `skein_worlds` test). Decided by the goal's coordinator, 2026-11-19.
  - 1: an option whose value is an option generates `Option<Option<T>>`, which `clippy::option_option` rejects outside exported items. Only the pin, `every-v3`, nests options, and only as a public field's type, which clippy leaves alone in exported items (its `avoid_breaking_exported_api`); the lean form has no accessor or `Parts` signature for the lint to reach, so its module's declaration carries no expectation. A family that nests options in earnest takes the generator's fix first. Decided by the goal's coordinator, 2026-11-18, to keep the pin off a generator change.
