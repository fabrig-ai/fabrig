# temper

The application, built on skein, smith and jig. Read the root `AGENTS.md` first, then `temper/docs/design/README.md`, which says where each part lives.

- **The design:** `temper/docs/design/`; `temper/docs/design/domain/testing.md` is how the testing strategy applies to temper. Code cites a document by its path from the root and its section: `temper/docs/design/domain/forge.md, 8.2`.
- **Markdown read by tests:** `temper/docs/help/` (with `reference/`) is compiled into temper-docs and read by its rules and CLI help tests, so a help change needs those tests.
- **`temper ctl` reads stdin:** in a script, give each call `</dev/null`.
