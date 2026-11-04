# to-begin-store: the plan

The storage behind "Skip" (`temper/docs/design/draft/to-begin.md`, 5, the draft's step 2). Temper
alone: neither jig nor skein changes.

## The design, written first

- `temper/docs/design/protocol/store.md`, 2, 3, 7: `Dismissed`, byte 34.
- `temper/docs/design/client/README.md`, 4.6 and 8: `Dismiss`, the subject `dismissed`.
- `temper/docs/design/domain/forge.md`, 5.5, new: the connector's part.
- `temper/docs/design/client/forge.md`, 9; `temper/docs/design/protocol/configuration.md`, 7.
- Their status files, with **Not yet**; `temper/docs/design/draft/to-begin.status.md`.

## Parts

- **store:** the family, restored and held by the forge connector.
- **wire:** the request, its outcome, the subject and its document.

## Tasks so far

| Task | Part | Needs | What |
|---|---|---|---|
| `dismissed-family` | store | | the row, the key, the record, the restore |
| `dismiss-wire` | wire | | the schema's lines, the codec's functions |

## Critical path

As it stands: `dismissed-family`, about 50 minutes; `dismiss-wire` beside it.

## What the draft left open, or is unclear in it

To be brought up to date with the last push.
