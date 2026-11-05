# ui-browser-red: the plan

From `inputs/request.md`.

## Parts and tasks

- **browser-says** (skein): `say-what-failed`: the 503's URL, the CDP method and error behind `Protocol`, the real step. Released first: everything else reads its failures.
- **browser-red** (wherever the cause falls): `catch-and-fix`, needs `say-what-failed`: catch each failure, fix it at its lowest tier, say whether a person can hit it.

## Critical path

`say-what-failed`, then `catch-and-fix`.

## Overlaps

- `ui-chat-draft/draft-proof` changes `temper/tests/client-browser/tests/browser_all/browser_pages.rs`. `say-what-failed` touches that file only if the compiler stops it, and `catch-and-fix` not at all.
- `catch-and-fix` may reach `temper/crates/temper-client-view/src/pages/settings.rs` and `skein/crates/skein-ui-web/src/page.rs`, which other drafts change, if the cause lies there. It can't be known beforehand; its spec keeps such edits small.
- `ui-phone`'s cut is in flight, so its files aren't known.

## Decisions

- **`url` beside the text, never in it.** `Setup::expected` is matched by substring, and the test expects `401`: a URL in the text would let a port or a digest holding `401` pass as expected.
- **`detail` beside `Refusal`, never in it.** `Refusal` stays `Copy`, and only `Protocol` needs words.
- **Two tasks, and the cause isn't assumed.** The cut read the code and found leads, not causes; `catch-and-fix` names them as leads and settles each on evidence. Its options say what each finding leads to, so that it needn't ask.
- **`503` joins `expected` only after it's shown harmless,** and the first question is whether one person's page should fill the room at all.
- **The repeats depart from "no unchanged rerun",** because repeating is how an intermittent failure gets caught; the spec says so.
- **No public interface or stored format changes** beyond the browser kit's `Event` fields, which only test code reads.
