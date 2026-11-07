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

| Part | Task | Needs | Crates | Minutes |
|---|---|---|---|---|
| `jig-approval` | `wire` | none | `jig-client-protocol-wire` | 60 |
| | `surface` | `wire` | `jig-protocol-engine-clients`'s boundary and routes; the compiler's fields in every tree | 75 |
| | `component` | `surface` | `jig-protocol-engine-clients` (the approval, its tables) | 210 |
| | `http` | `surface` | `jig-protocol-engine-clients-http` | 75 |
| | `config` | `surface` | `jig-protocol-config`, `jig-test-service` | 45 |
| | `client` | `surface` | `jig-client-protocol-link`, `jig-client-protocol-http`, `jig-client`; the compiler's arms in every tree | 180 |
| | `jig-story` | `component`, `client` | `jig-client-kit`'s joint, `jig/tests/client` | 120 |
| `temper-approval` | `temper-config` | `config`, `component`, `http` | `temper-init`'s documents, `temper-service`'s tests, the configuration reference pages | 60 |
| | `temper-domain` | `client` | `temper-client-domain` (`Place::Approve`, `approve`, `asked`); `temper-client-shell-web`'s host | 210 |
| | `temper-view` | `temper-domain` | `temper-client-view`; `temper.css` and one rule test; the gallery's eleven `approve-*` states | 180 |
| | `temper-story` | `temper-domain`, `component`, `client` | `temper/tests/client`: the world's browser mark and approval helpers, two stories | 150 |

- **`wire`** adds the records and `ApprovalCode`; nothing existing changes.
- **`surface`** is small on purpose: the component's four messages, `browser` and `Settings.approval`, answered as not enabled, with every literal and pattern they force. `component`, `http`, `config` and `client` then start at once.
- **`component`** builds the approval in the component, red tests first for each refusal, in the crate's own tables.
- **`http`** routes the four paths and tells the component a browser's exchange by its cookie alone.
- **`client`** builds both halves in the client, the take sent again each second while it waits, and the unlinked mode the gallery and temper's joint world step.
- **`jig-story`** is jig's meeting point: the joint world, real component, real core.
- **`temper-config`** turns the approval on beside the printed link in `temper init`'s two documents (so every box's) and the owned example, and proves a service answers an ask.
- **`temper-domain`** gives the root the approval page's place, codec, state and reads (`approve`), and the native client's ask that signs in at once with the secret and keeps only the code and expiry (`asked`); the web shell keeps the page's address.
- **`temper-view`** draws the page in each state with the design's words, under the banner alone, its look under `temper.css`'s `/* ==== tui: approval ==== */`, and the gallery's states.
- **`temper-story`** is temper's meeting point (`temper/docs/design/domain/testing.md`, 6.3): a client asks, a second, played as a browser, approves through its root's page, the first is signed in as the second's party; a client that isn't a browser is refused.

temper's design is `temper/docs/design/client/README.md`, 3.5 (*The approval page*), 5.1, 5.12 and 7.1 (*A terminal signed in from the browser*: the root's half, and the TUI's flow, which the follow-up plan builds), `client/browser.md`, 2.3, 2.4, 2.5 and 7.2, `protocol/configuration.md`, section 3, and `protocol/setup.md`, 8.3.

## Critical path

`wire` (60) → `surface` (75) → `client` (180) → `temper-domain` (210) → `temper-view` (180): about 705 implementer-minutes. `component` (210) runs beside `client`; `jig-story` (120) and `temper-story` (150) follow it; `http`, `config` and `temper-config` are off the path.

## Decisions

- The approval lives in the component, not in people: nothing is durable before the take, a restart forgets every approval, and people sees an ordinary sign-in (`jig/docs/design/protocol/client.md`, 8).
- A `surface` task carries the component's new messages, the `browser` field and `Settings.approval`, answering as not enabled, with every compiler-forced field across the trees, so the engine's body, the mapping, configuration and the client start together.
- A take still waiting is sent again by the client itself every second (`jig/docs/design/client/domain/client.md`, 12): the TUI only reads the link's state.
- No stored format changes. The wire and the component's boundary change, before v0.1.0, with no compatibility.
- temper's page is the web client's (section 8's default), a place of its own beside `Authorize` at `/terminal?code=WDJB-MJHT`, outside the frame; it learns the take by reading again each second while approved, up to the approval's ten minutes.
- temper's root gains no `Request` variant (its readers are many): the native half's state is read by its host from `Domain::asked()`.
- temper's own loopback check already refuses any clients' listener off loopback in standalone, so `temper-config` adds no check of its own.

## Overlaps

- **`per-wake`** (integration, not landed) edits `jig-protocol-config` (`members.rs`, profiles), `jig-domain-people`, `jig-client-domain`'s settings and the wire's schema (3 lines of `client-v19.schema`). This plan's schema hunk is new records after `SignOutAnswer`; its config hunk is one optional member in `clients.rs` and its row in `members.rs`; `surface` adds one literal field in `jig-protocol-config/src/clients.rs`. Expect a catch-up where the schema's generated file meets.
- **`tui-connected`** (cut beside this plan) adds the TUI's crates, a small part of `temper-client-domain` and the TUI's part of `client/README.md`. `client` adds one compiler-forced arm each in `temper-client-shell-terminal` (`client.rs`, `terminal/native.rs`), which `tui-connected` may also touch; `temper-domain` edits `temper-client-domain` (new modules `approve`, `asked`, and arms in `address.rs`, `boundary.rs`, `domain.rs`, `route.rs`); this plan's README section is 7.1, inside section 7, so it takes no number `tui-connected` may add. Expect a catch-up on those.
- **`look-round-17`** edits `temper-client-view`'s terminal pages and `temper.css`; this plan's CSS goes under its own marker beside the sign-in's rules. The gallery's `names.txt` and `tests.rs` meet other plans' too: on a catch-up they are regenerated from the samples, never hand-merged.
