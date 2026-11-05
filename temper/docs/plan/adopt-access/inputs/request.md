# adopt-access: adopting a repository gives the bot access and a hook, on a forge connected by its admin

Found on a real deployment on 2026-11-05 at `f9955bf`: a managed Forgejo, connected with `temper ctl forge connect 1 forgejo-admin FILE`, whose token has `write:admin, write:organization, write:repository, read:user` and answers the admin API. `temper ctl repository adopt` of a repository in an organization the bot isn't in left undone two things `skein/docs/design/forge-setup.md`, 4.5 says it does:

- **Step 3:** the bot stayed read-only over three tries, until a person gave it write.
- **Step 4:** no webhook on the repository, in Forgejo's database, after a re-adopt and after an engine restart, with `application.webhooks` on `127.0.0.1:8368`. An engine start's reconcile step 6 didn't do them either.

Polling still worked, as `temper/docs/design/protocol/forge.md` promises, so nothing was lost but latency, once the bot had its write by hand. But an adopted repository the bot can't write to fails its first run.

## The task

1. **A failing test first,** at the lowest tier with a real Forgejo: the process tier's owned or throwaway Forgejo, as `connect forgejo-admin` has it. Connect as a site admin, adopt a repository in an organization the bot isn't in, then assert: the bot has write, one hook with the listener's URL, and the same after a restart with the bot's write and the hook removed by hand.
2. **Find why.** From reading only, nothing confirmed: the plan's in `skein/crates/skein-forge-setup` (`forgejo/reconcile.rs`, and the `Adopt` plan), run by `temper/crates/temper-service/src/setup/lifecycle.rs`. Check whether a forge connected as `forgejo-admin` reaches them at all: whether its admin token is kept as `Item::AdminToken` for a forge that's `managed`, and whether temper starts the `Adopt` plan or the reconcile's step 6 for that kind. Also whether `managed` differs from `owned` here by design: `skein/docs/design/forge-setup.md` and `temper/docs/design/protocol/setup.md` say what each is given. If the design meant the plans for `owned` alone, say so in a `Question:` rather than widening it.
3. **Fix it in the lowest tier.** A refusal the plan met must reach the person: a repository adopted with the bot read-only, with no word, is the worst of it. If anything stops short, `repository adopt` says what, or the forges row does.

No test reaches a real host. skein's `skein-forge-setup` isn't on hearth's surface. Leave fabrig/ui's files alone. A stored format, a protocol or a public interface change is a `Question:`. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Size

One or two tasks.
