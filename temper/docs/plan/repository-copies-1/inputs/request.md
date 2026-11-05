# repository-copies-1: the foundation of repository copies

At the user's request (through their laptop session, 2026-11-04): start the foundation of `temper/docs/design/draft/repository-copies.md`, which copies public repositories into a Forgejo temper runs and adopts them into a project. The user asked for it to be built on reusable primitives, to be offered in the welcome and later in Settings, and said "putting in place the foundation is probably something that can start already".

## The scope: the draft's section 10, step 1

- **skein's setup-host plan `Copy`** (section 3), beside `Create`, with its own deadline.
- **The Forgejo facts it relies on,** recorded by the conformance on the throwaway Forgejo, copying from a second throwaway on loopback (sections 4 and 6).
- **The fake forge's migration route,** and its catalogue of sources under `.invalid` hosts, with the timing faults.
- **temper's FORGE request `MakeRepository`** (`Empty` or `Copy`, optionally `into` a project, as its home or not), adopting what it made once it is ready.
- **The `Making` store family,** with its restart, its limit `forge.makings`, and its rows on the `forges` watch.
- **`temper ctl repository make` and `repository copy`,** with `--until adopted`, so that a copy can be made and adopted from the command line on the fake backends and on a real box.

## Not this plan's

The welcome's choice, Ready's copies, and the "Add a repository" sheet (sections 7 and 9): they wait for the look. **No test reaches a real host.**

## The draft's Open items

They're the user's to confirm. Build on their defaults, which formats changing in place makes cheap to redo:
- Forgejo's own migration, not a clone and a push;
- `MakeRepository` and `Making` as written;
- a copy keeps its source's owner and name;
- the starter workflow in every copy.

Until v0.1.0 there is no supported deployment: formats change in place, with no compat or migration.

## How the cut shapes it

- **The design from the draft first,** then skein's primitive and its conformance, then the fake, then temper's request and store family, then `ctl`. Slice so that the walk, a copy made and adopted through `temper ctl` on the fake backends, can be marked `walk = true`.
- **Keep `temper/docs/design/draft/repository-copies.status.md` current** as the work lands.
- **Report what you decided:** anything of the draft's you had to decide or found unclear goes in `plan.md` and the cut's report, so that it reaches the user.
- **skein is shared with hearth:** name the skein change in `plan.md`; it should stay a generic primitive, with no temper in it (`AGENTS.md` 1).
- **Beside it:** `look-round-3`, `to-begin-store` and `macos-cheap` are building. `macos-cheap` moved the Forgejo pin, and `to-begin-store` takes a store family byte: check `store.md` on the trunk at the time of the batch, and expect catch-ups.

## Size

One cut is at most 90 minutes. Release early. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).
