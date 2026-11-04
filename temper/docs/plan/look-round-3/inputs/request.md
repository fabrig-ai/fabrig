# look-round-3: land the `project-home` look request

At the user's request (through their laptop session, 2026-11-04): land the look request `project-home`, on `origin/look-requests` at `0197eaea31`, not held. **The procedure is `temper/docs/design/draft/look-requests.md`: every step of sections 3 to 5, and its account.** Read the request's commit and whole message first (`git show 0197eaea31`).

## The request

The project's first page becomes Home, as the greenfield mockup's `home.html` draws it, with the frame every project page gets:
- a rail at the left: brand, project switcher, Home, Needs you, Chats, "+ New chat", Settings, and the person at its foot;
- a top bar and a tab bar on a phone;
- Home's column: kicker, headline, lede, "To begin" with three numbered steps, "Connected".

It is drawn on `project-landing/01.html` (the journey the user recorded today, on main) and on a hand-made page, `project-switcher/01.html`, for the switcher's repository words in each case.

Its message names the states the pages do not show (the steps done, in progress and refused; the headline's counts; GitHub's step; the link's state off-live), what it replaces (the banner and its menu, the chats sidebar, the composer and the model line on this page), and what is not the look's. Land every state it names, not the drawn one alone, in the browser and the terminal.

## Two draft changes it needs

Both are on main in `temper/docs/design/draft/first-run-projects.md`, section 4; its status lists the design sections that lag. Carry both into the design and the code with this landing, because the page's drawn state needs them, and the reprint cannot equal the request until Ready adopts and names:

1. **Ready's "Create your project" adopts the forge's first repository** (Forgejo on this machine's `work/hello`) into the project it makes, as `temper ctl forge setup --project` does: owned, the home, landing on `main`, with CI. The project opens without waiting for the adoption.
2. **The first project is offered the first repository's name with its first letter in capital** ("Hello"), or "My project" with no first repository, in Ready's form.

Then **record `project-landing` again** (`temper/docs/design/client/journeys.md`, 8), or say what stops it.

## Choices the request and the drafts leave open

The routes, which repository is the home in `me`, and the adoption's progress words: settle each by the design, or ask, as `look-requests.md`, 6 says. **A change to a stored format, a protocol or a public interface is a `Question:`,** never the landing's choice. If `me` needs a new fact to say which repository is the home, that is such a change.

## How the cut shapes it

- **This is more than a view change:** the frame reaches every project page. Slice it so the design and the Ready adoption and naming can land first, then the frame, then Home, each with its own task, then the reprint and the recording, then the account. Other pages keep working at each step; tests that walk a part the request removes are a question if the request doesn't settle them (section 3, step 2).
- **Every commit of the landing** carries `Look-request: project-home` directly above `Co-Authored-By:`, a later fix included, the repair of a landing's red too. Never the request's own commit, and nothing of `look-requests` merged.
- **The last commit holds the account table** (section 5), and the last task's report holds it and the render check's result (section 4).
- **The screenshots come back to the user:** beside the request's pages at 1440, 1000 and 420 wide, with the switcher open, in `~/src/fabrig/look-round-3/project-home/` on the builder, outside any worktree, the report saying which builder. Never committed.
- **Throwaway render scripts** stay untracked and are gone before the push.

## Size

One cut is at most 90 minutes. Release early: the design batch first. A cut's designers write only what the draft and request leave unwritten (`DESIGN.md`). Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).
