# fetch-progress-1: the fetch's progress, and the bar's fill

At the user's request (through their laptop session, 2026-11-04): they asked whether this had landed and want it scheduled.

## The scope

1. **`temper/docs/design/draft/fetch-progress.md`, sections 2 to 5, and its tests (section 7):**
   - the fetch counting its bytes;
   - the box's `fetched` line;
   - serve keeping the latest in memory;
   - the clients' words: "Setting up: fetching Forgejo · 42%".
   
   Its status lists the parts the look request `welcome-fetch-percentage` left to it (Design lags 5, 6).
2. **The bar's fill: skein's `skein/docs/design/draft/progress-value.md`:** a fraction on skein-ui's progress node, the web mapping's `aria-value*` and `data-progress`, and `skein-ui.css`'s `--skein-ui-progress`.
3. **temper's side:** `progress-value.md`, section 9, step 2, and `fetch-progress.md`, section 6. The meter's style token and rule landed in round 2, ready for a value.

## How the cut shapes it

- **The design first,** from the two drafts, then skein's node, then temper's fetch and box line, serve, and the words and the fill. Keep both drafts' status files current.
- **The look requests' trailers:** a commit that finishes `welcome-fetch-percentage`'s deferred rows carries `Look-request: welcome-fetch-percentage` directly above `Co-Authored-By:`.
- **A change to a stored format, a protocol or a public interface beyond what the drafts say** is a `Question:`. Decisions go in `plan.md` and the cut's report, so that they reach the user.
- **skein-ui isn't on hearth's skein surface,** but name the change in `plan.md`.
- **Beside it:** `look-round-3`, `look-round-4`, `to-begin-store` and `repository-copies-1` are building. Expect catch-ups.

## Size

One cut is at most 90 minutes. Release early. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).
