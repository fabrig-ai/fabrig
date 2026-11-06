# ui-repo-link: the plan

The request: a project's repository is one click to its page on the forge (`inputs/request.md`). Part 1 is worth having alone and went first.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| link | `repo-link` | | the link everywhere a repository is named with its forge known |
| issuer | `provider-public` | | skein's public client; jig's `"secret": null`; a link provider in `addresses` |
| box | `box-sign-in` | `provider-public`, `repo-link` | key, document, claims, `auto`, refused logins, the source's step, serving, the Forges line |
| proof | `box-proof` | `box-sign-in` | the real Forgejo's story, the browser's walk, the attacker's review |

## Critical path

`provider-public` and `repo-link` side by side, then `box-sign-in`, then `box-proof`.

`box-sign-in` waits for `repo-link` because both write `temper-client-view` (`forge_sign_in.rs`, `forge/forges.rs`, `words.rs`): one writer per crate, and its edit there is small.

## Decisions

The request's six decisions are in `temper/docs/design/protocol/sign-in.md`, 9.9, with their reasons. What the cut decided beyond them:

- **A folded Forges row's summary stays words,** and the link is in the row's body. A disclosure's description is text inside `<summary>`, and a link there is the trap the request names for a button. Moving one out would redraw the row. This is a narrowing of the request's "wherever a project's repository is named".
- **Encoding:** a name's bytes are percent-encoded as a path's segment, then again in `redirect_to`, so a hostile name ends neither.
- **`application.sign_in` stays `null` on a box,** as decision 6 has it. `sign_in.forges` names a number, known only at runtime, and `sign_in` holds Google and members, which a box lacks. So the box's entry is `application.machine.owned.sign_in`, a client's id, standing for the kept forge of mode owned, under 9.7's check.
- **The key is `openssl`'s.** skein has no key generation: `ring` signs and does not generate, and no RSA crate is in the lock. So `init --box` makes it as `temper forge sign-in` does, at the first run, before the document; with no `openssl` the provider is `null`, the links are plain and nothing else stops. A new cryptography dependency wasn't worth it for this.
- **The login:** the link provider gives `owner` and `Owner` today, which would match no account. With `machine.owned.sign_in` set it gives `machine.owned.person.login` and `Owner`; with it `null`, a box is as it was. This changes a fresh box's party login, so `box-sign-in` runs its readers' tests.
- **`addresses` takes a link provider:** it refused any but application. A box's owner is provider 1, kind link, and needs an `email` claim. The owner's subject becomes the person's address.
- **The public client ignores what it's sent.** Forgejo's client library sends Basic with an empty password, so a public client's password is read and dropped. That also makes the request's fallback, a fixed placeholder, a one-word change in `add-oauth`'s arguments.
- **The set-up's step is one the group ignores the ending of,** a new property of a step in the box's machine: a non-zero exit, a signal or "did not start" all go on to `started`. Exiting 0 in `temper forge source` alone wouldn't hold "never a welcome failure" against a kill.
- **"The status saying so"** is the Forges row's line, through a page fact (`temper-forge-sign-in-off`), and the log's named reason. There's no `temper status` command, and a wire field would make every journey recording stale for one sentence. The terminal doesn't show it.
- **Four tasks,** as the request asks, so `box-sign-in` is long (about 170 minutes) rather than split. It commits as it goes.

A stored format, a protocol and a public interface change here, as the request allows, with no compatibility: the box's document, jig's `ProviderClient`, skein's `Client`, the box's pipe line. An existing box's document has no provider and no `machine.owned.sign_in`, so by the design its links stay plain and its login stays `owner`; no task tests an upgrade, and `box-proof` says so.

`temper ctl` and a coding run's credentials aren't touched: they use the bot's and the person's tokens, which don't pass this provider. `box-sign-in` checks and reports this, for temper/forge.

## Overlaps with other drafts

- `repo-link`: `temper-client-view/src/words.rs`, `frame.rs`, `temper.css`'s switcher block, as `ui-pause-close`, `model-catalogue-1` and `chat-reading-c` also edit. Each edit is local. Goldens and the gallery are regenerated on a catch-up.
- `box-sign-in`: `temper-init/template/temper.json`, as `model-catalogue-1/stock-seven` edits (its `llm` block; ours `owners`, `clients`, `machine.owned`). `temper-box/src/tests/`, `temper-service/src/route/`: no other draft's named files overlap. `temper/crates/temper/tests/support/`: `model-catalogue-1` holds `first_repository.rs` and `forge_land*`, which `box-proof` only reads.
- `provider-public`: `jig/crates/jig-protocol-config/src/members.rs`, as `model-catalogue-1/effort-config` did (merged), and `temper/docs/help/reference/configuration-engine.md`, regenerated.
