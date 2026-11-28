# formats: status

2026-11-28. Where the design lags or departs from [formats.md](formats.md).

- **Design lags:** none.
- **Design departs:**
  - 8: a settings file's items are converted by the settings file's own moves, recorded by `FIELDS`' shape and member (`../client/ctl/tools-acts.md`, 1; `../protocol/configuration/file-formats.md`, 3), not as the store converts a value from the layouts a release keeps: a settings file is ctl's own JSON, not the store's encoding, and a release keeps no layout older than dev.fabrig.ai runs ([layouts-both-ways.md](layouts-both-ways.md), 7.4), which would end a file's reading at the deployment's floor against sections 1 and 5. A settings file kept in a repository still applies after a release that reshapes a kind. The goal's designer decided, 2026-11-27, keeping sections 1 and 5.
  - 4: a key records its moves alone, and its format is the format of the move that made it, or 1 (`../protocol/configuration/file-formats.md`, 3), rather than each key recording the format it came in: an addition moves no format, so the number it would record says nothing a move doesn't. The goal's designer decided, 2026-11-27.
  - 5: `temper.toml`'s moved names, `temper.json`'s members (`../protocol/configuration/file.md`, 3.3), stay refused with their words rather than read: none was a key of any format of `temper.toml`, so none is an older file's key, and section 5 refuses an unknown key at every format. The goal's designer decided, 2026-11-27.
- **Open:** 11: smith's `config.toml`; how long a format keeps reading.
