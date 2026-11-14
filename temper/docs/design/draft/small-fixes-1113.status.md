# small-fixes-1113: status

2026-11-14. Where the design lags or departs from [small-fixes-1113.md](small-fixes-1113.md).

- **Design lags:** none.
- **Design departs:** 1: the forge call's deadline is the lesser of the lifetime and 270 s, not the lifetime, so a Forgejo merge has 270 s, not 300: a GitHub forge refuses a deadline past 270 s, half a JWT's life (skein's `forge/connection.md`, 7.4); an override of the lifetime moves the deadline with it up to that cap (`../protocol/configuration/limits.md`, "The forge client's lifetime and the forge call's deadline"). The user's decision, 2026-11-13.
- **Design departs:** 2: its second want's directory already signed in is ctl's own refusal, made before anything is sent (its first want), never the engine's: ctl sends no link beside a held token, so the engine's refusals it words apart are spent, expired and unknown, the last a link temper does not know (`../client/ctl/README.md`, 6.1). The first want's refusal is the link's `sign-in` alone: `sign-in --service` spends nothing and keeps its behaviour. The goal's designer, 2026-11-14, keeping the draft's intent.
- **Open:** none.
