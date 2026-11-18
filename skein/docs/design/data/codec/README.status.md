# codec: status

2026-11-18. Where skein's code lags or departs from [README.md](README.md).

- **Not yet:**
  - 1, 2 and 4: `skein-codec` has no `Record`, helpers, `Path`, `Problem` or bound walk, and the generator writes the sealed form alone, with no `lean` mark. [in flight: change 8451]
  - 4: jig's tasks store family is generated sealed. [in flight: change 8452]
  - 6: there is no `missing`, and the completeness check can't fail. [in flight: change 8448]
  - 6: there are no `roots` and `largest`. [in flight: change 8449]
  - 1, 4 and 6: only the families their owner lists in a `lean` file in its `schema/` directory, one family name a line (jig's tasks store), and skein's `every-v3`, written both ways as its pin, are generated as lean records. Every other family keeps the sealed form: private fields behind a `Parts` twin, constructors that check, accessors and `into_parts`, a `Path` enum per family, bounds as generated arithmetic, and functions split by field. `skein_codegen::generate` writes the sealed form, `generate_as` either, and the command the form its first schema's mark gives.
- **Departs:** 6: an option whose value is an option generates `Option<Option<T>>`, which `clippy::option_option` rejects, so generated code does not pass the lints there. Only the pin, `every-v3`, nests options; its sealed module's declaration in `skein/tests/codec`'s library carries a scoped `#[expect(clippy::option_option, ...)]`, and the generated text carries none. The lean module's declaration carries none: its nested option is only a public field's type, which clippy leaves alone in exported items (its `avoid_breaking_exported_api`), and the lean form has no accessor or `Parts` signature for the lint to reach. A family that nests options in earnest takes the generator's fix first. Decided by the goal's coordinator, 2026-11-18, to keep the pin off a generator change.
