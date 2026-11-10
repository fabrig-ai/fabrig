# To begin

Draft, 2026-11-04, from a session with the user that day, whose
decisions it records. A project's Home opens with a small "To begin"
while some of its first steps are left: the greenfield mockup's
first-run steps, made small, with "Skip". What shows is computed from
the project's state, so a refresh, another device or a restart shows
the same; "Skip" is the one thing kept, by the engine, for the person.
The look is tuned on the person's `look-requests` branch
(`../client/web/journeys.md`, 7), as the request `project-home-onboarding`
draws it, and landed apart from this draft.

## 1. In one page

- **On Home, at the top:** the intro and "To begin", a card of its
  own marked "Optional", with "Skip" a button, its steps one row each.
  Nothing else on Home while it shows, but what needs the person and
  the chats once there are any. It fits a laptop's first screen
  (section 2).
- **Three steps:** "Tell temper how you work", the project's
  repository, and "Start a chat", each with its act while it is left
  (section 3).
- **Computed, not kept:** each step is done or left from what the
  project holds; "To begin" shows on a project while a step is left
  and its person has not skipped it there. A project the welcome has
  just made has steps left, whatever was chosen in it (section 4).
- **"Skip" is kept by the engine,** per person and project, in a family
  of temper's store, so it holds on every device and across restarts
  (section 5).
- **Nothing is lost by skipping:** every step's act has its own place
  elsewhere (section 5).
- **Changed in place:** until v0.1.0 there is no supported deployment,
  so the store's families, the client's records and `CLIENT_VERSION`
  change in place, with no version kept for an older side (the user,
  2026-11-04).

## 2. On Home

- **The intro,** while "To begin" shows: the kicker as on any Home,
  the day and the time, enough for now (the user, 2026-11-04); the
  headline "PROJECT is ready.", never a
  count of what is left, since nothing in "To begin" is owed; the
  lede naming the model's account and the forge, "Your agents run on
  your ChatGPT plan, and Forgejo on this machine.", from the LLM
  accounts and the forges the client reads already; after it, a link
  to Accounts naming what the person does not hold yet: "Add Claude or
  another plan" with a ChatGPT plan alone, "Add ChatGPT or another
  plan" with a Claude subscription alone, "Add another account" with
  both or with a key (the user, 2026-11-04).
- **More accounts, never more room:** the link offers other providers
  and models, and its words promise no capacity, since a limited task
  waits for its own account and never moves to another of its
  holder's (jig's `domain/accounts.md`, 4.3), as OpenAI's terms ask
  (jig's `domain/providers.md`, 2.5), and Anthropic's limits assume
  one person's ordinary use (jig's `domain/providers.md`, 3.4).
- **"To begin," optional and plainly so:** a card of its own, headed
  "To begin" beside an "Optional" tag, "Skip" a button at its top
  right; under the heading, "Two steps left. Skip them if you like;
  they stay in Settings and Chats.", its count the steps left. Then
  one row a step, in section 3's order: its mark, the step's number or
  a check once done; its title over one short line; its act at the
  row's right while the step is left (the user, 2026-11-04).
- **Nothing else but what has content:** Needs you and Chats when they
  hold something, as on any Home; empty, they are left out, since the
  steps stand for them. Connected and the landing's line wait for
  "To begin" to go: the lede says what Connected would, the second
  step says the repository, and nothing has landed yet (the user,
  2026-11-04).
- **Small:** "To begin" and the intro fit the first screen of a 1366
  by 768 laptop with its browser's bars, and of a phone (the user,
  2026-11-04).
- **Gone,** once skipped or once every step is done: Home is as on any
  project, its intro what changed since the person last looked.

## 3. The steps

| Step | Done once | Left: its line and act | Done: its line |
|---|---|---|---|
| 1. "Tell temper how you work" | a configuring chat (`configuring.md`, 5.1) runs or ran in the project, or a setting the project reads holds a change (`configuring.md`, 3.2) | "temper runs its default way of working until you change it.", "Start", filled: the configuring chat, as the chats page's offer starts it (`first-run-projects.md`, section 6) | its title alone |
| 2. the repository | the project has a home repository (jig's `domain/people/acts.md`, 2) | "Adopt a repository", "Adopt", to the adoption form into the project (`first-run-projects.md`, section 5) | "ORG/NAME is in PROJECT", "temper can change its code now; checked changes land on main through temper's merge queue.", one line on a laptop |
| 3. "Start a chat" | the project holds a chat other than a configuring one | "Ask a question or ask for a change.", "New chat" | its title alone |

- **A new project runs on temper's defaults,** its stock policy and
  landing rule (`configuring.md`, 3.2), so the first step is an offer,
  never a gate: the rest works without it (the user, 2026-11-04).
- **The second step names how changes land,** the project's own
  landing setting (`../domain/landing.md`, section 13) and its home's
  checks as the adoption set them. By queue, the default, it says
  temper's merge queue (`../domain/forge/change/README.md`, 2): checked at its
  head, brought up to date with main and checked again if main moved,
  merged at exactly that head. That queue, and its checks, are what a
  project of many changes lands through, more than a forge's own
  "green, then merge" (the user, 2026-11-04). By turn, it names
  temper's landing turn (`../domain/landing.md`, section 1).
- **The adoption under way,** as Ready leaves it running
  (`first-run-projects.md`, section 4): the second step says "Adopting
  ORG/NAME into PROJECT…" with no act; refused, the refusal's words and
  "Adopt" again, filled for the first repository.
- **Until the configuring template is built,** the first step's act
  starts the stock chat, as the chats page's offer does
  (`first-run-projects.md`, section 6), and only a setting's change
  marks the step done.
- **The order is the mockup's,** and a done step keeps its place.

## 4. When it shows

- **On a project's Home while a step is left and its person has not
  skipped it there.** Nothing records that the welcome made the
  project: a project the welcome has just made has its first and third
  steps left at least, and so has a project made later from the
  switcher (`first-run-projects.md`, section 7), which may need its
  repository too.
- **From what the client reads already:** the project's home repository
  and its adoption's state, its chats, the settings' changes; and the
  person's skips (section 5). A refresh, a restart of the engine or
  another device computes the same.
- **The playground's project** is as any project: its steps are read as
  the others' are.

## 5. Skip

- **"Skip" hides "To begin" for the person on that project,** at once,
  on every device: the project's other people still see theirs, and the
  person's other projects keep theirs.
- **Kept by the engine,** never in the browser alone, so a phone does
  not show again what the laptop skipped (the user, 2026-11-04). A
  person's state kept by the engine has a precedent in jig's inbox,
  whose read positions are a party's own (jig's `domain/people/README.md`,
  section 5).
- **A family of temper's,** `Dismissed`, at the store's next free byte
  (`../protocol/store.md`, section 2), keyed by the party, then what
  was dismissed: `ToBegin(project)`, the one kind for now; its row
  holds its key's fields. A family of the forge connector's though it
  is people's, as `Binding` is. Written by the person's request,
  erased with its project or its party; bounded by the members times
  the projects.
- **A FORGE request, `Dismiss { what }`** (`../client/forge/records.md`),
  "Skip" in the view, from any party with a role in the project; a
  second is answered as the first. The client reads the person's
  dismissals, live, so another device's Home drops "To begin" when the
  skip commits.
- **Nothing is lost:** "Tell temper how you work" stays on the chats
  page and on an empty "How work runs" (`configuring.md`, 5.1); a
  project with no repository stays highlighted, with "Adopt a
  repository" (`first-run-projects.md`, section 5); "+ New chat" stays
  in the rail. A skip is never undone, and needs no undoing.
- **One mechanism for later dismissals:** the plan's first use, a card
  that waits to be dismissed, is the client's open question
  (`../client/README.md`, section 5); if it is chosen, it is another
  kind of `Dismissed`, keyed by its account.

## 6. Its order

1. Section 2 and 3 without "Skip": "To begin" on Home, its steps
   computed, as the look request draws it.
2. Section 5: `Dismissed`, the request and the read.

## 7. Open questions

- **Every new project, or the welcome's alone?** Default: every
  project with steps left (section 4), since nothing records which
  project the welcome made.
