# look-round-2: land this round of look requests

At the user's request (through their laptop session, 2026-11-04): land the five look requests on `origin/look-requests` at `7bbb7ec2d7`, none held. **The procedure is `temper/docs/design/draft/look-requests.md`: follow every step of sections 3 to 5.** It is new, and what it asks beyond `temper/docs/design/client/journeys.md`, 7 is the point of this round. The last round missed markup and states given in words, the terminal, a part another plan drew, and fixes without the trailer.

## How the cut shapes it

- **One task per request,** in the order below, each needing the one before: they build on one another's pages and share the stylesheets.
- **Each spec quotes its request's commit message whole** and names the commit. The implementer reads `git show <commit>` itself as well (section 3, step 1).
- **Every commit of a landing** carries the request's `Look-request: NAME` trailer directly above `Co-Authored-By:`, a later fix of what it missed included; the repair of a landing's red too. Never the request's own commit, and nothing of `look-requests` merged. The trunk is named `look-round-2`, not `look-requests`, so that it never meets the person's branch.
- **Each task's report holds its account table** (section 5), which is also in the landing's last commit, and the render check's result (section 4).
- **The screenshots come back to the user:** each task keeps them, beside the request's pages at 1440, 1000 and 420 wide with the open states, in `~/src/fabrig/look-round-2/<request name>/` on its builder, outside any worktree, and the report says which builder. They are never committed.
- **Throwaway render scripts** stay untracked and are gone before the push (`AGENTS.md` 4). The render-check command is that document's section 8, Later: don't build it.

## The requests, oldest first

1. **`welcome-model-forge-pane`** (`87cddc4453`): the forge's state in a pane beside the Model step. Land it whole: its done state from the message's words, the narrow width, the terminal.
2. **`ready-account-email`** (`0a94e5f1fb`): the view needs no change. The email is the engine's: the user chose B, and `jig/docs/design/draft/account-facts.md`, under the user's review and not on main yet, designs it. **Do not build it.** Land the gallery world's email-like display only. The rest is a deferred row of the account, to that draft.
3. **`welcome-forge-busy-bar`** (`95532806d8`): the moving bar under the setup line. Land it whole.
4. **`welcome-fetch-percentage`** (`218a9b9080`): the percentage and the fill need data the client does not have. `temper/docs/design/draft/fetch-progress.md`, on main, carries the percentage, and the fill needs a skein draft not written yet. Land what the view can without them: the meter's style token and rule, ready for a value. Account the rest as deferred to those drafts.
5. **`welcome-claude-card-words`** (`7bbb7ec2d7`): the Claude card's words. Land it whole. The card missing from a real box is `temper/docs/design/draft/claude-provider.md`'s, on main with a change queued as `ready/claude-provider/always`, not this request's. Leave that change's files to it.

## Questions

Section 6 holds: an engine's fact goes to the user as a question when it changes a stored format, a protocol or a public interface, or becomes a draft otherwise, and the account names it. A conflict with a design or a test the landing cannot settle is a question, never the landing's choice (section 3, step 2).

## Size

One cut is at most 90 minutes. No new design of its own: the design follows each request, as section 3's steps 2 and 8 say.
