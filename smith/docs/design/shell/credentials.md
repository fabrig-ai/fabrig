# Credentials

In smith's product, an account has one source, which its settings name
(`settings.md`, 1.2), and the local host lends every account's grants the same way,
whatever its source (../domain/host.md, section 7).

## 1. Sources

| `source` | Its keys | What smith holds | Renewed by |
|---|---|---|---|
| `sign_in` | the registration's addresses, moved for a test, and `redirect_port` | a record, `tokens/<account>` | smith, under the record's lock (3) |
| `key` | none | the key, `keys/<account>`, kept by `smith login` | the person, with `smith login` |
| `env` | `variable` | nothing: the variable's value, read at start | whoever sets the variable |
| `program` | none: the endpoint names the program (`settings.md`, 1.2) | nothing: the program's own login, in `programs/<account>/` | the program |
| `none` | none | nothing | nothing |

- **`sign_in`:** smith's own OAuth sign-in, with skein's OAuth client
  and its driver (skein's `oauth.md`, section 6), as a public client
  with the registration its provider's preset carries (`settings.md`, 2.2), never one a
  setting gives.
  - **Its addresses may move,** so that a test reaches the fake issuer
    (../testing.md, section 4): `issuer`, `authorization_url`, `token_url`,
    `keys_url` and `revocation_url`, each `https`, or `http` on
    loopback, and `trust`, the roots their servers are trusted by, in
    the same form as an endpoint's `trust` (`der` or `pem`), the
    machine's by default. The registration itself, its client, name,
    scopes and redirect path, never moves.
  - **`redirect_port`,** optional: the port the loopback listener tries
    first, on a port drawn for the sign-in when that one is taken
    (skein's `oauth.md`, 6.5). Without it, each sign-in draws its port.
    Only the port changes between sign-ins.
  - **What a sign-in checks:** the ID token, by the provider's published
    keys, its issuer, audience, expiry and nonce (skein's `oauth.md`,
    2.2), and the granted scopes. The account is the token's subject,
    never an email. A sign-in without a scope the preset requires keeps
    its record, and the account is not usable for a run until it signs
    in again and allows it (`commands.md`, 5).
  - **The issued client id** is kept with the record, and every
    exchange, refresh and revocation after uses it (skein's `oauth.md`,
    2.1).
  - **One subject per account.** A sign-in whose subject another of the
    settings' accounts holds is refused, naming that account, and its
    new tokens revoked; one whose subject is not the record's is refused
    too. Two names never race one rotating token, and an account never
    changes person.
  - **Refreshed before it lapses** by the local host, under the record's
    lock (3), and lent as a bearer.
- **`key`:** a key the person gave `smith login`, kept privately, sent as
  its endpoint's dialect sends a key (`settings.md`, 2.2), never refreshed.
- **`env`:** the variable `variable` names, holding a key, read at
  start, for unattended use.
- **`program`:** the program's own login, signed in by the program's own
  flow into the account's configuration directory (../protocol/programs.md,
  section 4). smith holds nothing of it: the local host checks the
  source at startup and before each run on it (../protocol/programs.md,
  section 3), grants the account's name, and runs the run on a spawned
  agent (`README.md`, section 5).
- **`none`:** for an endpoint that takes no credential, such as a local
  model's. It lends no grant: a run's LLM whose account it is names no
  account in its charter, so its calls carry no credential
  (../protocol/charter.md, section 2; ../protocol/channel.md, section 6). The
  account's name still has a number, checked with the rest (4).

## 2. The credentials directory

```
<credentials>/                    private, the user's own
├── host-id                       the installation's identifier, urn:uuid:…
├── tokens/<account>              a sign-in's record, replaced whole
├── tokens/<account>.lock/        the record's lock: a directory, never replaced
├── keys/<account>                a key, as `smith login` read it
└── programs/<account>/           a program source's configuration directory
```

- **smith's own,** never another program's: no other program's login,
  such as `~/.codex` or `~/.claude`, is read, and the person's own
  settings, hooks and plugins for a program never load into a run
  (../protocol/programs.md, 4.2).
- **Each file private,** read and written as skein's private files are,
  each replaced whole by durable replace (skein's `io.md`, 5.2 and 5.3).
  The programs directory is private; `smith login` makes each
  configuration directory in it, private, and only the program writes
  there.
- **`host-id`** is the installation's identifier, which a sign-in sends
  to its provider as `ext_agent_host_id` (`settings.md`, 2.2): `urn:uuid:` and a random
  UUID, drawn once, by the first process that finds it absent, and
  stored expecting no file. A process that finds one stored meanwhile
  reads that one. It is not a credential, and names no person.
- **Chats are kept under the state directory** (`settings.md`, 1.1), apart from the
  credentials directory, so that a test or a benchmark gives each run a
  scratch state directory and a durable credentials directory.

## 3. Records across processes

- **Several processes share a record:** two terminals on one account, a
  benchmark's attempts, a live test beside a person's chat.
- **Each sign-in record is shared** (skein's `oauth.md`, 6.4): every
  write of it, a refresh, `smith login`'s keep and `smith logout`'s
  revocation, runs under the record's lock, and first reads the record
  again. A newer generation another process kept is used with no
  refresh; no record is an account signed out; another subject fails
  the grant, so a run never changes person mid-run.
- **The lock** is `tokens/<account>.lock/`, a directory opened and
  locked with skein's `Lock` (skein's `io.md`, section 5): exclusive,
  released when it is closed or its process ends, never left behind by
  a crash. A process that finds it busy asks again on a timer, within
  the driver's lock wait (skein's `oauth.md`, 6.7), and then the write
  fails as busy, and the grant is asked for again.
- **A refresh refused for good,** such as `invalid_grant` or
  `refresh_token_reused`, means the account must sign in again (exit 3,
  `commands.md`, 3); a transient failure keeps the record, and is retried.
- **A crash between the provider's rotation and the kept record** leaves
  a spent refresh token: the next refresh is refused as reused, and the
  person signs in again.

## 4. Account numbers

A charter's LLM names its account by number, a `u32`, and a
transcript's opening turn keeps it (../protocol/charter.md, section 2;
../protocol/transcript.md, section 2). The local host derives each
account's number from its name: the first four bytes of the name's
SHA-256 (skein's `lib.md`, 6.2), read as a big-endian `u32`. It never
comes from the account's place in the settings or among the run's
models, so that an account has one number in every invocation, and a
chat resumed on the same account is not carried over. Renaming an
account changes its number.

Two of the settings' accounts with one number are refused, naming both
(exit 2). Every declared account is checked, not only the run's, and
`none` accounts among them though their numbers name no grant (1), so
that neither the models a run uses nor an account's source changes
whether the settings are admitted.

## 5. What the person sees

- **`smith login` and `smith logout`** act per source (`commands.md`, 5 and 6).
- **Interactive sign-in:** the interactive `smith` signs in a needed
  `sign_in` account that holds no record, at the terminal, as `smith
  login` does, and the run starts once the record is kept. `smith exec`
  never does, and exits 3 naming `smith login`.
- **The notices** (../domain/host.md, section 7):
  - **limited:** the provider says the account has reached its usage
    limit, with when it resets only where it says. The run fails for
    its account (exit 1), naming the usage address the preset gives
    (`settings.md`, 2.2). A run never moves to another account; the person chooses one
    with `--account`;
  - **refused:** the provider refused the account what was asked, such
    as an account not eligible for plan usage, or a scope not granted.
    It is never retried, and the run fails for its account (exit 3),
    with the provider's code and reason;
  - **unauthorized:** the local host refreshes a sign-in's grant once and
    lends the new generation. A key, a variable, a refresh refused for
    good, or a program's login lost, fails the run for its account
    (exit 3), naming the command to run.
- **The provider refused smith's registration** (`invalid_client`) at a
  sign-in or a refresh: exit 2, since no sign-in of the person's mends
  it.
- **Never shown:** credential values appear in no setting, event, trace,
  terminal line or `smith check` output.
- **Refresh tokens are never copied.** An issuer rotates them, and a
  copied one is spent by whichever copy refreshes first. Live tests and
  benchmarks sign in smith's own accounts once, in a credentials
  directory of their own (../testing.md, section 2.3; ../benchmarks/README.md,
  section 6), and never copy a login.
