# tui-approval: the plan

2026-11-07. From `inputs/request.md`: the TUI's M2 (`temper/docs/design/draft/tui.execution.md`, M2) less the TUI's own flow. jig's third built-in way of signing in, a sign-in approved by a signed-in browser, on the wire, the engine and the client; temper's configuration enabling it; the web's approval page; `temper-client-domain`'s half for a native client. The TUI's screens for it follow `tui-connected`'s landing in a plan of their own, on the design this plan lands.

## The shape

The binding calls are `notes/contracts.md`. In short:

- **The approval is the clients' component's,** in memory, as the printed link's state is: asked (a 32-byte secret, an eight-letter code, ten minutes), read and decided by a browser by its code, and taken with its secret. Only the take makes a sign-in: the component asks people `Whose` the approver's sign-in is and signs that identity in as it signs in the printed link's owner. **jig's people child and core are untouched.**
- **A browser approves, nothing else:** the transport tells the component, on each exchange, whether it carried the credential itself as a browser's cookie (`FromTransport::Exchange.browser`); reading and deciding are refused `not_a_browser` otherwise, a bearer's (`temper ctl`'s) among them.
- **The wire gains records only** (`jig-client-wire 19`, in place): no existing enum gains a variant, so no reader of `SignInAnswer` changes.
- **Bounds are fixed constants** of the component (4 live, 8 remembered, 10 minutes), not profile limits, which keeps this plan off `per-wake`'s profile files. Section 8's defaults hold: eight characters, ten minutes, the web client's page.
- **Configuration:** `engine.clients.approval`, optional, false when absent; temper's templates set it beside `link`.

## Parts and tasks

| Part | Task | Needs | Crates |
|---|---|---|---|
| `jig-approval` | `wire` | none | `jig-client-protocol-wire` |
| | `surface` | `wire` | `jig-protocol-engine-clients` (boundary, routes); the compiler's fields in every tree |

Later batches (this file grows with each push): the engine's approval (`component`), the HTTP routes (`http`), configuration (`config`), the client (`client`), jig's joint story (`jig-story`); then temper's part: configuration, the client root's half and the page's state, the view and the gallery, the client world's story.

## Critical path, as it stands

`wire` (60) → `surface` (75) → `client` → `temper-domain` → `temper-view`. `component`, `http` and `config` run beside `client` once `surface` lands.

## Decisions

- The approval lives in the component, not in people: nothing is durable before the take, a restart forgets every approval, and people sees an ordinary sign-in (`jig/docs/design/protocol/client.md`, 8).
- A `surface` task carries the component's new messages, the `browser` field and `Settings.approval`, answering as not enabled, with every compiler-forced field across the trees, so the engine's body, the mapping, configuration and the client start together.
- A take still waiting is sent again by the client itself every second (`jig/docs/design/client/domain/client.md`, 12): the TUI only reads the link's state.
- No stored format changes. The wire and the component's boundary change, before v0.1.0, with no compatibility.

## Overlaps

- **`per-wake`** (integration, not landed) edits `jig-protocol-config` (`members.rs`, profiles), `jig-domain-people`, `jig-client-domain`'s settings and the wire's schema (3 lines of `client-v19.schema`). This plan's schema hunk is new records after `SignOutAnswer`; its config hunk is one optional member in `clients.rs` and its row in `members.rs`; `surface` adds one literal field in `jig-protocol-config/src/clients.rs`. Expect a catch-up where the schema's generated file meets.
- **`tui-connected`** (cut beside this plan) adds the TUI's crates and a small part of `temper-client-domain`. `client` adds one compiler-forced arm each in `temper-client-shell-terminal` (`client.rs`, `terminal/native.rs`), which `tui-connected` may also touch.
- **`look-round-17`** edits `temper-client-view`'s terminal pages and `temper.css`; this plan's CSS goes under its own marker beside the sign-in's rules.
