+++
id = "own-way-words"
part = "own-way-words"
kind = "feature"
needs = ["own-way-pane"]
files = [
  "temper/crates/temper-service/src/stock/configuring.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
  "temper/crates/temper-service/src/tests/room.rs",
  "temper/crates/temper-service/Cargo.toml",
]
effort = "medium"
minutes = 25
+++

# The configuring agent reads the same page

## Goal

Asked to set a project up the way temper builds itself, the configuring agent reads the stock page with `read_docs`, proposes its settings sized for the deployment, and keeps temper's own values in a column of "How PROJECT works". These are the agent's words: no new view and no new tool.

Acceptance:

- **`configuring.txt`** holds the paragraph of `notes/contracts.md`, 6, word for word, as its last paragraph but one.
- **A new test** `configuring_txt_starts_from_temper_own_way`, beside `configuring_txt_tells_it_how_project_works`, holds its phrases, and that `temper_docs::find(b"how-temper-builds-temper")` is a page.
- **Charter 3's tools still answer `read_docs`:** read its tools' test, and say which one in the report; no tool changes.
- **The measured length** in `the_configuring_agent_is_charter_3_and_template_3` is the new one.
- **No limit moves:** `every_stock_charter_admits_its_whole_maximal_brief` (`stock_brief.rs`) and `stock_charters_leave_at_least_2048_bytes` (`room.rs`) pass.

## Inputs

- `temper/docs/plan/look-round-8/notes/common.md`.
- `temper/docs/plan/look-round-8/notes/contracts.md`, 6.
- `temper/docs/design/domain/agent.md`, 8.1, *temper's own way*.

## Owns and may change

Owns, as `files` lists them: the words, their tests, the measured figures of `stock_brief.rs` and `room.rs`, and a `temper-docs` dev-dependency line in `temper-service/Cargo.toml`.

Options, each pre-approved:

- **A stock fit fails:** shorten this paragraph alone, keeping `read_docs`, `how-temper-builds-temper` and the column, and say so. No limit moves, and no other sentence changes.
- **`configuring-words/stock-words` has landed on your base,** or lands on it in a catch-up: it rewrites this file and the same length line. Keep its words and add this paragraph in the same place, then measure the length again.

Leave:

- The client, and `temper/docs/help/`.
- `temper/tests/engine/domain/tests/suite/configuring.rs`: it reads the same file and needs no edit. It is run below.

## Steps

1. Add the paragraph, then the test. Set the length, then the figures of `stock_brief.rs` and `room.rs` to what they measure.
2. **Close:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-service --lib`; `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/configuring/)'`. Commit, with the trailer, and push.

## Report

`Done:`, then:

- the test's rule broken and the failure seen;
- the new length and the fit tests' figures;
- which test holds charter 3's `read_docs`;
- any words shortened.
