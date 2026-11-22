# Signing in to an owned forge

This document holds signing in to an owned forge, split from [`owned.md`](owned.md).

## 1. Signing in through temper

**A box's owned Forgejo needs none of this:** it makes its key at its first run
(`init.md`, 2) and adds its own source (`box.md`, 1), with no secret and no command for the person
(`../openid-provider.md`, 9). What follows is for every other arrangement.

`temper-box forge sign-in [DIRECTORIES] FORGE`, for a Forgejo of
`application.sign_in.forges` (`../openid-provider.md`), managed or
owned, on temper's host, before `temper-box serve`:

1. **The provider's key,** when its file, `engine.clients.provider.key`,
   is missing: `openssl genpkey -algorithm RSA -pkeyopt
   rsa_keygen_bits:3072`, run with umask 077 into a file beside the
   key's and renamed over it, so the key is PKCS#8 PEM, mode 0600 and
   the service's own user's. It is never printed, copied or logged; a
   key there is left as it is. No `openssl` on the `PATH` refuses the
   command, naming it.
2. **The forge's client secret,** when its file, the forge's client's
   `secret`, is missing: 32 bytes of the system's randomness in
   base64url, written whole and private (`README.md`, section 3). A secret there is
   left as it is.
3. **It prints,** never running them: the `add-oauth` command
   (`../openid-provider.md`, 5) with the client's id, the secret file's absolute
   path in `--secret "$(cat PATH)"`, never the secret's bytes, and the
   discovery document's URL at temper's public origin, or its listener's
   with none. The source's callback is the forge's browser origin's
   `/user/oauth2/temper/callback`: its owned setup's `public` when present,
   else its record's `web` when it has one, else its `git` URL
   (`../openid-provider.md`, 1 and 7), matching Forgejo's
   `ROOT_URL` (`owned.md`, 2). For an owned Forgejo, the command has the binary and
   its `--work-path` and `--config`; and Forgejo's settings of
   `../openid-provider.md`, 4, which an owned Forgejo's `app.ini` already holds
   (`owned.md`, 2), and a managed one's owner
   writes.

Then `temper-box serve`, and the printed command, once, on Forgejo's host.

- **Printed for `temper-box serve`'s owned Forgejo too, not run** (with o/sign-in,
  2026-10-23), for two reasons: `add-oauth` fetches temper's discovery
  document at once and adds no source when temper does not answer, and
  the preparation runs before `temper-box serve` (`owned.md`, 3, `box-supervision.md`, 1); and `add-oauth`
  takes its client secret only as an argument, where the preparation
  puts no secret (`owned.md`, 3). A source added by Forgejo's command line while
  Forgejo runs works at once, so the order costs a person nothing.
- **Its code:** the command, and the key's and the secret's making, in
  `temper-engine-shell`'s `forge_sign_in.rs`, dispatched from its
  `forge.rs`; the printed bytes in `temper-forge-setup`'s `sign_in.rs`.

## 2. Open questions

None.
