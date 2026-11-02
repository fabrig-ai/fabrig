# Every live-smith-manual task

- Read `AGENTS.md` and smith's own `AGENTS.md` first; follow their checks and style.
- The design is `smith/docs/design/testing.md`, 2.3 (the live suite, inside "End to end"). Change
  only your spec's `files`; anything else is a `Question:`.
- **Never run a live test.** The live stories call real providers with real logins: listing
  them (`nextest list`) is allowed and is how this draft is checked; `nextest run` on the
  `live-smith` profile, or on the `live` binary of `smith` or `smith-agent-process-world`, is not.
- Never read, print or set a credential: no API key, no `$SMITH_LIVE` record, no login.
- `.config/nextest.toml` is not this draft's: another draft (live-temper) is changing it.
- Your report's first line is `Done: ...`, `Question: ...`, `Infra: ...` or `Failed: ...`.
