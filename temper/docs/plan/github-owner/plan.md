# github-owner: the plan

The personal GitHub account leaves temper's code and scripts, as `inputs/request.md` asks. One part,
`owner`, one slice, one task, `neutral-owner`. No design change.

## Where the name is

`git grep` across every tree gives the request's list and nothing new: six files of code and scripts,
all in this task, and two design files, `temper/docs/design/domain/testing.md` line 1238 and
`temper/docs/design/draft/github.md` lines 620 to 629, which `fail-stop-names` rewords. The other hits for
the surname are the private domain or the licence, not the account, and none are this plan's:
`fail-stop-names` takes the skein transcripts and smith's grader, and `hearth-docs` takes `hearth/`.

## Does a story need a User owner?

**One does, and only for the App, not the repository.**

- **`proof-live-github.sh` (`live_github`): no.** It already allows `fabrig-ai`, its relay already admits
  the pull's GraphQL read an organization needs, and it uses the fixtures' App.
- **`proof-live-github-managed.py`: yes, for the App's maker.** Connect always uses `Target::Personal`
  (`temper/crates/temper-service/src/setup/public.rs:320`; `temper/docs/design/protocol/setup.md`, 17.2),
  so the conversion's `owner.login` is the person, a User. The App is public, so it installs on an
  organization. `relay_managed.rs` holds one login for both the conversion and the repository: turned
  into `fabrig-ai` whole, the fake run would refuse its own conversion once it is faithful, and a live
  sitting would stop at `learn:conversion-owner`.
- **So the guard splits.** The repository and installation owner stays a fixed name in the file,
  `fabrig-ai`, and is never an input. The conversion's owner becomes the request's "guard as an input":
  `TEMPER_MANAGED_PERSON`, exact equality, no default in `live`. It is still one login, so the guard
  reaches no further than it did. The word is `setup.md`'s: "the person creates the App".
- **The fake's `User`** was that same person. It stays a User under the neutral `managed-person`, and
  `fabrig-ai` becomes an `organization` there, as it is on GitHub. The dry run then shows what the live
  run will meet, an open pull's GraphQL read included (the task's pre-approved option).

## What was decided

- **The sitting's fake run keeps `fabrig-ai`,** not a neutral owner. The request sets the script's `OWNER`
  to `fabrig-ai`, the fake run is its dry run, and a neutral owner there would make the guard's owner a
  per-mode input, the thing the request warns against. The neutral owner is in the relay's tests, whose
  policy now reads the guard.
- **`startup.rs`'s test keeps `fabrig-ai`:** it proves the live-shaped configuration, and `acme` is what
  the template holds already, so it would prove nothing.
- **Neutral names:** `acme` and `alice`, as skein's fake fixtures have them.
- **The scripts' tests are the fake sitting itself:** no test reads them. Implementers may run it
  (the script's header). If it can't run in a slot the task says what stands in.

## Tasks and critical path

`neutral-owner` (`needs = []`) is the whole path, about 75 minutes.

## Overlaps

None could not be avoided. On today's main none of the seven files has moved from this plan's base, and
no draft beside this one names them. `fail-stop-names` touches `testing.md` and `draft/github.md`; this
plan stays out of both, so until it lands `testing.md` 2.7 still gives two owners.

## Left for the next GitHub walk

No live probe runs here. fabrig/qa's next GitHub walk proves it, and a managed sitting there must set
`TEMPER_MANAGED_PERSON` to a login that may install an App on `fabrig-ai`: an org owner, since an owner's
approval isn't built (`temper/docs/design/draft/github.md`, section 18).
