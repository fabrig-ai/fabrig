# formats: status

2026-11-27. Where the design lags or departs from [formats.md](formats.md).

## Design lags

- 1 to 7: `../protocol/configuration/file.md`, sections 2, 3, 5, 6, 7 and 9: `temper.toml` names no format, its table records no format per key, a moved name is refused rather than read, and no binary prints a file at a newer format.
- 8: `../client/ctl/tools-acts.md`, section 1, and `../client/ctl/overlays.md`: a settings file names no format, its items have the reading release's shapes alone, and there is no `settings upgrade`. [ways.md](ways.md), 8.8, gives `.temper/`'s files theirs.

## Design departs

None.

## Open

- 11: smith's `config.toml`; how long a format keeps reading.
