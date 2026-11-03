# open-default: the plan

From `inputs/request.md` and its two addenda: making a project (the welcome's Open Default, the
playground's start) leaves the browser Offline and Behind for good on a fresh `temper box`.
Reproduce it, fix it in the lowest tier that shows it, and land it; the playground look work
waits on it. Every task names `notes/common.md`.

## The fault

Found by reading, by this cut's survey: the browser's client receives three entries in a
document's short lists (`temper-client-shell-web`'s `limits.rs`, `rows: 3`), while the box
sends up to eight since `de759e2369` (`temper init`'s `documents.lines` 8; the terminal's `rows`
raised to 8 in the same commit, the web client's left at 3) and `18cf66fa7c` (the engine sends
creating's charter and procedure lists whole). Once a project exists, `Me` carries its creating
offer; jig's link decodes those lists with `client.rows`, cannot decode `Me`, pushes
`UndecodableEvent` and drops the stream (`jig-client-protocol-link`'s `component.rs`): every
watch ends, the page reads Offline and Behind, and every reconnect drops again on the same `Me`.
That matches each line of the report: the store wrote the project, serve went idle with no
client linked, the Forges and Accounts pages came up empty after a reload, `/chats` rendered.

- **Not the second addendum's hypothesis:** nothing differs between wasm32 and native. The four
  "scope size expectations to non-wasm targets" commits only scope `#[expect]` lints, and the
  policy and store-size commits (`6ed91f5429`, `c743971fbe`, `3d524c8f4b`, `655de5b63c`,
  `239f6e6bf8`) never reach a client bound. The native clients (the terminal at 8, ctl at 32)
  receive the lists, which is why no native test saw it.
- **Not jig's:** jig's link drops a stream on an undecodable record, as designed. The fault is
  temper's configuration of its browser client, so the fix is temper's, in
  `temper-client-shell-web`; jig needs no reproduction.
- **No bisect:** the cause is read from the code. The browser tier's run before the fix
  (`web-rows`, step 2) confirms it.

## The design, written by this cut

- `temper/docs/design/client/README.md`, 2: the paragraph on receiving capacity now says both
  shells receive eight entries, the browser's client bounds being the terminal's, and what a
  longer list does to a client's link. It said "the native terminal" alone.
- `temper/docs/design/client/README.status.md`: a **Not yet** item for section 2 (the browser's
  `rows` 3), which `web-rows` removes.

## Parts and tasks

1. **web-rows** (temper, `temper/docs/design/client/README.md, 2`), slice 1.
   - `tasks/web-rows.md`, high, ~70 minutes, needs nothing: `rows: 8` in the web client's
     limits; a guard in `temper`'s `suite` that the browser's client bounds equal the terminal's
     (which already fit the box's and the owned example's documents); browser test 11 gains
     link checks once the playground's project is made and across a reload; the tier run before
     the fix (red) and after (green), with its CPU figures; the status item gone.

Critical path, so far: `web-rows` alone.

## What was decided, and why

- **One field, pinned by equality.** The web client's bounds were meant to be the terminal's
  (`limits.rs` says so) and differ only in `rows`. A test that the two are equal guards every
  future field, where a test on `rows` alone would guard one.
- **No new browser test.** The browser tier is capped at 12 tests (browser.md, 4.3, the user's)
  and holds 11, with a twelfth coming from playground-browser. The link checks join test 11,
  which already makes a project (the playground's Demo) on a real `temper box`.

## Overlaps with other drafts

- `temper-client-shell-web/src/limits.rs`: public-reading-build's `public-web` owns
  `temper-client-shell-web/**`; `web-rows` changes one line of `limits.rs`.
- `temper/crates/temper/Cargo.toml` and `Cargo.lock`: one dev-dependency line.
