# tui-native-secret: the link file holds a one-time secret of its own

The request: `inputs/request.md`, the user's pick of 2026-11-08. Today the engine's link file holds the box's printed link, so a `temper` started before the person opens that link spends it, and the browser finds it used. The engine now keeps two one-time links live: the printed link, as today, and a local link, kept only in the file. `notes/common.md` holds the rules every task follows, `notes/contracts.md` the shapes that cross tasks.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `jig-local` | `tasks/local-link.md` | | jig's component draws a local link beside the printed link, says it as `Request::LocalLink { secret }`, spends each apart, draws the local one again at once when spent, and never prints a link for a local link's try; jig's world stories and kit | 150 |
| `serve-local` | `tasks/serve-file.md` | `local-link` | `temper-service` queues local links apart (`Service::local_links`); serve keeps them in the link file and prints the printed links alone; the box's first secret no longer written; the service's either-order test and the box stories | 90 |
| `world-local` | `tasks/world-file.md` | `local-link` | the joint world's file reads the local link; a fresh TUI and a browser each sign in, in either order, and a second TUI by the next local link | 70 |

**Critical path:** `local-link` then `serve-file`, about 240 implementer-minutes. `serve-file` and `world-file` start together once `local-link` merges; neither needs the other.

## What was decided

- **jig's, not temper's:** the component that checks a link's secret is jig's (`jig-protocol-engine-clients`), so the second secret is jig's: *The local link* in `jig/docs/design/protocol/client.md`, section 8, with the message `LocalLink { secret }` in 4.2. "One link is live at a time" becomes two at most, each one at a time, with the request's bounds: each spent once, each expiring as the printed link does (`link_span`), a restart forgetting both.
- **Same route, same wire, no TUI change.** The local link is signed in by the printed link's route and record, and its URL is made as the printed link's, so the file's form is today's and the TUI reads it as it does: `temper-client-tui`, `temper-client-shell-tui`, `jig-client` and the wire don't change. That keeps this plan off `post-landing-probes`' files.
- **A new message, not a field on `Link`:** `Request::LocalLink` adds one arm at each reader, where a field would have changed every `Link { secret }` pattern. No configuration: the local link is drawn wherever the printed link is enabled, and an application that keeps none drops it (`jig-test` does, `service.md`, section 7).
- **Drawn again at once when spent,** as the request asks, so a second `temper` reads a live link. Expiry stays lazy, as the printed link's: `fire` takes no `out`, so an expired local link is drawn again by the next sign-in that finds none live, which the TUI's existing second read (6.14) picks up.
- **The printed link is never drawn again by a local link's secret:** the past links remember which link each was, so a TUI's try with a spent or expired file link never makes the box print. An unknown secret still draws a printed link when none is live, as today: the TUI sends one only when its file was evicted from the 8 remembered links or is from another engine, a fallback case the request doesn't name.
- **The printed link is drawn before the local link,** so a seeded component's printed link is today's; tokens and codes drawn after the start move by one draw, and `local-link` re-pins any jig test that pinned one.
- **The box's first secret is never written to the file;** serve writes the file at each `LocalLink` once its listener is up. The file's path, name (`printed-link-KEY`), mode, directory and removal stay as built, as the request asks.
- **Design written here:** jig's `protocol/client.md`, 4.2 and 8 (*The local link*, the past links' memory, the reservation per link, full's reserved link's expiry); `protocol/service.md`, 7 (`jig-test` drops it); temper's `client/README.md`, 6.12 (the file holds the local link alone; *Written*; *The risk*, the user's pick of 2026-11-08), 6.14 (the second read is for an expired or raced local link), section 7's first bullet and section 10's joint-world stories; `protocol/setup.md`, 13.4 (the box's story); `domain/testing.md`'s "one printed link is live at a time". Status: a **Not yet** bullet per task in `jig/docs/design/protocol/client.status.md`, `temper/docs/design/client/README.status.md` (two, apart) and `temper/docs/design/protocol/setup.status.md`, each deleted by its task. The draft's status, `temper/docs/design/draft/tui.status.md`, 4.1's departure, records the file's own secret as the user's decision of 2026-11-08; the draft's text is unchanged. `setup.md`, 7.2's block says nothing of the file's contents, so it doesn't change.
- **Tests, cheap:** jig's engine-clients world gains three test functions for the new behaviour (two links each apart, a dead local link never printing, a restart forgetting both) and rows in existing tables; temper's service one either-order test; the box's two stories gain asserts; the joint world's browser-spent story becomes a three-row either-order table. No slow tier runs.

The engine's protocol between the component and the application gains a message (`LocalLink`); no stored format and no wire record changes.

## Overlaps with drafts beside this one

- **`walks-lower/walks-go`** changes `temper/docs/design/client/README.status.md` and `temper/docs/design/protocol/setup.status.md`; `serve-file` and `world-file` each delete their own **Not yet** bullet there, and `serve-file` edits one sentence of `setup.status.md`'s prose. Unavoidable: the status follows the code. The two bullets in `README.status.md` sit apart (top and end) so the two tasks' edits don't meet. `walks-go` also owns `temper/crates/temper/tests/fake_backends/**` and `tests/support/**`, which `serve-file` reads and doesn't change.
- **`post-landing-probes`** (finding-463) changes `temper-client-shell-tui`'s `host.rs` and `temper-client-tui`'s state and foot: no task of this plan touches either crate.
- **`ui-ask-m3a`** and **`look-outside-pages`** change `temper-service` files other than `clients.rs` and `system.rs`, and no file of this plan.
- **`temper-program-child`** changes `temper/crates/temper/src/main.rs` and `tests/suite/main.rs`; `serve-file` changes `lib.rs`, `host.rs` and `tests/suite/box_listeners.rs`, not those.
- **`tui-box-starts`** follows this plan and builds on it.
