# ui-browser-red: the plan

From `inputs/request.md`.

## Parts and tasks

- **browser-says** (skein): `say-what-failed`: the 503's URL, the CDP method and error behind `Protocol`, the real step. Released first: everything else reads its failures.
- **browser-red** (temper, jig or skein, as the cause falls): `catch-and-fix`, needs `say-what-failed`: catch each failure, fix it at its lowest tier, say whether a person can hit it. Follows in the next push.

## Critical path

`say-what-failed`, then `catch-and-fix`.

## Overlaps

None with the drafts beside this one on `skein-browser` or `skein-fake-person-browser`.

## Decisions

- **`url` beside the text, never in it.** `Setup::expected` is matched by substring, and the test expects `401`: a URL in the text would let a port or a digest holding `401` pass as expected.
- **`detail` beside `Refusal`, never in it.** `Refusal` stays `Copy`, and only `Protocol` needs words.
