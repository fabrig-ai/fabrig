# first-goal-fixes: plan

Four findings from the first real goal, cut into three tasks so far. One part of item 1 waits for the user's answer.

## Parts and tasks

| Part | Task | Request | Crates it writes |
|---|---|---|---|
| repair-words | `repair-words` | item 2; item 1's empty name | `temper-engine-domain`, `temper-engine-domain-forge` |
| kept-workspace | `kept-workspace` | item 3 | `temper/tests/worker/checkout` |
| init-text | `init-text` | item 4 | `temper-init`, `temper` (a test) |

Not yet cut: item 1's green head, in `temper-engine-domain-forge-change` and the forge world. See *Waiting for an answer*.

## Critical path

The three tasks need nothing of one another, so each starts at once. The longest is `repair-words`, about 90 minutes.

## What was decided, and why

- **A cancel keeps `forge_change::Hold::Failed`, and only its task's hold changes.** A new `Hold` variant would change the store's and the client wire's schemas, which the ui drafts hold. A cancelled repair, resolution or merge job maps to `tasks::Hold::Procedure` with an account. The account uses the existing 512-byte `producer_failure` field.
- **`change show`'s `held:` line still reads `a delegate or a write failed for good` for a cancel.** Its word feeds a hand-counted total in `temper-client-view/src/words.rs`, which is fabrig/ui's. `forge.status.md`, 8.4 records it.
- **`documents.text_bytes` goes to 512, not the request's 8,192.** By reading, a start takes 8,192, but the web and terminal clients decode documents with `document_text_bytes` 512 and refuse longer text. 512 is the most that breaks nothing. Going further is the user's call.
- **The checkout world's proof is against the fake,** which replaces the tree whether or not `clean` ran, so its stories assert the operations too. Real git's part was run once by hand (git 2.47.3); no test runs it.

## Waiting for an answer

1. **Item 1's green head.** The rule the request cites is not in `forge.md`, 8.4, on main or on `brief-failure`'s trunk. 8.4 says a released failed repair "asks for a fresh repair with the same reason and current forge facts", which the code does. The request says to raise this in a `Question:`.
2. **Item 4 beyond 512.** How a run reads a spec in one call: see the report.

## Overlaps we could not avoid

- `temper-engine-domain/src/route/forge_route.rs` and `forge_routes.rs`' `mod` lines: `ui-home-status/status-run`, `ui-away/decided-tool`. Other functions.
- `temper-engine-domain-forge/src/domain.rs`, `boundary.rs`: `ui-home-status/project-facts`, `status-run`, `ui-away/away-forge`, `decided-tool`, `ui-chat-draft/person-facts`. Other functions, and one doc comment.
- `temper-init/template/temper.json`: `ui-chat-draft/fake-document`. One line.
- `temper-service/src/tests/stock_brief.rs`: `ui-home-status/status-charter`, `ui-chat-draft/chat-stock`, `ui-away/decided-charter`. Only if a figure moves.
- `forge.status.md`, `configuration.status.md`: `adopt-access/adopt-grant`, `ui-away/away-proof`, `ui-chat-draft/chat-stock`, `person-facts`. One entry each.
- `temper/crates/temper/Cargo.toml`, `Cargo.lock`: dev-dependency lines.
