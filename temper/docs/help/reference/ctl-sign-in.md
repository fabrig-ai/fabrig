# temper ctl sign-in

Kind: reference

Sign in with a printed link or a service file.

## sign-in

```
sign-in FILE
  FILE (file): a file; required. The file holding the link, or - for stdin
sign-in --service FILE
  --service FILE: a file; required. The private file party make wrote
Sign in with a printed link or a service file
Prints: Your party, as me prints it
--json: the record sign_in.
Key: none.
It writes the token, engine's address and party into DIR. A link
signs in only into a directory not signed in: sign-out first,
`temper ctl DIR sign-out`, or use another directory. A held token
refuses before sending anything, leaving the link unspent.
A link signs in once and expires. `the sign-in link was refused`
(exit 1) distinguishes `it was already used`, `it has expired`,
and `temper does not know it, as another deployment's link or
one it no longer remembers`; each ends `temper-box serve prints
a fresh one`. Other refusals keep the line without a reason.
With every sign-in in use, it says when the earliest ends and
until when this link stays usable. It leaves the link's file as
it was. With --service FILE, sign in as the service at the address
FILE holds, even into a signed-in DIR. FILE must be owner-only
and in party make's form; otherwise exit 1 before sending anything.
```
