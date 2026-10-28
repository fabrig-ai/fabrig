# smith

The agent kit, built on skein. Read the root `AGENTS.md` first.

- **The contracts:** `smith/docs/design/domain/`, cited from code at module or type level, never on every field.
- **Module docs** say what state is kept, what is never known, and the entry points and contracts. Each public type and variant has one line of prose: what it is, who sends it and, where it has one, its terminal. A field is documented only when its name and type don't say it: units, bounds, invariants.
- **Production domains use the strict subset** (`skein/docs/foundation/programming-model.md, 10`): exhaustive own-enum matches, skein's bounded containers, checked arithmetic. Review what the compiler can't check: no own-enum `if let` or `let else`, tuple scrutinees, closures, `loop`, `while`, application traits or generics. Test harnesses are ordinary Rust with deterministic clocks and seeds (`programming-model.md, 10.2`).
- **Style:** public names re-exported explicitly, descriptive parameter names, a blank line between items, the standard `Result` name for fallible operations.
- **Reuse skein's test kit:** smith owns no second copy of generic schedules, replay, referee plumbing or counting allocators; a missing shared mechanism goes into skein first.
- **Markdown read by tests:** the design, and the benchmark README and fixture trees.
