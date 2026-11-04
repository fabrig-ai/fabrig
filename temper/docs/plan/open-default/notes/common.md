# open-default: standing rules

Every task of this plan names this note.

- The tree is temper; read `AGENTS.md`, then `temper/AGENTS.md`. No task changes jig, skein or
  smith.
- **The fault, as this cut found it.** Making a project drops the browser's link because the
  web client receives fewer entries in a document's short lists than the box sends:
  - `temper/crates/temper-client-shell-web/src/limits.rs` (~line 59) gives the web client
    `rows: 3`; `temper-client-shell-terminal`'s `limits.rs` (~line 65) gives the terminal
    `rows: 8`, and every other field of the two `jig_client::Limits` is equal. Commit
    `de759e2369` raised the terminal's, `temper init`'s template (`documents.lines` 8) and the
    owned example, but not the web client's.
  - jig's link decodes the creating offer's lists with `client.rows`
    (`jig/crates/jig-client-protocol-link/src/limits.rs` ~200, `bounds_create_charters`,
    `bounds_create_procedures`); since `18cf66fa7c` the engine sends them whole, up to
    `documents.lines` (`jig/crates/jig-domain-core/src/acts/bounds.rs`).
  - Once a project exists, `Me` carries one `MeProject` whose `creating` lists may hold more
    than three entries; the link cannot decode it, pushes `Fact::UndecodableEvent` and drops
    the stream (`jig-client-protocol-link`'s `component.rs` ~657-666): every watch ends
    `Dropped`, the page reads Offline and Behind, and each new link drops again on the same
    `Me`. That jig behaviour is as designed and stays.
- **The design is written:** `temper/docs/design/client/README.md`, 2 (the paragraph *Both
  shells receive eight entries ...*), by this cut. Never change it or any other `docs/design/`
  text; what the code taught goes in the report. Status edits are only those a spec names.
- **Files other drafts change now; keep off them** but where a spec says otherwise:
  `temper/tests/client-browser/src/**`, `temper/tests/client-browser/Cargo.toml`,
  `temper/tests/client-browser/tests/browser_all/main.rs` and any new `browser_*.rs` there,
  `temper/docs/design/client/browser.md`, `temper/docs/design/client/browser.status.md`,
  `temper/docs/design/STATUS.md`, and `temper-client-shell-web`'s files but `src/limits.rs`.
- **The browser tier** (`temper/docs/design/client/browser.md`, 4.3) runs only through
  `temper/scripts/browser.sh [NEXTEST_ARGS...]` (`tools/tiers/run temper browser`, which builds
  the wasm bundle and `temper` through `heavy`), one test alone with `-E 'test(=NAME)'`. It
  reads Chromium from `SKEIN_TEST_CHROMIUM`, the machine's. If that is unset or the bundle step
  fails for want of the machine's tools, say so under `## Left` with the output: it is not
  yours to install, and the rest of your acceptance stands.
- **Budgets are the user's.** The browser tier is capped at 12 tests and 120 s (browser.md,
  4.3 and 4.6) and holds 11 today with a twelfth coming from another draft: no task adds a
  browser test; a step joins an existing one. Its suite CPU budget is 85 s, 60 s per test
  (`tools/tiers/budgets.toml`). Measure a changed tier with `tools/tiers/run --cpu DIR temper
  browser -- -E 'test(=NAME)'` (DIR a new path outside the slot's tracked files) and report the
  figures; over budget is red (`skein/docs/foundation/testing-strategy.md`, 8.1).
- Before pushing: `python3 -I -B tools/citations/citations.py --base origin/main` from the
  repository root shows no new broken citation.
- Commits are small and say why; push your branch.
