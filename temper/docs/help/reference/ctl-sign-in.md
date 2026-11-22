# temper ctl sign-in

Kind: reference

Sign in with a person's link, a service link or a service file.

## sign-in

```
sign-in FILE
  FILE (file): a file; required. A file holding a person's or service's link, or - for stdin
sign-in --service FILE
  --service FILE: a file; required. The private file party make wrote
Sign in with a person's link, a service link or a service file
Prints: Your party, as me prints it
--json: the record sign_in.
Key: none.
Writes the token, engine's address and party into DIR.
A person's link signs in once and expires. A held token refuses it
before sending, leaving it unspent: run `temper ctl DIR sign-out`
first, or use another directory. `the sign-in link was refused`
(exit 1) says whether the link was used, expired, or unknown; other
refusals give no reason. For a fresh link, use temper-box serve or
ask with `temper ctl DIRECTORY link --out FILE` when signed in.
With all sign-ins in use, it says when the earliest ends and when
the link expires. The link's file stays as it was.
A service link from `party reissue` signs in as the service, even
in a signed-in DIR. If refused, an owner can replace it with
`temper ctl DIRECTORY party reissue PARTY --out FILE`.
`--service FILE` takes party make's owner-only file instead; it
refuses any other form or mode before sending.
```
