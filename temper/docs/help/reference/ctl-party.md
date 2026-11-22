# temper ctl party

Kind: reference

Parties: make a service and reissue its credential.

## party make

```
party make --project PROJECT --role ROLE --out FILE TEXT [--key KEY]
  --project PROJECT: a project's name or number; required. The project, by name or number
  --role ROLE: a number from 0 to 4294967295; required. The role's number
  --out FILE: a file; required. A new private file for sign-in --service
  TEXT (text): text, at most 1048576 bytes, or - to read stdin; required. The service's name
Make a service party in a project with a role
Prints: `made: party N`; writes the address and one-time credential to FILE, never to output
--json: the record party_make.
Key: --key KEY, optional.
The project's owner makes a party with no person, holding ROLE.
FILE existing refuses the call before anything is sent, exit 1.
The host writes FILE at 0o600, with this engine's address and
the credential answered once. The credential is never printed
or passed in argv. A key replay prints the party and writes nothing.
```

## party reissue

```
party reissue PARTY --out FILE [--key KEY]
  PARTY (party): a number; required. The service's party number
  --out FILE: a file; required. A new private file for sign-in, or - for stdout
Reissue a service's credential and write its service link
Prints: `reissued: party N; its sign-ins have ended`, then how to sign its directory in again
--json: the record reissue-service.
Key: --key KEY, optional.
An owner of the deployment or of every project the service holds
a role in reissues it, not the service itself. Its sign-ins end at once.
FILE must not exist; the service link is written at 0o600.
With --out -, stdout holds only the service link and the answer goes
to stderr. Sign the service directory in again with sign-in FILE.
```
