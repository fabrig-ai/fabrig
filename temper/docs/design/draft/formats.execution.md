# formats: execution

2026-11-27. How [formats.md](formats.md) gets built. It designs nothing: the goal writes its own design from the draft as it runs, and lands whole, with its own tests. Plans use the focused and fuzzy suites alone; walks come after landing, on the user's word.

## 1. The goals

| Goal | Draft parts | Needs | State |
|---|---|---|---|
| V1, versioned files: `format` in each binary's `temper.toml` and in a settings file and its charter files, absent read as 1; the key table recording each key's format and moves, with conversions as named code; an older format converted as it is read, with its notices; moved keys seeding their new home once, then said to be ignored; gone keys ignored and said; a newer format refused by name; help and the reference page listing each format's moves; `config upgrade` for the three binaries and `settings upgrade`; the frozen fixtures and their check | 1 to 10 | — | landed (`../protocol/configuration/file-formats.md`) |

`.temper/`'s files take the same rule with the ways draft's W2 and the hooks' timeouts ([ways.execution.md](ways.execution.md); [hooks.execution.md](hooks.execution.md)).

## 2. Order

- On the user's approval. It needs nothing, and nothing waits on it: format 2 of any file comes later, with the change that needs it.
