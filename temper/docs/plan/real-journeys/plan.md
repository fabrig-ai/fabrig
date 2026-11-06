# real-journeys: the plan

`temper-client-journeys add` redacts a recording made on a real box, proves it, and writes nothing until a last scan has passed. The request narrowed the draft (2026-11-05): no `temper box --bundle`, no terminal line, no email redaction; `--redact TEXT` stays. The design says so (`temper/docs/design/client/journeys.md`, 3.6).

## What reading the code changed

The draft's "a consistent replacement replays" needs more than a replace:

- **A `stored` value is a binary `Kept`:** `x`s over the whole value make the host drop it and report an undecodable store. Its head stands; its `record` is the original.
- **The stream's name** is the stored credential, and is also issued in the first piece, saved, and sent in every `watch`. It is an original wherever it is saved.
- **A key's digest:** `AddKey`, `ReplaceKey` and `FinishSignIn` carry the sha-256 of the value, so a typed key couldn't replay, and a digest of it would stay behind. The digest is replaced too.
- **Typing is an act a byte:** a typed `s`, `sk` ... each shows `Secret { len }`; replacing `s` everywhere would wreck the recording. A head stays in its own act.
- **A paste is trimmed** before it is carried, so the original is what is inside the surrounding whitespace.
- **The stale refusal prints lines whole:** the proof's own message would leak what the redaction missed.
- **`task-pages` holds `sk-`,** and a 51-byte key's length prefix reads `3`: the key's left boundary is two bytes, not one.

## Splits

`chat-draft`'s recording already holds `data:` lines split across pieces, so a stream is joined before anything else. A model's stream also splits a token across `RunText` deliveries, so each run's text is joined by its offsets, and each drawn page is scanned. `journeys.md`, 3.8, says what stays unseen.

## Parts and tasks

| Part | Task | Needs |
|---|---|---|
| pieces | `stream-fields` | |
| originals | `originals` | |
| drawn | `replay-drawn` | |
| quiet | `add-writes-last` | |

Later batches: `redact` and `scan` on the first two; `add-guard` on all of them; `real-walk`, the host's world, on `redact`, `scan` and `replay-drawn`; `journeys-close` last.

## The critical path

`stream-fields` or `originals`, then `scan` (the longest), then `add-guard`, then `journeys-close`.

## Overlaps

- **`temper/docs/design/client/journeys.md`:** `look-round-6/reading` and `look-round-8/own-way-proof` edit it too. This plan's hunks are its own sections, and section 9's one paragraph.
- **`temper/crates/temper-client-shell-web/src/journey.rs`:** this plan's tasks each add a `mod` and a `pub use` line, in distinct places.
- **`box-link-first`:** none, the request's line under the sign-in block having gone with `--bundle`.
- **`temper/docs/design/client/journeys/**`:** other drafts' by priority. This plan's tests read the recordings and write nothing there: one that a re-recording goes stale for goes stale with the reprint's.
