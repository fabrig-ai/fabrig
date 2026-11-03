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
     forge domain and the root, refused `NotTaken`; the routing's forced arms.
   - `tasks/forge-connect.md`, high, ~150 minutes, needs `forge-surface`: the domain's admission
     of a tokens connect (a new connected Forgejo's record, or a kept one not connected or
     stopped) and of a Replace (GitHub, or a connected Forgejo, connected or stopped),
     `Request::Replace`, `BotLogin` kept in the record's bot; the routing answers both `not
     taken` meanwhile.
3. **wire** (`client/README.md, 4.6`), slice 1.
   - `tasks/wire-connected.md`, interface, high, ~90 minutes, needs `pasted-list` and
     `forge-surface`: C2 in place at 11, regenerated; the service's mapping to C3; the client
     domain's `pasted_tokens` and the refusal words a Replace needs; the client's forced arms.
4. **service** (`protocol/forge.md, 2.6`), slice 1: the meeting point.
   - `tasks/check-connect.md`, high, ~150 minutes, needs `pasted-list`, `wire-connected` and
     `forge-connect`: `tokens_connect` stages `Check`; on `Done` the tokens, the secret and
     `Origin` kept, `BotLogin`, the erase, the connect from records as the record's login; the
     stops in C5's words; `forges::record` takes a login-only change in place; one story from the
     wire's bytes to `Connected` against skein's fake Forgejo.
5. **ctl** (`client/tools.md, 5.5`), slice 1.
   - `tasks/ctl-connected.md`, high, ~120 minutes, needs `pasted-list` and `wire-connected`:
     `forge connect [FORGE] forgejo FILE` with a tokens file, `forge replace FORGE FILE` by key
     or tokens, the CLI shell's reads, the help (the lead's words given in the spec) and the
     regenerated reference.
6. **replace** (`protocol/forge.md, 2.2`), slice 2.
   - `tasks/replace-run.md`, high, ~120 minutes, needs `check-connect`: the routing's
     `Replace`, `Check` with the kept token beside the pasted one, `Paste` with the kept `AppId`,
     a second connect of a connected forge.
7. **web** (`client/forge.md, 5.4`), slice 3.
   - `tasks/web-tokens.md`, high, ~150 minutes, needs `pasted-list` and `wire-connected`: the
     tokens draft, Your Forgejo's two forms, a kept connected Forgejo's Details, Connecting, the
     row's "Connect ›"; the client world's bridge and story.
   - `tasks/web-replace.md`, high, ~150 minutes, needs `web-tokens` and `ctl-connected`: the
     row's Replace on GitHub and a connected Forgejo; the client world's bridge, and the joined
     stories of the page's and ctl's connect and replace.

## Critical path

`forge-surface` (~60) → `forge-connect` (~150) → `check-connect` (~150) → `replace-run`
(~120): about eight hours of implementer time, up to sixteen with the buffer. Beside it,
`pasted-list` (~90) → `wire-connected` (~90) → `web-tokens` (~150) → `web-replace` (~150),
about eight hours; `ctl-connected` (~120) runs beside `web-tokens`. Nine tasks, about 18 hours
in all.

## What stays open

- `client/tools.md`, 11.3: the real loop's connected Forgejo stays **Not yet** in
  `client/tools.status.md`: a slow tier's story, outside the gate, for a later cut.
- Forget (`forge forget`, the row's "Forget the admin's token") and the connected-to-managed
  change stay as the design and their status entries have them: not this request's.
- The mockups (`client/mockups/pages.js`) draw a connected Forgejo's row with Replace but no
  tokens form in Your Forgejo's Details; no test reads them, and they are left as they are.

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
- `web-tokens` and `web-replace` add words to `temper-client-view/src/words.rs`, raising
  `PHRASES` and `SEGMENTS`, whose line other drafts' branches (`reference-m8`) also move:
  regenerated where they meet, never merged by hand.
- `check-connect` and `replace-run` follow each other in `temper-service`'s `setup/public.rs`
  and `lifecycle.rs`; `pasted-list` and `forge-connect` touch `public.rs`' `pasted` and
  `route.rs` before them, in other hunks.
- `ctl-connected` regenerates `ctl-forge.md` and the help goldens after `pasted-list` did.
