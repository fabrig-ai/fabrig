# look-round-4: Home's everyday and "To begin"

At the user's request (through their laptop session, 2026-11-04). Of Home's small "To begin" they said: "much better, I think it's good enough, let's keep it". So the look requests are released and `temper/docs/design/draft/to-begin.md` is built whole. This plan starts after `look-round-3` (`project-home`: Home, the frame, Ready adopting and naming) and `to-begin-store` (the `Dismissed` family, `Dismiss`, the live read) have landed.

## 1. The look requests

On `origin/look-requests` at `d40071cd81` (rewritten by the user; read each request by the commit named below), on top of `project-home`. **The procedure is `temper/docs/design/draft/look-requests.md`: every step of sections 3 to 5, and its account.** Read each request's commit and whole message: each lists the states not drawn and what is not the look's.

- **`project-home-everyday`:** Home as on any project. Its own state is drawn on the hand-made pages `journeys/project-at-work/` (a project at work) and `journeys/project-after-skip/` (just after the welcome, "To begin" skipped).
- **`project-home-onboarding`,** drawn over it: `journeys/project-landing/01.html` is Home just after the welcome, with "To begin" as an optional card, "Skip" a button, and no Connected while it shows.
- **`to-begin-more-accounts`** (`0e816f6749`), directly over `project-home-onboarding`, accepted by the user ("link is great, accepted"): a link to Accounts after Home's lede while "To begin" shows, and a shorter lede. The link's words follow the accounts the person holds. The commit message gives the states not drawn. `to-begin.md`, section 2 carries them once `ready/to-begin/more-accounts` lands, with the rule that the words promise no capacity.
- **`home-drops-connected`** (`2c6f5a5146`), directly over `to-begin-more-accounts`, the user's decision ("re Connected let's drop it from the home page, it will go to settings, once we design it"): removes Home's Connected section and its `temper-home-lines` rules from every Home: the at-work Home and the one after "Skip". The onboarding's Home already had none.

- **`configure-chat-in-settings`** (`53ba7876f8`), released, the user's decision: "Tell temper how you work" opens the configuring chat's own page under Settings, in the project's frame (the rail stays), empty until the person's first words, from template 3 with template 1's fallback. It replaces the offer's canned first message, "Help me tell temper how I work." The commit message says what is the look's and what the landing settles with the design or asks: the page's address under Settings, and the Settings page itself, which isn't designed yet.

Two requests above it are held: `new-menu` (`851708d039`) and `configure-chat-draft` (`d40071cd81`). **Do not land them.** The earlier held `configure-chat-frame` is folded into `configure-chat-in-settings`.

Oldest first, stopping at any request still held. Every commit carries its request's `Look-request: NAME` trailer directly above `Co-Authored-By:`, a later fix and a red's repair included. Each request's last commit holds its account table.

**Re-record `project-landing`** once the welcome leads there (`temper/docs/design/client/journeys.md`, 8), or say what stops it. Plan `fake-signin-model` is fixing what stopped `look-round-3`'s recording: a fake box's ChatGPT sign-in left Ready with no model. If it hasn't landed when this plan's recording runs, say so and end that task with a `Question:`, rather than working round it. Reach `project-after-skip` and `project-at-work` by their hand-made pages as drawn.

## 2. The draft

`temper/docs/design/draft/to-begin.md`, sections 2 to 4:
- "To begin" on Home;
- its steps, computed from the project's home repository and its adoption, its chats and the settings' changes;
- when it shows.

Wire "Skip" to the `Dismiss` request of section 5, which `to-begin-store` built. Keep `to-begin.status.md` current. The draft's section 7 has Open questions; take its default where it has one, and record what you decided in `plan.md`, so that it reaches the user. A change to a stored format, a protocol or a public interface beyond what the draft says is a `Question:`.

## 3. A look fix from an earlier round

`ready/look-fixes/playground-pane` holds two commits that never merged, and they conflict with main: `a3616ef9e9` (the playground's line in the chats pane alone, showing on every page only while it is leaving) and `4bf27f716c` (a chat in the side pane as a compact line). The queue refused it at 00:55Z on `temper-client-shell-terminal`'s golden pages.

With `project-home`'s frame on main, land whichever of them still holds on the current pages, and drop whatever the frame supersedes. **Say which in the report**, so that it reaches the user. One small task, after the requests above.

## How the cut shapes it

- **One order:** the design from the draft first; then `project-home-everyday`; then `project-home-onboarding` with the draft's sections 2 to 4 and "Skip", since the request's page needs them; then `to-begin-more-accounts`; then `home-drops-connected`; then `configure-chat-in-settings`; then the recording.
- **Screenshots come back to the user:** beside each request's pages at 1440, 1000 and 420 wide, plus **1366 by 630 for `project-landing`**, with open states, in `~/src/fabrig/look-round-4/<request name>/` on the builder, outside any worktree, the report saying which builder. Never committed.
- **Throwaway render scripts** stay untracked and are gone before the push.

## Size

One cut is at most 90 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).
