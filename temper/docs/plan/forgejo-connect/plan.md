# forgejo-connect: the plan

From `inputs/request.md`: the status-gaps question `forgejo-connect`, answered (a) on 2026-11-03
at the director's word. The lead settled the five open points in temper's designs first (this
branch's first commit), taking the suggested settlement and departing from it nowhere:

1. **The wire** (`client/README.md`, 4.6 and section 8): `ForgeAsk` gains `connected:
   ManagedForge`; `How` gains `forgejo_tokens: ForgejoTokens {}`; the requests gain `replace:
   Replace { forge: u16 }`; the values carried. No new outcome, refusal or row field: a Replace
   is answered as a Connect, and the client reads both offers from a row's kind, mode and
   connection. `CLIENT_VERSION` stays 11.
2. **The paste** (`protocol/credentials.md`, 2.2): `Pasted` holds a list of at most three items
   in the codec's own form, in place at `temper-secrets` 1; each request's bundle holds exactly
   the items it takes, else its run stops in its way's words.
3. **The login** (`protocol/forge.md`, 2.1, 2.2 and 2.6): the `Check`'s `Identity` login is
   written into the forge record's bot by the connector, before the carried row's erase and
   `Request::Connect`, so `Connected` names it and a restart reads it.
4. **Connect again by another user** (`protocol/credentials.md`, 2.1; `protocol/setup.md`,
   section 4; `protocol/forge.md`, 2.2): refused `OtherUser` by `Check` against the kept `User`;
   a forge's writer is its user, so another user's tokens are a new forge.
5. **The client** (`client/forge.md`, 5.4; `client/tools.md`, 5.5): Connect with the bot's tokens
   on the web page (Your Forgejo's Details, a kept connected Forgejo's Details) and in `temper ctl
   forge connect [FORGE] forgejo FILE`; Replace on the Forges row and in `temper ctl forge
   replace FORGE FILE`.

The status files record each as **Not yet**; each task closes its entries. The shapes that cross
tasks are `notes/contracts.md` (C1 the bundle, C2 the wire, C3 the forge domain's surface, C4 a
Check's end, C5 its stops in words); every task names `notes/common.md`.

## Parts and tasks

1. **bundle** (`protocol/credentials.md, 2.2`), slice 1.
   - `tasks/pasted-list.md`, interface, high, ~90 minutes, needs nothing: the list schema in
     place, `encode_pasted`/`decode_pasted` on lists, `PASTED_PAYLOAD` computed; its readers in
     the service, the client domain, ctl and the CLI shell moved, the help regenerated for the
     one-item bound's new number.
2. **engine** (`protocol/forge.md, 2.2`), slice 1.
   - `tasks/forge-surface.md`, interface, medium, ~60 minutes, needs nothing: C3's types in the
     forge domain and the root, refused `NotTaken` until the next task; the routing's forced
     arms.
   - `forge-connect` (to come, needs `forge-surface`): the domain's admission of a tokens connect
     (a new connected Forgejo's record, or a kept one not connected or stopped) and of a Replace
     (GitHub, or a connected Forgejo, connected or stopped), `Request::Replace`, `BotLogin` kept
     in the record's bot.
3. **wire** (to come): `wire-connected`, needs `forge-surface`: C2 in place, the generated code
   and goldens, the service's mapping to C3, the client's forced arms.
4. **service** (to come): `check-connect`, the meeting point, needs `pasted-list`,
   `wire-connected` and `forge-connect`: the routing stages `Check`, keeps the tokens, the secret
   and `Origin`, tells `BotLogin`, connects with the record's login; then `replace-run`: Check
   with partial tokens, Paste with the kept `AppId`, connecting again a connected forge.
5. **ctl** (to come): `ctl-connected`, needs `pasted-list` and `wire-connected`: `forge connect
   forgejo` and `forge replace`, the tokens file, the help.
6. **web** (to come): `web-connected`, needs `pasted-list` and `wire-connected`: the Details'
   tokens form, the row's Replace, their words, the client world's bridge and stories.

## Critical path

`forge-surface` (~60) → `forge-connect` (~150) → `check-connect` (~150) → `replace-run` (~120):
about eight hours of implementer time on the longest chain, up to sixteen with the buffer.
`pasted-list` and `wire-connected` run beside the domain; ctl and the web view beside the
service.

## Overlaps

- `pasted-list` and `forge-surface` both edit `temper-service`, in disjoint files
  (`setup/public.rs` and its two test files; `route.rs` and three test fixtures' lists).
- `forge-surface` edits `temper-engine-domain-forge/src/boundary.rs`, `domain.rs` and
  `connect.rs`, and `temper-engine-domain/src/boundary.rs` and `domain.rs`, which
  `reference-m1` and `configuring-p1` branches also edit: each change here is an added variant
  or arm, far from their hunks.
- `pasted-list` regenerates `temper/docs/help/reference/ctl-forge.md` and the
  `help_forge_connect` golden, which `configuring-p1/ctl-chat-template` (merged) touched among
  `temper-client-tools`' goldens: regenerated, never merged by hand.
- The mockups (`client/mockups/pages.js`) draw a connected Forgejo's row with Replace but no
  tokens form in Your Forgejo's Details; they are left as they are (no test reads them).
