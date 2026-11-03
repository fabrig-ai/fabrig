# first-release: status

2026-11-03. Where the design lags or departs from [first-release.md](first-release.md).

`../release.md` carries the download (9). `../protocol/sign-in.md`, 2 carries a deployment at a name of its own behind a TLS proxy (2). Reading without signing in (5.3) is carried by jig's `domain/people.md`, section 13 (the public party, a project's visibility, the public documents, who sees what, the admission), jig's `protocol/client.md`, 17.5 and section 19 (the public reads and their `GET`) and `client/domain/contract.md`, section 12 (jig's public records), and by temper's `../client/forge.md`, section 11 (the forge's public documents), `../client/browser.md`, section 8 (the pages' anonymous mode), `../protocol/sign-in.md`, 2.4 (the proxy and the figures) and `../client/tools.md`, 5.3 (`project amend --visibility`, `goal amend --title`); the code lags them, as their statuses say.

- **Design lags:**
  - 3: the site has no design.
  - 5.1: dev.fabrig.ai's shape (a standalone engine and workers under systemd, over WireGuard) is designed nowhere.
- **Design departs:**
  - 5.3: "A request with no credential is served as that party" holds for public reads alone: any other exchange with no credential is answered signed out (jig's `protocol/client.md`, section 19), so a client still learns it must sign in and the public party's surface is one message. Decided by the cut (plan `public-reading`, 11-03) while the user is away; the security review may say otherwise.
- **Open:**
  - 5.3: people's names on public pages (12). For now hidden, decided conservatively while the user is away (plan `public-reading`, `inputs/request.md`): no public document carries a party's name, login or number, nor a person executor's or a role's holder (jig's `domain/people.md`, 13.3); a project's name is public with the project, by its owners' mark (13.2).
  - 5.3: the mirror fallback, a process holding one observer credential and serving cached pages. For now Later, unless the security review asks for it: the design reads the engine directly, behind the public budget and the proxy's cache.
  - 5.3: the public budget's figures, the user's (jig's `protocol/configuration.md`, section 6, where every profile's are zero, public reading off). Proposed for dev.fabrig.ai: `clients.public.rate` 20 reads a second, `clients.public.bytes` 2 MiB a second, `clients.public.reads` 8 in flight, `clients_http.public_calls` 8, `clients.public.max_age` 5 seconds; the proxy's per address, 5 reads a second and 4 in flight (`../protocol/sign-in.md`, 2.4). With a 5-second `max-age`, a board, a task and the landings read by any number of visitors cost the engine about one read each per 5 seconds per document read.
