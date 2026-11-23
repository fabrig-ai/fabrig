# browser: status

2026-11-20. Where temper's code lags or departs from [README.md](README.md).





- **Not yet:** `look.md`, *What a phone presses*: at 760 px and under an Ask
  temper chat's thread keeps the mock's sizes, its "Latest" and a
  proposal's acts among them, and the path's stops on Ask temper's page
  take no target past their marks: `temper.css`'s 44 px targets reach a
  chat's bar and composer and the opening's tabs, acts and zoom link
  alone (`each_phone_target_stands_44_pixels`).

`look.md`'s phone paragraph and *What a phone presses* have no other
known implementation gap, by plan `ui-p5a-phone`: the tab bar's
"Needs", the Ask tab's tabs kept under Ask temper's page and an Ask
temper chat by `temper-tabbar-ask`, Needs you's open card under its own
bar, `temper-needs-bar`, the rail's switcher and Away chip not drawn
there, and at 44 px to a press the top bar's switcher, Away chip and
night switch, each tab, and the controls of Home, Needs you, Work and
every chat's bar and composer, the mockups' smaller drawings kept inside
a transparent target. `each_phone_target_stands_44_pixels` reads them
from the stylesheet, and the gallery's `needs-list`, `ask-chat` and
`rail-new-change` prints hold the bar, the Ask tab and "Needs". Their
render check follows landing (`../README.md`, section 5, *The phone after
landing*).

- **Design departs:** `public.md`, 3, from the request `ui-public-2` of 2026-11-06:
  ended rows say how long ago they ended, where the request says "for how
  long", because "landed for 2 hours" would not describe the ending. The
  board shows its Landings section with a queue or a retained landing,
  where the request says only with a queue: the plan's `public-page`
  contract asks for either, and records no reason for widening that rule.

`look.md`'s terminal rules, tokens and bundled fonts and `gallery.md`, 2's nine
`chat-terminal-*` gallery states have no known implementation gap beyond
the departures below. The conversation's terminal ground outranks the
shell's Split and Draft grounds, including phone Split. The pane has a
thin scrollbar and no reserved gutter at every width. The quiet model
button is plain, and the running line is first inside the composer form,
before the box, with its outer margins removed. The existing view table
checks that parent and order, the plain button and Exit's retained token.
The repair accounts are `463459a7d1` (`chat-terminal`) and `078e99c4ed`
(`chat-terminal-draft`); the stylesheet and tree establish no measured
line heights, pane geometry or rendered equality. The shared shell
document and flow rules win over the request's restated strong,
code, marker and figure-margin rules, as `look.md` requires. Temper/ui's
render comparison follows landing (`README.md`, section 3;
`look-requests.md`, 4 and 5). Compilation and print checks
establish no rendered equality.

- **Departs:** `look.md`, *A chat's terminal*: at 760 px or less, DejaVu is 13px,
  but Plex stays 14px because `.temper-terminal-plex` has greater specificity
  than the phone's `.temper-terminal` rule. The cascade retains the desktop
  Plex size instead of the requested single phone size; temper/ui owns it.
- **Departs:** `look.md`, *A chat's terminal*: adjacent local notes share the
  `.temper-terminal-note + .temper-terminal-note` margin, including help's
  code block, where `chat-terminal-font` narrows its mock selector to
  paragraphs. One note token retains uniform spacing. The shared options
  menu also has no mousemove selection handler, so hovering an option
  leaves the active keyboard choice where it is, unlike the request's
  handler. Temper/ui owns these differences from the request.
- **Design departs:** `look.md`, from the request `ui-phone` of 2026-11-05, decided by
  its plan's cut and its sweep. A chat's foot stands 16 px in, where the
  request's check has the mock's 12: the look request `configure-temper-name`
  moved it for every chat, and the request keeps look requests made. A chat's
  foot and the toast take the home bar's inset, which the request asks of the
  tab bar alone: a covered page leaves them under it. "Every tab and primary
  button at least 44 tall" holds where the mock draws no shorter: the welcome's
  primaries stand 40 px, and a goal's primary outside its
  choices, the sheet's among them, 40, as the mock's `.btn` and `.btn.small` do.
  The mock's `box.html` draws a terminal, what `temper box` prints, and no page
  of the client's draws it.
- **Design departs:** `look.md`, from the request `close-chat` of 2026-11-06, decided by
  its plan's cut: a confirmation sheet's acts stand 44 px on a phone, where the
  request's overlay has 38, to keep a phone's target. So at 420 px the sheet stands
  6 px taller and 3 px higher than the request draws it.
- **Departs:** `look.md`, *The first paint*: handing over fades in 160 ms, where this section
  has 200: `loading.css` takes `transition: opacity .16s` from the request
  `bundle-loading`'s own stylesheet, whose message says 200 ms, because
  `look-requests.md`, 3 lands a request's rules as its stylesheet has them. The
  loader removes the placeholder at 200 ms all the same. `#boot` also sets
  `letter-spacing: normal`, beyond that stylesheet, since an arriving `temper.css` would
  otherwise move its words by -0.18 px.
- **Design departs:** `testing.md`, 3, 6, from the request `browser-trim` of 2026-11-06, decided
  by its plan's cut. Goals' link's target is held by the view's
  `the_link_shows_with_a_tracked_template`, where the request has the client world,
  because the view is the lowest tier that shows it. Story A's download is compressed
  by the machine's `gzip`, where the request looks in the playground for a missing cache
  or a needless rebuild alone, because `skein_http::gzip::encode` in the dev profile
  took 4.85 CPU s of the story; so the printed figure's bytes changed (`../../playground.md`,
  9).
- **Not yet:** `look.md`, *The welcome's look*: while `me` is read at
  `/welcome`, once the deployment's read has answered, the top bar draws
  temper box's line and the steps (`../view.md`, 1) without the
  welcome's look: its rules select `#app:has(> .temper-welcome)`, the
  welcome's main, and the main is then empty, with no class, so the
  steps take the flows' list rule (`.temper-flow-steps`, a grid) and the
  bar the outside pages' row. Seen by reading `temper.css`; no render
  shows how long it lasts.

- **Design departs:** `look.md`, from the look request `outside-pages` of
  2026-11-08, decided by its plan's cut: `--temper-accent` and the light
  chat terminal's `--accent` take the signal, `#c2410c`, which the
  request does not draw, so that no source of the bundle names the old
  mark's `#b8664e`, as its test asks.

`addresses.md`, section 1, and `gallery.md`, section 2 have no known implementation gap in Home and Needs
you across projects, by plan `ui-p2-home`: `/home` for `AllHome`, which
the domain's address table parses and prints; `temper.css`'s `/*
==== ui: home ==== */` and `/* ==== ui: needs ==== */` holding Home's
cards, Projects, the Ask temper line, Needs you's list, its closed rows,
its *Tell it something else* and Decided for you, with their phone
forms; and the gallery's `all-home`, `all-home-nothing-new`,
`all-home-one-project`, `night-all-home`, the four `home-step-*`,
`needs-list`, `needs-list-second`, `needs-tell`, `needs-approval-first`
and `night-needs`, which its table checks. `gallery.md`, 2's states hold no red
"owned" and no Home at the standing roles' step, which nothing yet draws
(`../../draft/surfaces.status.md`, 14). The
rendered comparison follows landing (`../README.md`, section 5, *Home and
Needs you across projects after landing*).

`gallery.md`, section 2 has no known implementation gap in Only you, by plan
`ui-n3a-only`: `temper.css`'s `/* ==== ui: needs ==== */` holds the
region, its item and its question, and the item's link full width on a
phone; and the gallery's `needs-only-you`, a question then "Only you ·
2", a signed-out ChatGPT sign-in and an expiring Claude Code login,
`needs-only-you-done`, "Home" then "Only you · 1", `night-needs-only-you`
and `accounts-expiring`, "Expiring: sign in again before it ends." and
"Continue with ChatGPT", each with its row of
`each_state_holds_what_its_name_says`. The rendered comparison follows
landing (`../README.md`, section 5, *Home and Needs you across projects
after landing*).

`gallery.md`, section 2 has no known implementation gap in Away on All projects, by
plan `ui-away-all`: the gallery's `all-home-away`, the switch and the
chip "Away on 2 of 4", not pressed, and `all-home-away-on`, after a
press each project's answer Away, the switch "Away until 08:00",
pressed, and the toast; its table checks a button of each state's words,
and its state, a control not pressed by `Part::NotPressed`. `temper.css`
has no rule of its own for the words: whether they fit the switch and
the chip is the rendered comparison's, after landing (`../README.md`,
section 5, *Away on All projects after landing*).
