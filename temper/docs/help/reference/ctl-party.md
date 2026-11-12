# temper ctl party

Kind: reference

Parties: make a service with a project's role.

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
