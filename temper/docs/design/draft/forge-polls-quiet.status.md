# forge-polls-quiet: status

2026-11-09. Where the design lags or departs from [forge-polls-quiet.md](forge-polls-quiet.md).

Items 1 to 3 are designed in skein's `forge-client.md`, sections 4, 4.1
and 4.2, and 7.7: listings only for issue and pull watches, hearing and
the fallback, and refusals kept for their inputs with failures backing
off to `slow`.

- **Design lags:**
  - 2: `../domain/forge.md`, 5.2 names no fallback interval and says
    temper drops its own echoes before the client, which hearing now
    counts; `../domain/landing.md`, 7.1 and `../domain/after-landing.md`,
    section 3 do not stretch a flight's polls of its run;
    `../protocol/configuration.md` names no fallback member.
  - 4: skein's `forge-connection.md`, sections 4.1 and 11 send no
    validator on a call's reads, and skein's `forge.md`, 3.3 keeps
    Forgejo's none.
- **Open:**
  - 2: whether the fallback interval is the configuration member
    `forge.client.fallback_ms` or a fixed value (asked 2026-11-09).
  - 4: how kept answers are bounded with no new figure (asked
    2026-11-09).
