# codec: status

2026-11-19. Where skein's code lags or departs from [README.md](README.md).

- **Not yet:**
  - 4: a lean family's `Path` and `Problem` are `skein-codec`'s, one type for every family, so a `?` carries one family's refusal into another's result and its name is read from the wrong table. [in flight: change 9883]
  - 4: `Field` is a public trait any crate may implement, no boundary test refuses an implementation of `Record` or `Field`, and a lean module's `check_limits` is one generated line per limit. [in flight: change 9896]
  - 6: tests embed each family's versioned schema file and parse it and build its values their own ways; no owner exports its current text as a constant, and skein-codegen has no `values`. [in flight: change 9885]
  - 6: a memory test decodes a root enumeration at its longest-encoded variant alone; skein-codegen has no `largest_variants`. [in flight: change 9887]
- **Departs:**
  - 6: skein-codegen's construct tests read `every-v3` from its file, because its owner is a private world whose library no other package may name (`fabrig-boundary`'s `skein_worlds` test). Decided by the goal's coordinator, 2026-11-19.
  - 6: an option whose value is an option generates `Option<Option<T>>`, which `clippy::option_option` rejects outside exported items. Only the pin, `every-v3`, nests options, and only as a public field's type, which clippy leaves alone in exported items (its `avoid_breaking_exported_api`); the lean form has no accessor or `Parts` signature for the lint to reach, so its module's declaration carries no expectation. A family that nests options in earnest takes the generator's fix first. Decided by the goal's coordinator, 2026-11-18, to keep the pin off a generator change.
