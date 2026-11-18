# temper ctl link

Kind: reference

Write a fresh one-time sign-in link to a private file.

## link

```
link --out FILE
  --out FILE: a file; required. A new private file for the sign-in link
Write a fresh one-time sign-in link to a private file
Prints: sign-in link written to FILE; it signs in once, until U; JSON: file and expires
--json: the record link.
Key: none.
DIR must be signed in. FILE existing refuses before anything is sent.
The host writes the link's URL to FILE at 0o600 and makes it durable
before success prints. The link is never printed or passed in argv.
It signs in DIR's party once, until U, in universal time to the second.
Asking again replaces the previous asked link. The engine gives a
link to a person's sign-in alone, where its printed link is enabled.
```
