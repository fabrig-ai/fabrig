# Connect GitHub

Kind: walk

Connect a box to GitHub through a GitHub App, accept or make its installation, and adopt one of its repositories, with no browser when the App and its installation exist already.

## Before you start

- **A box's directories,** as temper help box makes them in its step
  1.
- **The App,** one of:
  - an App of your own: its id and a private key GitHub made for it;
  - temper's App, made from its manifest: a browser on the engine's
    machine makes and installs it.
- **The App's private key** in `FILE`: regular, owned by you, mode
  0600.
- `APP`: the App's id, from its page on GitHub. On fake backends, the
  box prints `box: GitHub own App id 1, key file FILE`: `APP` is 1 and
  `FILE` that file.
- `FORGE`: GitHub's forge number, which temper gives it at its
  connect and `forge list` prints (step 2).
- `OWNER`: the repository's owner, as `forge repositories` prints it.
- `NAME`: the repository's name, as `forge repositories` prints it.
- `REQUEST`: a file of the words that ask for a change in it.
- `CHANGE`: the change task's number, as the chat's tree lists it.

## 1. Start the box and sign in

```
temper box --config CONFIG --data DATA --cache CACHE
temper ctl DIR sign-in -
```

On fake backends, start the box as temper help box says in its step
2. Give the link on stdin.

Expect: `party 1 · owner · Owner`, the owner of a new box.

If not: as temper help box says in its steps 2 and 3.

## 2. Connect the App

An App that exists, its key read from `FILE`:

```
temper ctl DIR forge connect github-app APP FILE --key KEY
temper ctl DIR forge list --until settled
```

With an App another person or tool also uses, add --read-only: temper
then leaves the App's webhook and installations as they are.

temper's App, made from its manifest: open the address it prints in
a browser on the engine's machine, create the App, then install it:

```
temper ctl DIR forge connect github --key KEY
temper ctl DIR forge list --until settled
```

Expect: `connecting`; the wait prints the forges, GitHub's line
`forge FORGE · GitHub · connected · ` and its state, its
installations below it, then `met`.

If not: `credential file must contain 1 to 4085 bytes` or a key
file's mode (exit 1): mend `FILE`, then the same call with the same
key. `connected already` (exit 2): the forge holds a key; `forge
replace FORGE FILE` replaces it. An App short of a permission or an event
connects, and `temper serve` prints what it lacks on its stderr,
`temper serve: forge FORGE: App lacks ` then each name, which a box writes
to its `serve` log under `DATA/box/logs/`: grant them in the App's
settings on GitHub.

## 3. Accept an installation

An installation made already, by its id as `forge list` lists it:

```
temper ctl DIR forge list
temper ctl DIR forge accept FORGE INSTALLATION
```

Or a new one, on another account, through a browser on the engine's
machine; on fake backends, any HTTP client completes it:

```
temper ctl DIR forge install FORGE --key KEY
temper ctl DIR forge list --until settled
```

Expect: `accepted: installation INSTALLATION on forge FORGE`, then the
`forge repositories` call to make; or the forges, the new installation
listed, then `met`.

If not: `GitHub has no installation INSTALLATION of this App` (exit
2): read `forge list` for its ids. An installation not accepted is kept
and shown; accept the one temper works through.

## 4. Adopt a repository

```
temper ctl DIR forge repositories FORGE --installation INSTALLATION
temper ctl DIR repository adopt --project PROJECT --forge FORGE --owner OWNER --name NAME --landing main --no-home --key KEY
```

`--landing` names the branch changes land on. `--no-home` keeps the
home the project has, the box's Forgejo repository; without one,
leave it out, and the repository becomes the home.

Expect: a line per repository, `OWNER/NAME`, then `No more
repositories follow.`; then `Adopted OWNER/NAME on forge FORGE as
owned.`

If not: `The project has a home already.` (exit 2): it prints the call
with `--no-home`; make it with a new key. `No repository OWNER/NAME on
forge FORGE that temper's forge user can see.` (exit 2): the installation
does not reach it; grant it to the App on GitHub's installation page.

## 5. Work on the repository

Name `OWNER/NAME` in the request:

```
temper ctl DIR chat start --project PROJECT - --key KEY < REQUEST
temper ctl DIR change show CHANGE --until landed --timeout 3600
```

The chat's tree names the change task, as temper help box says in its
step 6.

Expect: `done: task TASK, chat`; then the change's panel, its pull
request on GitHub, and `met`.

If not: as temper help box says in its steps 6 and 7.

## Clean up

- Export a signed-in account, then stop the box, as temper help box
  says in its step 8.
- Delete the App's key file once the forge is connected: temper keeps
  the key, and GitHub makes a new one on its App page.
- `forge remove FORGE` removes the forge from temper; the App stays
  yours, to delete in its settings on GitHub.
- Leave the installations you did not accept to their owners.

## Next

- temper help box: the box, and a change on its own Forgejo.
- temper help ctl: the loop, keys and exit statuses for every call.
