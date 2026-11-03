# first-run-projects: projects from the first start to the welcome's end

At the user's request (through their laptop session, 2026-11-03), carry out the adopted draft `temper/docs/design/draft/first-run-projects.md` (the user's decisions of 11-03). It runs as a reference milestone does (`temper/docs/design/draft/reference-case.execution.md`, section 2): design and code go through this plan as one pipeline. Its designers write the design into the trunk with the specs that cite it, closing the lags its status file lists (`first-run-projects.status.md`; `DESIGN.md`).

## The scope

The whole draft, in its section 8's order:
1. Sections 2 and 3: no seeded project; Demo, its stored name honoured.
2. Sections 4 and 5: the wizard without adoption, Ready making "Default", the highlighted project with no repository ("Adopt a repository").
3. Section 7: rename (jig's Rename, owners only) and a new project (MakeProject) from the banner's switcher.
4. Section 6: the configuring chat as the first chat. For now this is the interim: a stock chat with no repository, until configuring's template is built (`configuring.execution.md`, P3).

## Its open questions

Section 9 leaves two decisions open. The user is asleep and the track runs unattended, so take the conservative, reversible reading below. Record each in the draft's status as Open, for the user's review; do not decide them in the design text.
- **A taken "Default" at Ready:** open the existing "Default" rather than making "Default 2", since it makes no new name and is easy to change.
- **Demo after a reset:** a second playground run makes "Demo" again and reuses nothing but the name, as the bullet itself says.

## What runs beside it

- **Plan `look-welcome-chats`, started 02:45Z:** it moves the look requests on `origin/look-requests` (the welcome-playground journey) into the view and the stylesheets. That covers the terminal-like chats page, the left pane, the quieter banner, the project switcher's markup (`details.temper-projects`) and "Set up temper ->". Those are the look; this plan is the behaviour behind them. Leave the pages' look, `temper.css`, `task-pages.css` and the journeys' prints to that plan, and build on them as they land. Release this plan's switcher and banner behaviour (step 3) last, so it meets the look already landed.
- **Wave 2 of the reference case** (plans `reference-m2`, `-m3`, `-m5`, `-m6`, `-m7b`) works on temper's engine, forge connector and ctl. Configuring's P2 starts later on M3's trunk. Leave their sections alone.

A budget or timeout is the user's (skein's `docs/foundation/testing-strategy.md`, 8.1): a role never raises one.

## Size

One cut is at most 90 minutes. Release early, family by family, in section 8's order. If the whole cannot be designed in one cut, release what holds together and end with a `Question:` naming the rest.

## Walk

Mark the slices that make section 8's steps run through `temper ctl` `walk = true`. fabrig/qa judges and walks them once this lands.
