# ui-repo-link: a project's repository is one click to its page on the forge

The user, journey 14 (2026-11-06): "the cowsql/vsr repo should be a link that opens a new tab and leads you to its home page in the relevant forge, no signin for owned Forgejo (via oauth), one click just works". In the recording they pressed "Vsr2 cowsql/vsr" in the switcher, which is one `button` (`project_row`, `temper-client-view/src/frame.rs`).

Two parts, in order, and the first is worth having alone: release it early.

## 1. The link

A link can't sit inside a button, so the row becomes two siblings: the project's name, a button as now, and the repository, a link in a new tab (`rel=noopener`). Same place, size and colour as today, with "↗", underlined on hover. "+N more" stays outside the link, which goes to the first repository. With no repository or no `web` address (the playground) the text stays plain. The same link wherever a project's repository is named: Forges' `OWNER/NAME · project PROJECT`, the Forge line, Settings' Repositories.

The address: GitHub or another forge: its `web` plus `/OWNER/NAME`. A Forgejo whose page facts offer sign-in: `ROOT/user/oauth2/temper?redirect_to=%2FOWNER%2FNAME`. Encode OWNER and NAME.

## 2. One click, signed in, on a box's owned Forgejo

A box has no provider today: its `engine.clients` has no `provider`, `application.sign_in` is null, its Forgejo has no `temper` source, and `protocol/setup.md`, 6.8 prints the command rather than running it. So a box's owned Forgejo sets this up by itself, with no command for the person. "Open Forgejo ↗" then appears on a box too. `temper serve` with an owned Forgejo keeps its printed command for now: the box first.

A laptop session tested this on a throwaway Forgejo 17.0.1, with curl and a stand-in OpenID provider, not temper's, against a site-admin account with a password, as the set-up makes. Repeat it against temper's own issuer before relying on it:
- `ACCOUNT_LINKING = login` (`protocol/sign-in.md`, 9.4, what an owned Forgejo writes): lands on Link account, asking for the password: the first thing that breaks one click.
- `auto`, matching name, different address: lands on the repository, signed in, three redirects, no second account.
- `auto`, no `email` claim: lands on Link account, asking for the password.
- A second visit in the same browser goes through the provider again, so every click passes temper's `/oauth/authorize` page: keep "Continuing to Forgejo…" brief.

### Decided (temper/ui)

This is an authentication change, so the reasoning stays visible in the design, and it gets a review (below).

1. **`ACCOUNT_LINKING = auto` on a box's owned Forgejo only,** where temper made every account and one person holds the machine. A managed Forgejo stays `login`, and `temper serve`'s owned stays as it is until the same reasoning has been checked for several members. **Temper never signs a person in as the bot:** a member whose login is the bot's is refused, since `auto` would hand them the bot's account. Say what else in Forgejo a name match could take, and how a box rules it out.
2. **The claims:** `preferred_username` equals `application.machine.owned.person.login` (the machine's login on a box), and `email` is present. A box's owner today is provider 1, `link`, subject `owner-a`, and `email` comes only from a provider in `addresses`, so the box's owner needs both. What login the link provider gives wasn't checked: check it.
3. **The provider's key:** `temper forge sign-in` makes it with `openssl`. The box makes it itself, in-process if skein has what it needs; with no way to make it, the link is plain and nothing else stops. The `redirect_uri` is known at init, since the owned `listen` socket is written there. A key file's mode and place follow the other secrets'.
4. **The client's secret:** `add-oauth` takes `--secret` only as an argument (its `--help`), which 6.8 and the repository's rule refuse, since a command line is visible to other local users. **Prefer a public client in the issuer,** with PKCE S256 required (skein's `oauth-issuer.md` already requires it; Forgejo sends it, as seen in the test) and the redirect exact, over a fixed value dressed as a secret: nothing pretends to be secret. If Forgejo can't register a client with no secret, register a fixed, public placeholder, and say in the design that it isn't a secret and why that's safe. If the issuer's public client would weaken other clients, it stays per client. Confirm by test: a wrong redirect, no PKCE, and a wrong verifier are each refused.
5. **A step at the end of the owned set-up:** if it fails, the set-up still succeeds and the link is plain, with one line in `box.log` and the status saying so. Never a welcome failure.
6. **`sign_in.forges` lives under `application.sign_in`, null on a box;** a forge's number is known only at runtime, which 9.7's check already allows for.

It changes `protocol/sign-in.md`, 9.4 for owned on a box, and `protocol/setup.md`, 6.8 for the box: change them with it, with the reasoning above, and tell temper/forge if `temper ctl` or a coding run's credentials are touched: they shouldn't be.

## The proof

Failing tests first, at the lowest tier: the address's building, refused members, the issuer's refusals, the set-up step's failure leaving the box running. Then a browser walk on a fresh fake box with Forgejo on this machine: the switcher's link opens a new tab on the repository's page, signed in, with no password, no Link account page and no second account, and a second click the same; the project's name still switches project; Forgejo's own login with the password still works. Screenshots at 1440 and 420 in `~/src/fabrig/ui-repo-link/` on the builder, outside any worktree. If the owned Forgejo can't run in that tier, use the process tier, and say what a browser didn't show. **Review:** the last task reads part 2 as an attacker on the same machine and in another browser tab would, and lists what's possible; fix or report each.

Note for the report: what isn't proved on a real, non-fake box.

## Size

Three or four tasks. One cut is at most 25 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/protocol/sign-in.md`
- `temper/crates/temper-client-view/src/frame.rs`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
