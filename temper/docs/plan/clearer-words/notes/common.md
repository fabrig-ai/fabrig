# clearer-words: what every task holds to

- **Nothing else changes** (the request). No behaviour moves but the words each task names: no engine, wire, store or configuration change, no jig, skein or smith file, and no exit status, key or request that differs from today.
- **The design is written.** This cut wrote the sections each task's `## Inputs` cites, and a **Not yet** entry for each. Each task deletes its own entry when its work is done, a departure from the implementer's text, which closes `docs/design/` to you. Nothing else in `docs/design/` is yours: what the code teaches goes in your report.
- **The help is opened by name.** `temper/docs/help/` is yours only where your task names a page. `docs/help/reference/` stays generated: regenerate it with its own test (`SKEIN_CODEGEN_WRITE=1` and the test the task names), never by hand.
- **Other plans hold neighbouring lines.** `temper-client-tools/src/jig/rows.rs`, `src/jig/render/**` and `temper/docs/help/reference/ctl-task.md` are also changed by `note-parts` and `ui-settings` on their own trunks. Touch only the lines your task names: no reformatting or reordering beyond `cargo fmt`.
