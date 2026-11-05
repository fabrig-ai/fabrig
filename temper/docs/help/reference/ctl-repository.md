# temper ctl repository

Kind: reference

Repositories: make, copy or adopt one into a project.

## repository adopt

```
repository adopt --project PROJECT [--forge FORGE] --owner OWNER --name REPOSITORY --landing BRANCH [--role owned|fork|context] [--prefix PREFIX] [--no-home] [--no-ci] [--check GATE]... --key KEY
  --project PROJECT: a project's name or number; required. The project
  --forge FORGE: a number from 0 to 65535; optional. The forge's number; 1 when absent
  --owner OWNER: text, at most 1048576 bytes, or - to read stdin; required. The repository's owner on the forge
  --name REPOSITORY: text, at most 1048576 bytes, or - to read stdin; required. The repository's name
  --landing BRANCH: text, at most 1048576 bytes, or - to read stdin; required. The branch its changes land on
  --role owned|fork|context: one of owned, fork or context; optional. owned, fork or context; owned when absent
  --prefix PREFIX: text, at most 1048576 bytes, or - to read stdin; optional. The prefix of temper's branches; temper/ when absent
  --no-home: a switch. Do not make it the project's home
  --no-ci: a switch. Do not use the forge's CI
  --check GATE: a number; repeated, at most 8. A gate, by its number, each once
Adopt a forge's repository into a project
Prints: `Adopted OWNER/NAME on forge N as ROLE.`, whether it is the project's home, and what temper may do there
--json: the record adopt.
Key: --key KEY, required.
A project has one home; a context repository is read, never
written, and never the home. A refusal says what mends it: another
project writes the repository (adopt it as context there), the
project has a home already (add --no-home, or choose a new
project), or the branch prefix is another deployment's (choose an
unused one); the last two print the call to make, with a new key.
`It is not known whether OWNER/NAME was adopted.` (exit 1): make
the same call again, with the same key.
```

## repository make

```
repository make --project PROJECT [--forge FORGE] ORG/NAME [--no-home] [--until CONDITION] [--timeout DURATION] --key KEY
  --project PROJECT: a project's name or number; required. The project
  --forge FORGE: a number from 1 to 65535; optional. The forge's number; 1 when absent
  ORG/NAME (repository): text, at most 1048576 bytes; required. The destination organization and repository
  --no-home: a switch. Do not make it the project's home
  --until CONDITION: one of adopted; optional. Wait until adopted
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. How long a wait lasts; 300 seconds when absent, at most 3600
Make an empty repository and adopt it into a project
Prints: `kept`; with --until adopted, `ORG/NAME is in PROJECT.` once adopted
--json: the record repository-make.
Key: --key KEY, required.
It makes the starter workflow and adopts the default branch, owned,
with CI. A project has one home: add --no-home if it has one already,
and make the call under a new key. --timeout needs --until adopted.
A wait exits 1 on a failed row, 2 on timeout; the request goes on.
Repeat the same call with the same key to get kept and wait again.
```

## repository copy

```
repository copy --project PROJECT [--forge FORGE] ADDRESS [--as ORG/NAME] [--no-home] [--until CONDITION] [--timeout DURATION] --key KEY
  --project PROJECT: a project's name or number; required. The project
  --forge FORGE: a number from 1 to 65535; optional. The forge's number; 1 when absent
  ADDRESS (address): text, at most 1048576 bytes; required. HOST/OWNER/NAME, optionally with https:// before and .git after
  --as ORG/NAME: text, at most 1048576 bytes; optional. The destination; the source's owner and name when absent
  --no-home: a switch. Do not make it the project's home
  --until CONDITION: one of adopted; optional. Wait until adopted
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. How long a wait lasts; 300 seconds when absent, at most 3600
Copy a public repository and adopt it into a project
Prints: `kept`; with --until adopted, `ORG/NAME is in PROJECT.` once adopted
--json: the record repository-copy.
Key: --key KEY, required.
ADDRESS is HOST/OWNER/NAME, with optional https:// and .git. It must
be public. An invalid address sends nothing. --as chooses another
organization and name. Adoption is owned, on its default branch,
with CI. A home refused needs --no-home and a new key.
--timeout needs --until adopted. A wait exits 1 on failure, 2 on
timeout; the copy goes on. The same key answers kept and waits again.
```
